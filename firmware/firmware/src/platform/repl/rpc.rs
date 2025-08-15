//! Definitions for the internal Rhai RPC engine to bridge async/sync API functions.

use crate::{
    apps::{
        rx::RxMode,
        tx::{TxMode, TxWords},
    },
    platform::{
        bq25895,
        i2c_io_expander::{
            self,
            model::ExpanderModel,
            models::{pca9536::PCA9536, tcal9539::TCAL9539},
        },
        mc3479::{
            self,
            data::Data,
            registers::{DecMode, LpfBw, ModeState, Range, SampleRate, TempPeriod, Tilt35},
        },
        repl::{
            common::{AckSignal, ControlCommand, ControlSender},
            display::{DisplayCommand, DisplaySender},
            led::LedSender,
            rx::{RxCommand, RxSender},
            tx::{TxCommand, TxSender},
        },
    },
};
use alloc::{borrow::ToOwned, string::String, vec};
use defmt::{debug, warn, Format};
use embassy_embedded_hal::shared_bus::{asynch::i2c::I2cDevice, I2cDeviceError};
use embassy_rp::{
    i2c,
    peripherals::{I2C0, TRNG},
    trng::Trng,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel, watch};
use embedded_graphics::pixelcolor::Rgb565;
use rhai::{Blob, EvalAltResult, INT};
use smart_leds::RGB8;

#[derive(Debug, Clone)]
pub enum RpcError {
    // ErrorSystem(String),
    // ErrorParsing,
    ErrorVariableExists(String),
    ErrorForbiddenVariable(String),
    ErrorVariableNotFound(String),
    ErrorPropertyNotFound(String),
    ErrorIndexNotFound,
    ErrorFunctionNotFound(String),
    ErrorModuleNotFound(String),
    // ErrorInFunctionCall(String, String),
    // ErrorInModule(String),
    ErrorUnboundThis,
    ErrorMismatchDataType(String, String),
    ErrorMismatchOutputType(String, String),
    ErrorIndexingType(String),
    ErrorArrayBounds(usize, INT),
    ErrorStringBounds(usize, INT),
    ErrorBitFieldBounds(usize, INT),
    ErrorFor,
    ErrorDataRace(String),
    ErrorNonPureMethodCallOnConstant(String),
    ErrorAssignmentToConstant(String),
    ErrorDotExpr(String),
    ErrorArithmetic(String),
    ErrorTooManyOperations,
    ErrorTooManyModules,
    ErrorStackOverflow,
    ErrorDataTooLarge(String),
    ErrorTerminated,
    // ErrorCustomSyntax(String, Vec<String>),
    ErrorRuntime,
    // LoopBreak(bool),
    // Return,
    // Exit,
}

impl Format for RpcError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            RpcError::ErrorVariableExists(_) => defmt::write!(f, "ErrorVariableExists"),
            RpcError::ErrorForbiddenVariable(_) => defmt::write!(f, "ErrorForbiddenVariable"),
            RpcError::ErrorVariableNotFound(_) => defmt::write!(f, "ErrorVariableNotFound"),
            RpcError::ErrorPropertyNotFound(_) => defmt::write!(f, "ErrorPropertyNotFound"),
            RpcError::ErrorIndexNotFound => defmt::write!(f, "ErrorIndexNotFound"),
            RpcError::ErrorFunctionNotFound(_) => defmt::write!(f, "ErrorFunctionNotFound"),
            RpcError::ErrorModuleNotFound(_) => defmt::write!(f, "ErrorModuleNotFound"),
            RpcError::ErrorUnboundThis => defmt::write!(f, "ErrorUnboundThis"),
            RpcError::ErrorMismatchDataType(_, _) => defmt::write!(f, "ErrorMismatchDataType"),
            RpcError::ErrorMismatchOutputType(_, _) => defmt::write!(f, "ErrorMismatchOutputType"),
            RpcError::ErrorIndexingType(_) => defmt::write!(f, "ErrorIndexingType"),
            RpcError::ErrorArrayBounds(_, _) => defmt::write!(f, "ErrorArrayBounds"),
            RpcError::ErrorStringBounds(_, _) => defmt::write!(f, "ErrorStringBounds"),
            RpcError::ErrorBitFieldBounds(_, _) => defmt::write!(f, "ErrorBitFieldBounds"),
            RpcError::ErrorFor => defmt::write!(f, "ErrorFor"),
            RpcError::ErrorDataRace(_) => defmt::write!(f, "ErrorDataRace"),
            RpcError::ErrorNonPureMethodCallOnConstant(_) => {
                defmt::write!(f, "ErrorNonPureMethodCallOnConstant")
            }
            RpcError::ErrorAssignmentToConstant(_) => defmt::write!(f, "ErrorAssignmentToConstant"),
            RpcError::ErrorDotExpr(_) => defmt::write!(f, "ErrorDotExpr"),
            RpcError::ErrorArithmetic(_) => defmt::write!(f, "ErrorArithmetic"),
            RpcError::ErrorTooManyOperations => defmt::write!(f, "ErrorTooManyOperations"),
            RpcError::ErrorTooManyModules => defmt::write!(f, "ErrorTooManyModules"),
            RpcError::ErrorStackOverflow => defmt::write!(f, "ErrorStackOverflow"),
            RpcError::ErrorDataTooLarge(_) => defmt::write!(f, "ErrorDataTooLarge"),
            RpcError::ErrorTerminated => defmt::write!(f, "ErrorTerminated"),
            RpcError::ErrorRuntime => defmt::write!(f, "ErrorRuntime"),
        }
    }
}

