use super::registers::NumBlocks;
use crate::platform::{
    locking_spi_bus::LockingSpiBus,
    sdmmc::{
        command::SendInterfaceCondition,
        registers::CSD,
        response::R1Status,
        transfer::{Token, TokenError},
    },
    set_frequency::SetFrequency,
};
use core::{convert::TryFrom, slice};

use super::{
    card_type::CardType,
    command::{AppCommand, Command},
    response, transfer, BLOCK_SIZE,
};
use alloc::format;
use core::marker::PhantomData;
use defmt::{debug, error, trace};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::spi;
use embedded_hal_async::{digital::OutputPin, spi::SpiBus};

pub trait SdCardBus: spi::ErrorType {
    // type Error;

    async fn go_idle(&mut self) -> Result<(), super::Error<Self::Error>>;

    async fn read_addr<'b, I>(
        &mut self,
        address: u32,
        blocks: I,
    ) -> Result<(), super::Error<Self::Error>>
    where
        I: core::iter::ExactSizeIterator<Item = &'b mut [u8; BLOCK_SIZE]>,
    {
        let num_blocks = blocks.len();
        let cmd = match num_blocks {
            1 => Command::ReadSingleBlock(address),
            _ => Command::ReadMultipleBlock(address),
        };
        self.send_command(cmd).await?;
        for block in blocks {
            self.read_block(block).await?;
        }
        if num_blocks > 1 {
            self.send_command(Command::StopTransmission).await?;
            // debug!("Stopping transmission");
            // self.wait(millis(10)).await?;
        }
        Ok(())
    }

    async fn read_block(&mut self, block: &mut [u8]) -> Result<(), super::Error<Self::Error>>;

    async fn read_csd(&mut self) -> Result<CSD, super::Error<Self::Error>>;

    async fn write_addr<'b, I>(
        &mut self,
        address: u32,
        blocks: I,
    ) -> Result<(), super::Error<Self::Error>>
    where
        I: core::iter::ExactSizeIterator<Item = &'b [u8; BLOCK_SIZE]>;

    async fn send_command(
        &mut self,
        cmd: Command,
    ) -> Result<response::Response, super::Error<Self::Error>>;

    async fn send_app_command(
        &mut self,
        cmd: AppCommand,
    ) -> Result<response::Response, super::Error<Self::Error>> {
        self.send_command(Command::AppCommand(0)).await?;
        self.send_command(Command::App(cmd)).await
    }

    async fn wait(&mut self, timeout: Duration) -> Result<(), super::Error<Self::Error>>;
}

// impl<T: SdCardBus> ErrorType for T {
//     type Error = super::Error<T::Error>;
// }

impl<T: SpiBus + spi::ErrorType> SdCardBus for T {
    // type Error = super::Error<T::Error>;

    async fn go_idle(&mut self) -> Result<(), super::Error<Self::Error>> {
        // SD v1.0 won't be considered
        for _ in 0..32 {
            match self.send_command(Command::GoIdleState).await {
                Ok(r) => match r.r1.has(R1Status::InIdleState) {
                    true => return Ok(()),
                    false => return Err(super::Error::NotIdle),
                },
                Err(super::Error::NoResponse) => (),
                Err(e) => return Err(e),
            }
            Timer::after_millis(10).await;
        }

        Err(super::Error::NoResponse)
    }

    async fn read_block(&mut self, block: &mut [u8]) -> Result<(), super::Error<Self::Error>> {
        let start = Instant::now();

        let token = loop {
            if Instant::now().duration_since(start).as_millis() >= 100 {
                error!("Timeout waiting for token");
                return Err(super::Error::Timeout);
            }

            let mut byte = 0xFFu8;
            self.transfer_in_place(slice::from_mut(&mut byte)).await?;

            if byte == 0xFF {
                continue;
            }

            match Token::try_from(byte) {
                Ok(token) => break token,
                Err(TokenError::NotToken) => continue,
                Err(e) => return Err(super::Error::Transfer(e)),
            }
        };

        if token != Token::Start {
            return Err(super::Error::Generic);
        }

        block.fill(0xFF);
        self.transfer_in_place(block).await?;
        let mut crc = [0xFFu8; 2];
        self.transfer_in_place(&mut crc)
            .await
            .map_err(|err| super::Error::BUS(err))
    }