impl From<EvalAltResult> for RpcError {
    fn from(value: EvalAltResult) -> Self {
        match value {
            EvalAltResult::ErrorVariableExists(s, _) => RpcError::ErrorVariableExists(s),
            EvalAltResult::ErrorForbiddenVariable(s, _) => RpcError::ErrorForbiddenVariable(s),
            EvalAltResult::ErrorVariableNotFound(s, _) => RpcError::ErrorVariableNotFound(s),
            EvalAltResult::ErrorPropertyNotFound(s, _) => RpcError::ErrorPropertyNotFound(s),
            EvalAltResult::ErrorIndexNotFound(_, _) => RpcError::ErrorIndexNotFound,
            EvalAltResult::ErrorFunctionNotFound(s, _) => RpcError::ErrorFunctionNotFound(s),
            EvalAltResult::ErrorModuleNotFound(s, _) => RpcError::ErrorModuleNotFound(s),
            EvalAltResult::ErrorUnboundThis(_) => RpcError::ErrorUnboundThis,
            EvalAltResult::ErrorMismatchDataType(s1, s2, _) => {
                RpcError::ErrorMismatchDataType(s1, s2)
            }
            EvalAltResult::ErrorMismatchOutputType(s1, s2, _) => {
                RpcError::ErrorMismatchOutputType(s1, s2)
            }
            EvalAltResult::ErrorIndexingType(s, _) => RpcError::ErrorIndexingType(s),
            EvalAltResult::ErrorArrayBounds(size, i, _) => RpcError::ErrorArrayBounds(size, i),
            EvalAltResult::ErrorStringBounds(size, i, _) => RpcError::ErrorStringBounds(size, i),
            EvalAltResult::ErrorBitFieldBounds(size, i, _) => {
                RpcError::ErrorBitFieldBounds(size, i)
            }
            EvalAltResult::ErrorFor(_) => RpcError::ErrorFor,
            EvalAltResult::ErrorDataRace(s, _) => RpcError::ErrorDataRace(s),
            EvalAltResult::ErrorNonPureMethodCallOnConstant(s, _) => {
                RpcError::ErrorNonPureMethodCallOnConstant(s)
            }
            EvalAltResult::ErrorAssignmentToConstant(s, _) => {
                RpcError::ErrorAssignmentToConstant(s)
            }
            EvalAltResult::ErrorDotExpr(s, _) => RpcError::ErrorDotExpr(s),
            EvalAltResult::ErrorArithmetic(s, _) => RpcError::ErrorArithmetic(s),
            EvalAltResult::ErrorTooManyOperations(_) => RpcError::ErrorTooManyOperations,
            EvalAltResult::ErrorTooManyModules(_) => RpcError::ErrorTooManyModules,
            EvalAltResult::ErrorStackOverflow(_) => RpcError::ErrorStackOverflow,
            EvalAltResult::ErrorDataTooLarge(s, _) => RpcError::ErrorDataTooLarge(s),
            EvalAltResult::ErrorTerminated(_, _) => RpcError::ErrorTerminated,
            EvalAltResult::ErrorRuntime(_, _) => RpcError::ErrorRuntime,
            _ => RpcError::ErrorRuntime,
        }
    }
}

impl<D> From<I2cDeviceError<D>> for RpcError {
    fn from(_: I2cDeviceError<D>) -> Self {
        // TODO
        RpcError::ErrorRuntime
    }
}

impl RpcError {
    pub fn with_position(self, pos: rhai::Position) -> EvalAltResult {
        match self {
            RpcError::ErrorVariableExists(s) => EvalAltResult::ErrorVariableExists(s, pos),
            RpcError::ErrorForbiddenVariable(s) => EvalAltResult::ErrorForbiddenVariable(s, pos),
            RpcError::ErrorVariableNotFound(s) => EvalAltResult::ErrorVariableNotFound(s, pos),
            RpcError::ErrorPropertyNotFound(s) => EvalAltResult::ErrorPropertyNotFound(s, pos),
            RpcError::ErrorIndexNotFound => EvalAltResult::ErrorIndexNotFound(().into(), pos),
            RpcError::ErrorFunctionNotFound(s) => EvalAltResult::ErrorFunctionNotFound(s, pos),
            RpcError::ErrorModuleNotFound(s) => EvalAltResult::ErrorModuleNotFound(s, pos),
            RpcError::ErrorUnboundThis => EvalAltResult::ErrorUnboundThis(pos),
            RpcError::ErrorMismatchDataType(s1, s2) => {
                EvalAltResult::ErrorMismatchDataType(s1, s2, pos)
            }
            RpcError::ErrorMismatchOutputType(s1, s2) => {
                EvalAltResult::ErrorMismatchOutputType(s1, s2, pos)
            }
            RpcError::ErrorIndexingType(s) => EvalAltResult::ErrorIndexingType(s, pos),
            RpcError::ErrorArrayBounds(size, i) => EvalAltResult::ErrorArrayBounds(size, i, pos),
            RpcError::ErrorStringBounds(size, i) => EvalAltResult::ErrorStringBounds(size, i, pos),
            RpcError::ErrorBitFieldBounds(size, i) => {
                EvalAltResult::ErrorBitFieldBounds(size, i, pos)
            }
            RpcError::ErrorFor => EvalAltResult::ErrorFor(pos),
            RpcError::ErrorDataRace(s) => EvalAltResult::ErrorDataRace(s, pos),
            RpcError::ErrorNonPureMethodCallOnConstant(s) => {
                EvalAltResult::ErrorNonPureMethodCallOnConstant(s, pos)
            }
            RpcError::ErrorAssignmentToConstant(s) => {
                EvalAltResult::ErrorAssignmentToConstant(s, pos)
            }
            RpcError::ErrorDotExpr(s) => EvalAltResult::ErrorDotExpr(s, pos),
            RpcError::ErrorArithmetic(s) => EvalAltResult::ErrorArithmetic(s, pos),
            RpcError::ErrorTooManyOperations => EvalAltResult::ErrorTooManyOperations(pos),
            RpcError::ErrorTooManyModules => EvalAltResult::ErrorTooManyModules(pos),
            RpcError::ErrorStackOverflow => EvalAltResult::ErrorStackOverflow(pos),
            RpcError::ErrorDataTooLarge(s) => EvalAltResult::ErrorDataTooLarge(s, pos),
            RpcError::ErrorTerminated => EvalAltResult::ErrorTerminated(().into(), pos),
            RpcError::ErrorRuntime => EvalAltResult::ErrorRuntime(().into(), pos),
        }
    }
}

pub type CallId = usize;

#[derive(Debug, Clone, Format, PartialEq, Eq)]
pub enum RpcEndpoint {
    SysAssumeControl,
    SysReleaseControl,
    SysRandom,
    InputRead,
    SaoGetDirection,
    SaoSetDirection,
    SaoRead,
    SaoWrite,
    DisplaySetBacklight,
    DisplaySetPixel,
    DisplayFillRegion,
    DisplayClear,
    DisplayFlush,
    DisplayReset,
    ConsoleWrite,
    LedSet,
    AccelRead,
    AccelSetReg8,
    AccelSetReg16,
    AccelSetIntEnable,
    AccelSetMode,
    AccelSetSampleRate,
    AccelSetMotionControl,
    AccelClearInterrupts,
    AccelRangeSelect,
    AccelSetXOffset,
    AccelSetYOffset,
    AccelSetZOffset,
    AccelSetFifoControl,
    AccelSetFifoThreshold,
    AccelSetFifoControl2,
    AccelSetCommControl,
    AccelSetGpioControl,
    AccelSetTiltFlipThreshold,
    AccelSetTiltFlipDebounce,
    AccelSetAnymThreshold,
    AccelSetAnymDebounce,
    AccelSetShakeThreshold,
    AccelSetShakeDuration,
    AccelSetTimerControl,
    AccelSetReadCount,
    BattStatus,
    TrxSetTerm,
    TrxSetTxRxTie,
    TxEnableDisable,
    TxSetBaud,
    TxGetMode,
    TxSetMode,
    TxSend,
    RxEnableDisable,
    RxSetMode,
    RxGetMode,
}

pub trait AppControl {
    fn requires_app_control(&self) -> bool;
}

impl AppControl for RpcEndpoint {
    fn requires_app_control(&self) -> bool {
        match self {
            RpcEndpoint::SysAssumeControl
            | RpcEndpoint::SysRandom
            | RpcEndpoint::InputRead
            | RpcEndpoint::SaoGetDirection
            | RpcEndpoint::SaoRead
            | RpcEndpoint::DisplaySetBacklight
            | RpcEndpoint::AccelRead
            | RpcEndpoint::BattStatus => false,
            _ => true,
        }
    }
}

pub trait ToEndpoint {
    fn to_endpoint(&self) -> RpcEndpoint;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcCall {
    SysAssumeControl,
    SysReleaseControl,
    SysRandom(u8),
    InputRead,
    SaoGetDirection,
    SaoSetDirection(bool, bool),
    SaoRead,
    SaoWrite(bool, bool),
    DisplaySetBacklight(u8),
    DisplaySetPixel(u16, u16, Rgb565),
    DisplayFillRegion(u16, u16, u16, u16, Rgb565),
    DisplayClear,
    DisplayFlush,
    DisplayReset,
    ConsoleWrite(String),
    LedSet(usize, RGB8),
    AccelRead,
    AccelSetReg8(u8, u8),
    AccelSetReg16(u8, u16),
    AccelSetIntEnable(bool, bool, bool, bool, bool, bool, bool),
    AccelSetMode(ModeState, bool, bool),
    AccelSetSampleRate(SampleRate),
    AccelSetMotionControl(bool, bool, bool, bool, bool, bool, bool, bool),
    AccelClearInterrupts,
    AccelRangeSelect(Range, LpfBw),
    AccelSetXOffset(i16),
    AccelSetYOffset(i16),
    AccelSetZOffset(i16),
    AccelSetFifoControl(bool, bool, bool, bool, bool, bool, bool),
    AccelSetFifoThreshold(u8),
    AccelSetFifoControl2(bool, bool, bool, DecMode),
    AccelSetCommControl(bool, bool, bool),
    AccelSetGpioControl(bool, bool, bool, bool),
    AccelSetTiltFlipThreshold(u16),
    AccelSetTiltFlipDebounce(u8),
    AccelSetAnymThreshold(u16),
    AccelSetAnymDebounce(u8),
    AccelSetShakeThreshold(u16),
    AccelSetShakeDuration(u8, u16),
    AccelSetTimerControl(bool, TempPeriod, Tilt35),
    AccelSetReadCount(u8),
    BattStatus,
    TrxSetTerm(bool, bool),
    TrxSetTxRxTie(bool),
    TxEnableDisable(bool),
    TxSetBaud(u32),
    TxGetMode,
    TxSetMode(TxMode),
    TxSend(TxWords),
    RxEnableDisable(bool),
    RxSetMode(RxMode),
    RxGetMode,
}

impl Format for RpcCall {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", defmt::Debug2Format(self))
    }
}