    async fn read_csd(&mut self) -> Result<CSD, super::Error<Self::Error>> {
        self.send_command(Command::SendCSD(0)).await?;
        let mut buffer = [0u8; 16];
        self.read_block(&mut buffer).await?;
        CSD::try_from(u128::from_be_bytes(buffer)).ok_or(super::Error::Generic)
    }

    async fn write_addr<'b, I>(
        &mut self,
        address: u32,
        blocks: I,
    ) -> Result<(), super::Error<Self::Error>>
    where
        I: core::iter::ExactSizeIterator<Item = &'b [u8; BLOCK_SIZE]>,
    {
        let num_blocks = blocks.len();
        let (cmd, token) = match num_blocks {
            1 => (Command::WriteBlock(address), Token::Start),
            _ => (
                Command::WriteMultipleBlock(address),
                Token::StartWriteMultipleBlock,
            ),
        };
        self.send_command(cmd).await?;

        for block in blocks {
            self.write(&[token as u8]).await?;
            self.write(block).await?;
            let crc = [0u8; 2];
            self.write(&crc).await?;
            let mut byte = 0xFFu8;
            self.transfer_in_place(slice::from_mut(&mut byte)).await?;

            match transfer::Response::try_from(byte) {
                Some(transfer::Response::Accepted) => (),
                Some(_) => return Err(super::Error::Transfer(TokenError::Generic)),
                None => return Err(super::Error::Generic),
            }

            self.wait(Duration::from_millis(250)).await?;
        }

        if num_blocks > 1 {
            self.write(&[Token::Stop as u8, 0xFF]).await?;
            self.wait(Duration::from_millis(250)).await?;
        }

        Ok(())
    }

    async fn send_command(
        &mut self,
        cmd: Command,
    ) -> Result<response::Response, super::Error<Self::Error>> {
        self.write(&[0xFFu8]).await?;
        let bytes: [u8; 6] = cmd.into();
        let s = format!("Send CMD {:?} bytes {:x?}", cmd, &bytes);
        trace!("{}", s.as_str());
        self.write(&bytes[..]).await?;

        if cmd == Command::StopTransmission {
            self.write(&[0xFFu8]).await?;
        }

        // Skip Ncr, 0~8 bytes for SDC, 1~8 bytes for MMC
        let mut r1 = response::R1::default();

        for _ in 0..=8 {
            self.transfer(slice::from_mut(&mut r1.0), &[0xFFu8]).await?;

            if r1.valid() {
                break;
            }
        }

        if !r1.valid() {
            return Err(super::Error::NoResponse);
        }

        if let Some(e) = r1.error() {
            return Err(super::Error::Command(e));
        }

        let mut response = response::Response {
            r1,
            ..Default::default()
        };

        let size = cmd.expected_response_ex_size();

        if size > 0 {
            let mut buffer = [0xFFu8; 4];
            self.transfer_in_place(&mut buffer[4 - size..])
                .await
                .map_err(|e| super::Error::BUS(e))?;
            response.ex = u32::from_be_bytes(buffer);
        }

        Ok(response)
    }

    async fn wait(&mut self, timeout: Duration) -> Result<(), super::Error<Self::Error>> {
        let start = Instant::now();
        let mut byte = 0x00u8;

        while byte != 0xFFu8 {
            if Instant::now().duration_since(start) >= timeout {
                return Err(super::Error::Timeout);
            }

            self.transfer(slice::from_mut(&mut byte), &[0xFFu8]).await?;
        }

        Ok(())
    }
}