impl ToEndpoint for RpcCall {
    fn to_endpoint(&self) -> RpcEndpoint {
        match self {
            RpcCall::SysAssumeControl => RpcEndpoint::SysAssumeControl,
            RpcCall::SysReleaseControl => RpcEndpoint::SysReleaseControl,
            RpcCall::SysRandom(_) => RpcEndpoint::SysRandom,
            RpcCall::InputRead => RpcEndpoint::InputRead,
            RpcCall::SaoGetDirection => RpcEndpoint::SaoGetDirection,
            RpcCall::SaoSetDirection(_, _) => RpcEndpoint::SaoSetDirection,
            RpcCall::SaoRead => RpcEndpoint::SaoRead,
            RpcCall::SaoWrite(_, _) => RpcEndpoint::SaoWrite,
            RpcCall::DisplaySetBacklight(_) => RpcEndpoint::DisplaySetBacklight,
            RpcCall::DisplaySetPixel(_, _, _) => RpcEndpoint::DisplaySetPixel,
            RpcCall::DisplayFillRegion(_, _, _, _, _) => RpcEndpoint::DisplayFillRegion,
            RpcCall::DisplayClear => RpcEndpoint::DisplayClear,
            RpcCall::DisplayFlush => RpcEndpoint::DisplayFlush,
            RpcCall::DisplayReset => RpcEndpoint::DisplayReset,
            RpcCall::ConsoleWrite(_) => RpcEndpoint::ConsoleWrite,
            RpcCall::LedSet(_, _) => RpcEndpoint::LedSet,
            RpcCall::AccelRead => RpcEndpoint::AccelRead,
            RpcCall::AccelSetReg8(_, _) => RpcEndpoint::AccelSetReg8,
            RpcCall::AccelSetReg16(_, _) => RpcEndpoint::AccelSetReg16,
            RpcCall::AccelSetIntEnable(_, _, _, _, _, _, _) => RpcEndpoint::AccelSetIntEnable,
            RpcCall::AccelSetMode(_, _, _) => RpcEndpoint::AccelSetMode,
            RpcCall::AccelSetSampleRate(_) => RpcEndpoint::AccelSetSampleRate,
            RpcCall::AccelSetMotionControl(_, _, _, _, _, _, _, _) => {
                RpcEndpoint::AccelSetMotionControl
            }
            RpcCall::AccelClearInterrupts => RpcEndpoint::AccelClearInterrupts,
            RpcCall::AccelRangeSelect(_, _) => RpcEndpoint::AccelRangeSelect,
            RpcCall::AccelSetXOffset(_) => RpcEndpoint::AccelSetXOffset,
            RpcCall::AccelSetYOffset(_) => RpcEndpoint::AccelSetYOffset,
            RpcCall::AccelSetZOffset(_) => RpcEndpoint::AccelSetZOffset,
            RpcCall::AccelSetFifoControl(_, _, _, _, _, _, _) => RpcEndpoint::AccelSetFifoControl,
            RpcCall::AccelSetFifoThreshold(_) => RpcEndpoint::AccelSetFifoThreshold,
            RpcCall::AccelSetFifoControl2(_, _, _, _) => RpcEndpoint::AccelSetFifoControl2,
            RpcCall::AccelSetCommControl(_, _, _) => RpcEndpoint::AccelSetCommControl,
            RpcCall::AccelSetGpioControl(_, _, _, _) => RpcEndpoint::AccelSetGpioControl,
            RpcCall::AccelSetTiltFlipThreshold(_) => RpcEndpoint::AccelSetTiltFlipThreshold,
            RpcCall::AccelSetTiltFlipDebounce(_) => RpcEndpoint::AccelSetTiltFlipDebounce,
            RpcCall::AccelSetAnymThreshold(_) => RpcEndpoint::AccelSetAnymThreshold,
            RpcCall::AccelSetAnymDebounce(_) => RpcEndpoint::AccelSetAnymDebounce,
            RpcCall::AccelSetShakeThreshold(_) => RpcEndpoint::AccelSetShakeThreshold,
            RpcCall::AccelSetShakeDuration(_, _) => RpcEndpoint::AccelSetShakeDuration,
            RpcCall::AccelSetTimerControl(_, _, _) => RpcEndpoint::AccelSetTimerControl,
            RpcCall::AccelSetReadCount(_) => RpcEndpoint::AccelSetReadCount,
            RpcCall::BattStatus => RpcEndpoint::BattStatus,
            RpcCall::TrxSetTerm(_, _) => RpcEndpoint::TrxSetTerm,
            RpcCall::TrxSetTxRxTie(_) => RpcEndpoint::TrxSetTxRxTie,
            RpcCall::TxEnableDisable(_) => RpcEndpoint::TxEnableDisable,
            RpcCall::TxSetBaud(_) => RpcEndpoint::TxSetBaud,
            RpcCall::TxGetMode => RpcEndpoint::TxGetMode,
            RpcCall::TxSetMode(_) => RpcEndpoint::TxSetMode,
            RpcCall::TxSend(_) => RpcEndpoint::TxSend,
            RpcCall::RxEnableDisable(_) => RpcEndpoint::RxEnableDisable,
            RpcCall::RxSetMode(_) => RpcEndpoint::RxSetMode,
            RpcCall::RxGetMode => RpcEndpoint::RxGetMode,
        }
    }
}