pub struct SdCard<'a, M, Braw, B, CS>
where
    M: RawMutex + 'a,
    Braw: SdCardBus + SpiBus + 'a,
    B: LockingSpiBus<'a, M, Braw> + SetFrequency,
    CS: OutputPin,
{
    bus: B,
    cs: CS,
    csd: Option<CSD>,
    card_type: Option<CardType>,
    _null0: PhantomData<&'a ()>,
    _null1: PhantomData<M>,
    _null2: PhantomData<Braw>,
}

type LBA = u32;

impl<T: spi::Error> From<T> for super::Error<T> {
    fn from(err: T) -> Self {
        super::Error::BUS(err)
    }
}

// impl<C: digital::Error, S: spi::Error> From<C> for super::Error<S> {
//     fn from(err: C) -> Self {
//         super::Error::ChipSelect
//     }
// }

impl<'a, M, Braw, B, CS> SdCard<'a, M, Braw, B, CS>
where
    M: RawMutex + 'a,
    Braw: SdCardBus + SpiBus + spi::ErrorType + 'a,
    B: LockingSpiBus<'a, M, Braw> + SetFrequency,
    CS: OutputPin,
{
    // TODO type Error = ... + From<Braw::Error>;

    pub fn new(bus: B, cs: CS) -> Self {
        Self {
            bus,
            cs,
            csd: None,
            card_type: None,
            _null0: PhantomData,
            _null1: PhantomData,
            _null2: PhantomData,
        }
    }

    /// Before init, set SPI clock rate between 100KHZ and 400KHZ
    pub async fn init(&mut self) -> Result<CardType, super::Error<Braw::Error>> {
        let card = {
            // Start at 200 kHz
            debug!("Set low frequency!");
            self.bus.set_frequency(200_000);
            let mut guard = self.bus.lock().await;
            // Supply minimum of 74 clock cycles without CS asserted.
            self.cs
                .set_high()
                .await
                .map_err(|_| super::Error::ChipSelect)?;
            trace!("Supply 74 clock cycles");
            let _ = guard.write(&[0xFF; 10]).await;
            self.cs
                .set_low()
                .await
                .map_err(|_| super::Error::ChipSelect)?;
            trace!("Go idle");
            guard.go_idle().await?;
            trace!("Query version");
            let mut version = 1;
            let r = guard
                .send_command(Command::SendIfCond(SendInterfaceCondition::spi()))
                .await?;

            if !r.r1.has(R1Status::IllegalCommand) {
                version = 2;
                let r7 = response::R7(r.ex);

                if !r7.voltage_accepted() || r7.echo_back_check_pattern() != 0xAA {
                    panic!();
                    // TODO
                    // return Err(Self::Braw::Error::Generic);
                }
            }

            debug!("Version is {}", version);
            debug!("Initialize");
            let mut r1 = response::R1::default();

            for _ in 0..100 {
                r1 = guard
                    .send_app_command(AppCommand::SDSendOpCond(version > 1))
                    .await?
                    .r1;

                if !r1.has(R1Status::InIdleState) {
                    break;
                }
                Timer::after_millis(10).await;
            }

            if r1.has(R1Status::InIdleState) {
                panic!();
                // TODO
                // return Err(Self::Braw::Error::Generic);
            }

            trace!("Read OCR");
            let mut card = CardType::SDSC(version);

            if version > 1 {
                let r = guard.send_app_command(AppCommand::ReadOCR).await?;
                let r3 = response::R3(r.ex);

                if r3.card_capacity_status() {
                    card = CardType::SDHC;
                }
            }

            self.cs
                .set_high()
                .await
                .map_err(|_| super::Error::ChipSelect)?;
            // Extra byte to release MISO
            let _ = guard.write(&[0xFF]).await;
            card
        };

        self.bus.set_frequency(24_000_000);
        self.csd = Some(self.read_csd().await?);
        debug!("CSD: {:?}", self.csd);
        self.card_type = Some(card);

        Ok(card)
    }

    pub async fn read<'b, I>(
        &mut self,
        address: LBA,
        blocks: I,
    ) -> Result<(), super::Error<Braw::Error>>
    where
        I: core::iter::ExactSizeIterator<Item = &'b mut [u8; BLOCK_SIZE]>,
    {
        if blocks.len() == 0 {
            return Ok(());
        }

        let address = if self.card_type().high_capacity() {
            address
        } else {
            address * BLOCK_SIZE as u32
        };

        let mut guard = self.bus.lock().await;
        guard.write(&[0xFF; 5]).await?;
        self.cs
            .set_low()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        guard.read_addr(address, blocks).await?;
        self.cs
            .set_high()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        // Extra byte to release MISO
        let _ = guard.write(&[0xFF]).await;

        Ok(())
    }

    pub async fn read_csd(&mut self) -> Result<CSD, super::Error<Braw::Error>> {
        let mut guard = self.bus.lock().await;
        let _ = guard.write(&[0xFF; 5]).await;
        self.cs
            .set_low()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        let res = guard.read_csd().await;
        self.cs
            .set_high()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        // Extra byte to release MISO
        let _ = guard.write(&[0xFF]).await;

        if let Err(e) = res {
            debug!("fuck this shit");
            return Err(e);
        }

        if let Ok(r) = res {
            debug!("CSD DATA read {:?}", r);
        }
        // NOTE: Cache for later use.
        let csd = res?;
        self.csd = Some(csd);

        Ok(csd)
    }

    pub fn num_blocks(&self) -> NumBlocks {
        self.csd
            .expect("num_blocks called before SdCard was initialized!")
            .num_blocks()
    }

    pub fn card_type(&self) -> CardType {
        self.card_type
            .expect("card_type called before SdCard was initialized!")
    }

    pub async fn write<'b, I>(
        &mut self,
        address: LBA,
        blocks: I,
    ) -> Result<(), super::Error<Braw::Error>>
    where
        I: core::iter::ExactSizeIterator<Item = &'b [u8; BLOCK_SIZE]>,
    {
        if blocks.len() == 0 {
            return Ok(());
        }

        let address = if self.card_type().high_capacity() {
            address
        } else {
            address * BLOCK_SIZE as u32
        };

        let mut guard = self.bus.lock().await;
        let _ = guard.write(&[0xFF; 5]).await;
        self.cs
            .set_low()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        let _ = guard.write_addr(address, blocks).await;
        self.cs
            .set_high()
            .await
            .map_err(|_| super::Error::ChipSelect)?;
        // Extra byte to release MISO
        let _ = guard.write(&[0xFF]).await;

        Ok(())
    }
}

// !!!!!

// #[derive(Debug)]
// pub enum Error<SPI, CS> {
//     SPI(SPI),
//     CS(CS),
// }

// pub type BUSError<SPI, CS> = bus::Error<Error<SPI, CS>>;

// impl<SPI: core::fmt::Debug, CS: core::fmt::Debug> embedded_io_async::Error for Error<SPI, CS> {
//     fn kind(&self) -> embedded_io_async::ErrorKind {
//         match self {
//             Error::SPI(_) => embedded_io_async::ErrorKind::Other,
//             Error::CS(_) => embedded_io_async::ErrorKind::Other,
//         }
//     }
// }

// pub trait ErrorType {
//     /// Error type of all the IO operations on this type.
//     type Error: Error;
// }

// TODO: fix
// impl<SPI, CS, C> ErrorType for Bus<SPI, CS, C>
// where
//     SPI: embedded_hal::spi::ErrorType,
//     CS: embedded_hal::digital::ErrorType,
// {
//     type Error = Error<SPI::Error, CS::Error>;
//     // type Error = SPI::Error;
// }