impl AppControl for RpcCall {
    fn requires_app_control(&self) -> bool {
        self.to_endpoint().requires_app_control()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcResult {
    SysAssumeControl,
    SysReleaseControl,
    SysRandom(Blob),
    InputRead(<TCAL9539 as ExpanderModel>::Inputs),
    SaoGetDirection(bool, bool),
    SaoSetDirection,
    SaoRead(bool, bool),
    SaoWrite,
    DisplaySetBacklight,
    DisplaySetPixel,
    DisplayFillRegion,
    DisplayClear,
    DisplayFlush,
    DisplayReset,
    ConsoleWrite,
    LedSet,
    AccelRead(Data),
    AccelSetReg8,
    AccelSetReg16,
    AccelSetIntEnable,
    AccelSetMode,
    AccelSetSampleRate,
    AccelSetMotionControl,
    AccelClearInterrupts,
    AccelRangeSelect,
    AccelSetXOffset,
    AccelSetYOffset,
    AccelSetZOffset,
    AccelSetFifoControl,
    AccelSetFifoThreshold,
    AccelSetFifoControl2,
    AccelSetCommControl,
    AccelSetGpioControl,
    AccelSetTiltFlipThreshold,
    AccelSetTiltFlipDebounce,
    AccelSetAnymThreshold,
    AccelSetAnymDebounce,
    AccelSetShakeThreshold,
    AccelSetShakeDuration,
    AccelSetTimerControl,
    AccelSetReadCount,
    BattStatus(bq25895::registers::StatusRegisters),
    TrxSetTerm,
    TrxSetTxRxTie,
    TxEnableDisable,
    TxSetBaud,
    TxGetMode(TxMode),
    TxSetMode,
    TxSend,
    RxEnableDisable,
    RxSetMode,
    RxGetMode(RxMode),
}

impl Format for RpcResult {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", defmt::Debug2Format(self))
    }
}

impl ToEndpoint for RpcResult {
    fn to_endpoint(&self) -> RpcEndpoint {
        match self {
            RpcResult::SysAssumeControl => RpcEndpoint::SysAssumeControl,
            RpcResult::SysReleaseControl => RpcEndpoint::SysReleaseControl,
            RpcResult::SysRandom(_) => RpcEndpoint::SysRandom,
            RpcResult::InputRead(_) => RpcEndpoint::InputRead,
            RpcResult::SaoGetDirection(_, _) => RpcEndpoint::SaoGetDirection,
            RpcResult::SaoSetDirection => RpcEndpoint::SaoSetDirection,
            RpcResult::SaoRead(_, _) => RpcEndpoint::SaoRead,
            RpcResult::SaoWrite => RpcEndpoint::SaoWrite,
            RpcResult::DisplaySetBacklight => RpcEndpoint::DisplaySetBacklight,
            RpcResult::DisplaySetPixel => RpcEndpoint::DisplaySetPixel,
            RpcResult::DisplayFillRegion => RpcEndpoint::DisplayFillRegion,
            RpcResult::DisplayClear => RpcEndpoint::DisplayClear,
            RpcResult::DisplayFlush => RpcEndpoint::DisplayFlush,
            RpcResult::DisplayReset => RpcEndpoint::DisplayReset,
            RpcResult::ConsoleWrite => RpcEndpoint::ConsoleWrite,
            RpcResult::LedSet => RpcEndpoint::LedSet,
            RpcResult::AccelRead(_) => RpcEndpoint::AccelRead,
            RpcResult::AccelSetReg8 => RpcEndpoint::AccelSetReg8,
            RpcResult::AccelSetReg16 => RpcEndpoint::AccelSetReg16,
            RpcResult::AccelSetIntEnable => RpcEndpoint::AccelSetIntEnable,
            RpcResult::AccelSetMode => RpcEndpoint::AccelSetMode,
            RpcResult::AccelSetSampleRate => RpcEndpoint::AccelSetSampleRate,
            RpcResult::AccelSetMotionControl => RpcEndpoint::AccelSetMotionControl,
            RpcResult::AccelClearInterrupts => RpcEndpoint::AccelClearInterrupts,
            RpcResult::AccelRangeSelect => RpcEndpoint::AccelRangeSelect,
            RpcResult::AccelSetXOffset => RpcEndpoint::AccelSetXOffset,
            RpcResult::AccelSetYOffset => RpcEndpoint::AccelSetYOffset,
            RpcResult::AccelSetZOffset => RpcEndpoint::AccelSetZOffset,
            RpcResult::AccelSetFifoControl => RpcEndpoint::AccelSetFifoControl,
            RpcResult::AccelSetFifoThreshold => RpcEndpoint::AccelSetFifoThreshold,
            RpcResult::AccelSetFifoControl2 => RpcEndpoint::AccelSetFifoControl2,
            RpcResult::AccelSetCommControl => RpcEndpoint::AccelSetCommControl,
            RpcResult::AccelSetGpioControl => RpcEndpoint::AccelSetGpioControl,
            RpcResult::AccelSetTiltFlipThreshold => RpcEndpoint::AccelSetTiltFlipThreshold,
            RpcResult::AccelSetTiltFlipDebounce => RpcEndpoint::AccelSetTiltFlipDebounce,
            RpcResult::AccelSetAnymThreshold => RpcEndpoint::AccelSetAnymThreshold,
            RpcResult::AccelSetAnymDebounce => RpcEndpoint::AccelSetAnymDebounce,
            RpcResult::AccelSetShakeThreshold => RpcEndpoint::AccelSetShakeThreshold,
            RpcResult::AccelSetShakeDuration => RpcEndpoint::AccelSetShakeDuration,
            RpcResult::AccelSetTimerControl => RpcEndpoint::AccelSetTimerControl,
            RpcResult::AccelSetReadCount => RpcEndpoint::AccelSetReadCount,
            RpcResult::BattStatus(_) => RpcEndpoint::BattStatus,
            RpcResult::TrxSetTerm => RpcEndpoint::TrxSetTerm,
            RpcResult::TrxSetTxRxTie => RpcEndpoint::TrxSetTxRxTie,
            RpcResult::TxEnableDisable => RpcEndpoint::TxEnableDisable,
            RpcResult::TxSetBaud => RpcEndpoint::TxSetBaud,
            RpcResult::TxGetMode(_) => RpcEndpoint::TxGetMode,
            RpcResult::TxSetMode => RpcEndpoint::TxSetMode,
            RpcResult::TxSend => RpcEndpoint::TxSend,
            RpcResult::RxEnableDisable => RpcEndpoint::RxEnableDisable,
            RpcResult::RxSetMode => RpcEndpoint::RxSetMode,
            RpcResult::RxGetMode(_) => RpcEndpoint::RxGetMode,
        }
    }
}

impl AppControl for RpcResult {
    fn requires_app_control(&self) -> bool {
        self.to_endpoint().requires_app_control()
    }
}

pub const RPC_MTU: usize = 1;

pub type RpcCallChannel = channel::Channel<CriticalSectionRawMutex, RpcCall, RPC_MTU>;
pub type RpcCallSender = channel::Sender<'static, CriticalSectionRawMutex, RpcCall, RPC_MTU>;
pub type RpcCallReceiver = channel::Receiver<'static, CriticalSectionRawMutex, RpcCall, RPC_MTU>;

pub type RpcResultChannel =
    channel::Channel<CriticalSectionRawMutex, (CallId, Result<RpcResult, RpcError>), RPC_MTU>;
pub type RpcResultSender = channel::Sender<
    'static,
    CriticalSectionRawMutex,
    (CallId, Result<RpcResult, RpcError>),
    RPC_MTU,
>;
pub type RpcResultReceiver = channel::Receiver<
    'static,
    CriticalSectionRawMutex,
    (CallId, Result<RpcResult, RpcError>),
    RPC_MTU,
>;

// TODO: This const increments with every variant of `AppAck`
pub const APP_ACK_MTU: usize = 2;

pub type AppContextWatch = watch::Watch<CriticalSectionRawMutex, bool, APP_ACK_MTU>;
pub type AppContextSender = watch::Sender<'static, CriticalSectionRawMutex, bool, APP_ACK_MTU>;
pub type AppContextReceiver = watch::Receiver<'static, CriticalSectionRawMutex, bool, APP_ACK_MTU>;

/// Enum denoting which application subsystem is acknowledging the context switch to user-mode.
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum AppAck {
    NeoPixel = 0x00,
    Display = 0x01,
}

impl From<AppAck> for usize {
    fn from(value: AppAck) -> usize {
        value as usize
    }
}

pub type AppAckChannel = channel::Channel<CriticalSectionRawMutex, AppAck, APP_ACK_MTU>;
pub type AppAckSender = channel::Sender<'static, CriticalSectionRawMutex, AppAck, APP_ACK_MTU>;
pub type AppAckReceiver = channel::Receiver<'static, CriticalSectionRawMutex, AppAck, APP_ACK_MTU>;

#[macro_export]
macro_rules! make_rpc_channels {
    () => {{
        use crate::platform::repl::rpc::{RpcCallChannel, RpcResultChannel};
        use embassy_sync::lazy_lock::LazyLock;

        static CALL_CHANNEL: LazyLock<RpcCallChannel> = LazyLock::new(|| RpcCallChannel::new());
        static RESULT_CHANNEL: LazyLock<RpcResultChannel> =
            LazyLock::new(|| RpcResultChannel::new());

        (CALL_CHANNEL.get(), RESULT_CHANNEL.get())
    }};
}

#[macro_export]
macro_rules! make_app_channels {
    () => {{
        use crate::platform::repl::rpc::{AppAckChannel, AppContextWatch};
        use embassy_sync::lazy_lock::LazyLock;

        static APP_WATCH: LazyLock<AppContextWatch> =
            LazyLock::new(|| AppContextWatch::new_with(false));
        static APP_ACK_CHANNEL: LazyLock<AppAckChannel> = LazyLock::new(|| AppAckChannel::new());

        (APP_WATCH.get(), APP_ACK_CHANNEL.get())
    }};
}

// TODO: Better input sanitization for commands.
#[embassy_executor::task]
pub async fn repl_rpc(
    app_tx: AppContextSender,
    app_ack_rx: AppAckReceiver,
    call_rx: RpcCallReceiver,
    result_tx: RpcResultSender,
    display_tx: DisplaySender,
    led_tx: LedSender,
    tx_tx: TxSender,
    tx_ack: &'static AckSignal,
    rx_tx: RxSender,
    rx_ack: &'static AckSignal,
    ctrl_tx: ControlSender,
    ctrl_ack: &'static AckSignal,
    mut accel_ctrl: mc3479::control::Control<
        CriticalSectionRawMutex,
        i2c::I2c<'static, I2C0, i2c::Async>,
    >,
    batt_ctrl: bq25895::control::Control<
        CriticalSectionRawMutex,
        i2c::I2c<'static, I2C0, i2c::Async>,
    >,
    mut gpio_1_ctrl: i2c_io_expander::Control<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
        16,
    >,
    _gpio_2_ctrl: i2c_io_expander::Control<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
        4,
    >,
    mut trng: Trng<'static, TRNG>,
) -> ! {
    debug!("Initializing REPL RPC task!");
    let mut call_count: CallId = 0;

    loop {
        debug!(
            "HEAP: USED {:?} FREE {:?} CALL_COUNT: {}",
            crate::HEAP.used(),
            crate::HEAP.free(),
            call_count
        );
        let call = call_rx.receive().await;
        call_count += 1;
        warn!("Got RPC Call {:?}!", call);
        debug!(
            "HEAP: USED {:?} FREE {:?} CALL_COUNT: {}",
            crate::HEAP.used(),
            crate::HEAP.free(),
            call_count
        );

        // TODO: Maybe change error types to send a static &str instead of a String.
        if let Some(user_control) = app_tx.try_get() {
            if !user_control && call.requires_app_control() {
                let err =
                    RpcError::ErrorDataRace("Function requires control of app context!".to_owned());
                let result = Err(err);
                result_tx.send((call_count, result)).await;
                continue;
            }
        }

        let mut switching_contexts = false;

        match call {
            RpcCall::SysAssumeControl => {
                let outcome = {
                    match app_tx.try_get() {
                        Some(true) => Err(RpcError::ErrorDataRace(
                            "App context is already owned!".to_owned(),
                        )),
                        Some(false) | None => {
                            app_ack_rx.clear();
                            app_tx.send(true);
                            switching_contexts = true;

                            Ok(RpcResult::SysAssumeControl)
                        }
                    }
                };
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SysReleaseControl => {
                let outcome = {
                    match app_tx.try_get() {
                        Some(true) => {
                            app_ack_rx.clear();
                            app_tx.send(false);
                            switching_contexts = true;

                            Ok(RpcResult::SysReleaseControl)
                        }
                        Some(false) => {
                            warn!("App context is already released!");
                            Err(RpcError::ErrorDataRace(
                                "App context is already released!".to_owned(),
                            ))
                        }
                        None => {
                            warn!("App context uninitialized!");
                            Err(RpcError::ErrorDataRace(
                                "App context uninitialized!".to_owned(),
                            ))
                        }
                    }
                };
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SysRandom(bytes) => {
                let mut values = vec![0u8; bytes as usize];
                trng.fill_bytes(values.as_mut_slice()).await;
                let outcome = Ok(RpcResult::SysRandom(values));
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::InputRead => {
                let inputs = gpio_1_ctrl.read_inputs().await;
                let outcome = Ok(RpcResult::InputRead(inputs));
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SaoGetDirection => {
                ctrl_tx.send(ControlCommand::GetSaoGpioDir).await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SaoSetDirection(dir_1, dir_2) => {
                ctrl_tx
                    .send(ControlCommand::SetSaoGpioDir(dir_1, dir_2))
                    .await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SaoRead => {
                ctrl_tx.send(ControlCommand::ReadSaoGpio).await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::SaoWrite(output_1, output_2) => {
                ctrl_tx
                    .send(ControlCommand::WriteSaoGpio(output_1, output_2))
                    .await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplaySetBacklight(percent) => {
                ctrl_tx
                    .send(ControlCommand::SetDisplayBacklight(percent))
                    .await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplaySetPixel(x, y, color) => {
                display_tx.send(DisplayCommand::SetPixel(x, y, color)).await;
                let outcome = Ok(RpcResult::DisplaySetPixel);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplayFillRegion(sx, ex, sy, ey, color) => {
                display_tx
                    .send(DisplayCommand::FillRegion(sx, ex, sy, ey, color))
                    .await;
                let outcome = Ok(RpcResult::DisplayFillRegion);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplayClear => {
                display_tx.send(DisplayCommand::Clear).await;
                let outcome = Ok(RpcResult::DisplayClear);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplayFlush => {
                display_tx.send(DisplayCommand::Flush).await;
                let outcome = Ok(RpcResult::DisplayFlush);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::DisplayReset => {
                ctrl_tx.send(ControlCommand::ResetDisplay).await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::ConsoleWrite(text) => {
                display_tx.send(DisplayCommand::ConsoleWrite(text)).await;
                let outcome = Ok(RpcResult::ConsoleWrite);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::LedSet(i, color) => {
                led_tx.send((i, color)).await;
                let outcome = Ok(RpcResult::LedSet);
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelRead => {
                let data = accel_ctrl.read().await;
                let outcome = Ok(RpcResult::AccelRead(data));
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetReg8(reg, value) => {
                let outcome = accel_ctrl
                    .set_register_8(reg, value)
                    .await
                    .map(|_| RpcResult::AccelSetReg8)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetReg16(reg, value) => {
                let outcome = accel_ctrl
                    .set_register_16(reg, value)
                    .await
                    .map(|_| RpcResult::AccelSetReg16)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetIntEnable(tilt, flip, anym, shake, tilt35, auto_clr, acq) => {
                let outcome = accel_ctrl
                    .set_interrupt_enable(tilt, flip, anym, shake, tilt35, auto_clr, acq)
                    .await
                    .map(|_| RpcResult::AccelSetIntEnable)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetMode(mode, i2c_wdt_neg, i2c_wdt_pos) => {
                let outcome = accel_ctrl
                    .set_mode(mode, i2c_wdt_neg, i2c_wdt_pos)
                    .await
                    .map(|_| RpcResult::AccelSetMode)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetSampleRate(rate) => {
                let outcome = accel_ctrl
                    .set_sample_rate(rate)
                    .await
                    .map(|_| RpcResult::AccelSetSampleRate)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetMotionControl(
                reset,
                raw_proc_stat,
                z_axis_ort,
                tilt35_en,
                shake_en,
                anym,
                motion_latch,
                tiltflip,
            ) => {
                let outcome = accel_ctrl
                    .set_motion_control(
                        reset,
                        raw_proc_stat,
                        z_axis_ort,
                        tilt35_en,
                        shake_en,
                        anym,
                        motion_latch,
                        tiltflip,
                    )
                    .await
                    .map(|_| RpcResult::AccelSetMotionControl)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelClearInterrupts => {
                let outcome = accel_ctrl
                    .clear_interrupts()
                    .await
                    .map(|_| RpcResult::AccelClearInterrupts)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelRangeSelect(range, lpf_bw) => {
                let outcome = accel_ctrl
                    .range_select(range, lpf_bw)
                    .await
                    .map(|_| RpcResult::AccelRangeSelect)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetXOffset(offset) => {
                let outcome = accel_ctrl
                    .set_x_offset(offset)
                    .await
                    .map(|_| RpcResult::AccelSetXOffset)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetYOffset(offset) => {
                let outcome = accel_ctrl
                    .set_y_offset(offset)
                    .await
                    .map(|_| RpcResult::AccelSetYOffset)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetZOffset(offset) => {
                let outcome = accel_ctrl
                    .set_z_offset(offset)
                    .await
                    .map(|_| RpcResult::AccelSetZOffset)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetFifoControl(
                mode,
                enable,
                reset,
                comb_int,
                th_int,
                full_int,
                empty_int,
            ) => {
                let outcome = accel_ctrl
                    .set_fifo_control(mode, enable, reset, comb_int, th_int, full_int, empty_int)
                    .await
                    .map(|_| RpcResult::AccelSetFifoControl)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetFifoThreshold(threshold) => {
                let outcome = accel_ctrl
                    .set_fifo_threshold(threshold)
                    .await
                    .map(|_| RpcResult::AccelSetFifoThreshold)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetFifoControl2(burst, wrap_addr, wrap_en, dec_mode) => {
                let outcome = accel_ctrl
                    .set_fifo_control2(burst, wrap_addr, wrap_en, dec_mode)
                    .await
                    .map(|_| RpcResult::AccelSetFifoControl2)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetCommControl(indiv_int_clr, spi_3wire_en, int1_int2_req_swap) => {
                let outcome = accel_ctrl
                    .set_comm_control(indiv_int_clr, spi_3wire_en, int1_int2_req_swap)
                    .await
                    .map(|_| RpcResult::AccelSetCommControl)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetGpioControl(
                gpio2_intn2_ipp,
                gpio2_intn2_iah,
                gpio1_intn1_ipp,
                gpio1_intn1_iah,
            ) => {
                let outcome = accel_ctrl
                    .set_gpio_control(
                        gpio2_intn2_ipp,
                        gpio2_intn2_iah,
                        gpio1_intn1_ipp,
                        gpio1_intn1_iah,
                    )
                    .await
                    .map(|_| RpcResult::AccelSetGpioControl)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetTiltFlipThreshold(threshold) => {
                let outcome = accel_ctrl
                    .set_tilt_flip_threshold(threshold)
                    .await
                    .map(|_| RpcResult::AccelSetTiltFlipThreshold)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetTiltFlipDebounce(debounce) => {
                let outcome = accel_ctrl
                    .set_tilt_flip_debounce(debounce)
                    .await
                    .map(|_| RpcResult::AccelSetTiltFlipDebounce)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetAnymThreshold(threshold) => {
                let outcome = accel_ctrl
                    .set_anym_threshold(threshold)
                    .await
                    .map(|_| RpcResult::AccelSetAnymThreshold)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetAnymDebounce(debounce) => {
                let outcome = accel_ctrl
                    .set_anym_debounce(debounce)
                    .await
                    .map(|_| RpcResult::AccelSetAnymDebounce)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetShakeThreshold(threshold) => {
                let outcome = accel_ctrl
                    .set_shake_threshold(threshold)
                    .await
                    .map(|_| RpcResult::AccelSetShakeThreshold)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetShakeDuration(cnt, p2p) => {
                let outcome = accel_ctrl
                    .set_shake_duration(cnt, p2p)
                    .await
                    .map(|_| RpcResult::AccelSetShakeDuration)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetTimerControl(per_int_en, period, tilt35) => {
                let outcome = accel_ctrl
                    .set_timer_control(per_int_en, period, tilt35)
                    .await
                    .map(|_| RpcResult::AccelSetTimerControl)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::AccelSetReadCount(count) => {
                let outcome = accel_ctrl
                    .set_read_count(count)
                    .await
                    .map(|_| RpcResult::AccelSetReadCount)
                    .map_err(|err| err.into());
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::BattStatus => {
                let outcome = Ok(RpcResult::BattStatus(batt_ctrl.status().await));
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TrxSetTerm(sel_0, sel_1) => {
                ctrl_tx.send(ControlCommand::SetTerm(sel_0, sel_1)).await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TrxSetTxRxTie(enabled) => {
                ctrl_tx.send(ControlCommand::SetTxRxTie(enabled)).await;
                let outcome = ctrl_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TxEnableDisable(enabled) => {
                tx_tx.send(TxCommand::EnableDisable(enabled)).await;
                let outcome = tx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TxSetBaud(baud) => {
                tx_tx.send(TxCommand::SetBaud(baud)).await;
                let outcome = tx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TxGetMode => {
                tx_tx.send(TxCommand::GetMode).await;
                let outcome = tx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TxSetMode(mode) => {
                tx_tx.send(TxCommand::SetMode(mode)).await;
                let outcome = tx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::TxSend(words) => {
                tx_tx.send(TxCommand::Send(words)).await;
                let outcome = tx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::RxEnableDisable(enabled) => {
                rx_tx.send(RxCommand::EnableDisable(enabled)).await;
                let outcome = rx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::RxGetMode => {
                rx_tx.send(RxCommand::GetMode).await;
                let outcome = rx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
            RpcCall::RxSetMode(mode) => {
                rx_tx.send(RxCommand::SetMode(mode)).await;
                let outcome = rx_ack.wait().await;
                let result = (call_count, outcome);
                result_tx.send(result).await;
            }
        }

        debug!(
            "HEAP: USED {:?} FREE {:?} CALL_COUNT: {}",
            crate::HEAP.used(),
            crate::HEAP.free(),
            call_count
        );

        // NOTE: We have to do this *after* sending a Result, lest we deadlock.
        if switching_contexts {
            // Wait for all subsystems to acknowledge receipt.
            let mut acks: [Option<AppAck>; APP_ACK_MTU] = [None; APP_ACK_MTU];
            let mut ack_cnt = 0_usize;

            while ack_cnt < APP_ACK_MTU {
                let ack = app_ack_rx.receive().await;
                acks[ack as usize] = Some(ack);
                ack_cnt += 1;
                debug!("Got an app ACK: {} [{}]: {}", ack, ack_cnt, acks);
            }
        }

        debug!(
            "HEAP: USED {:?} FREE {:?} CALL_COUNT: {}",
            crate::HEAP.used(),
            crate::HEAP.free(),
            call_count
        );
    }
}
