// Ref: https://github.com/MichaelBell/micropython/blob/22e04e2e05f1f6f7d07ba255d1381466e0a0c4e6/ports/rp2/rp2_psram.c

use defmt::info;
use embassy_rp::{
    clocks,
    pac::{qmi, IO_BANK0, PADS_BANK0, QMI, XIP_CTRL},
};

#[inline(never)]
#[link_section = ".data"]
pub fn detect() -> usize {
    let mut psram_size: usize = 0;

    // Try and read the PSRAM ID via direct_csr.
    QMI.direct_csr().write(|reg| {
        reg.set_en(true);
        reg.set_clkdiv(30);
    });

    // Need to poll for the cooldown on the last XIP transfer to expire
    // (via direct-mode BUSY flag) before it is safe to perform the first
    // direct-mode operation
    while QMI.direct_csr().read().busy() {}

    // Exit out of QMI in case we've inited already
    QMI.direct_csr().modify(|reg| reg.set_assert_cs1n(true));

    // Transmit as quad.
    QMI.direct_tx().write(|reg| {
        reg.set_oe(true);
        reg.set_iwidth(qmi::vals::Iwidth::Q);
        reg.set_data(0xf5);
    });

    while QMI.direct_csr().read().busy() {}

    QMI.direct_rx().read();

    QMI.direct_csr().modify(|reg| reg.set_assert_cs1n(false));

    // Read the id
    QMI.direct_csr().modify(|reg| reg.set_assert_cs1n(true));

    let mut kgd: u8 = 0;
    let mut eid: u8 = 0;

    for i in 0..7 {
        if i == 0 {
            QMI.direct_tx().write(|reg| {
                reg.set_data(0x9f);
            });
        } else {
            QMI.direct_tx().write(|reg| {
                reg.set_data(0xff);
            });
        }

        while QMI.direct_csr().read().txempty() {}

        while QMI.direct_csr().read().busy() {}

        if i == 5 {
            kgd = QMI.direct_rx().read().0 as u8;
        } else if i == 6 {
            eid = QMI.direct_rx().read().0 as u8;
        } else {
            QMI.direct_rx().read();
        }
    }

    // Disable direct csr.
    QMI.direct_csr().modify(|reg| {
        reg.set_en(false);
        reg.set_assert_cs1n(false);
    });

    info!("kgd={:02x}, eid={:02x}", kgd, eid);

    if kgd == 0x5D {
        psram_size = 1024 * 1024; // 1 MiB
        let size_id = eid >> 5;
        if eid == 0x26 || size_id == 2 {
            psram_size *= 8; // 8 MiB
        } else if size_id == 0 {
            psram_size *= 2; // 2 MiB
        } else if size_id == 1 {
            psram_size *= 4; // 4 MiB
        }
    }

    psram_size
}

#[inline(never)]
#[link_section = ".data"]
pub fn init() -> usize {
    // Set CS1n pin function
    PADS_BANK0.gpio(47).modify(|reg| {
        // reg.set_iso(false);
        reg.set_ie(true);
        reg.set_od(false);
    });

    IO_BANK0.gpio(47).ctrl().write(|reg| reg.set_funcsel(9));

    PADS_BANK0.gpio(47).modify(|reg| {
        reg.set_iso(false);
    });

    // Set PSRAM timing for APS6404
    //
    // Using an rxdelay equal to the divisor isn't enough when running the APS6404 close to 133MHz.
    // So: don't allow running at divisor 1 above 100MHz (because delay of 2 would be too late),
    // and add an extra 1 to the rxdelay if the divided clock is > 100MHz (i.e. sys clock > 200MHz).
    const MAX_PSRAM_FREQ: u32 = 133000000;
    let clock_hz = clocks::clk_sys_freq();

    let mut divisor = (clock_hz + MAX_PSRAM_FREQ - 1) / MAX_PSRAM_FREQ;
    if divisor == 1 && clock_hz > 100000000 {
        divisor = 2;
    }
    let mut rxdelay = divisor;
    if clock_hz / divisor > 100000000 {
        rxdelay += 1;
    }

    // - Max select must be <= 8us.  The value is given in multiples of 64 system clocks.
    // - Min deselect must be >= 18ns.  The value is given in system clock cycles - ceil(divisor / 2).
    let clock_period_fs = 1000000000000000u64 / clock_hz as u64;
    let max_select = (125 * 1000000) / clock_period_fs; // 125 = 8000ns / 64
    let min_deselect = (18 * 1000000 + (clock_period_fs - 1)) / clock_period_fs;

    // max_select = 4;
    // divisor = 2;
    // min_deselect = 3;

    info!("SYS Clock frequency: {}", clock_hz);
    info!("SYS Clock period: {} fs", clock_period_fs);
    info!("Max select: {}, Min deselect: {}", max_select, min_deselect);
    info!("Rx delay: {}, Divisor: {}", rxdelay, divisor);

    let mut size = 0;

    critical_section::with(|_cs| {
        size = detect();

        info!("PSRAM size: {} bytes", size);

        if size == 0 {
            panic!("No PSRAM detected. Skipping initialization.");
        }

        // Enable direct mode, PSRAM CS, clkdiv of 10.
        QMI.direct_csr().write(|reg| {
            reg.set_en(true);
            reg.set_clkdiv(10);
            reg.set_auto_cs1n(true);
        });

        while QMI.direct_csr().read().busy() {}

        // Enable QPI mode on the PSRAM
        QMI.direct_tx().write(|reg| {
            reg.set_nopush(true);
            reg.set_data(0x35);
        });

        while QMI.direct_csr().read().busy() {}

        // Set PSRAM commands and formats
        QMI.mem(1).timing().write(|reg| {
            reg.set_cooldown(1);
            reg.set_pagebreak(qmi::vals::Pagebreak::_1024);
            reg.set_max_select(max_select as u8);
            reg.set_min_deselect(min_deselect as u8);
            reg.set_rxdelay(rxdelay as u8);
            reg.set_clkdiv(divisor as u8);
        });

        // Set PSRAM commands and formats
        QMI.mem(1).rfmt().write(|reg| {
            reg.set_prefix_width(qmi::vals::PrefixWidth::Q);
            reg.set_addr_width(qmi::vals::AddrWidth::Q);
            reg.set_suffix_width(qmi::vals::SuffixWidth::Q);
            reg.set_dummy_width(qmi::vals::DummyWidth::Q);
            reg.set_data_width(qmi::vals::DataWidth::Q);
            reg.set_prefix_len(qmi::vals::PrefixLen::_8);
            reg.set_dummy_len(qmi::vals::DummyLen::_24);
        });

        QMI.mem(1).rcmd().write(|reg| {
            reg.set_prefix(0xeb);
        });

        QMI.mem(1).wfmt().write(|reg| {
            reg.set_prefix_width(qmi::vals::PrefixWidth::Q);
            reg.set_addr_width(qmi::vals::AddrWidth::Q);
            reg.set_suffix_width(qmi::vals::SuffixWidth::Q);
            reg.set_dummy_width(qmi::vals::DummyWidth::Q);
            reg.set_data_width(qmi::vals::DataWidth::Q);
            reg.set_prefix_len(qmi::vals::PrefixLen::_8);
        });

        QMI.mem(1).wcmd().write(|reg| {
            reg.set_prefix(0x38);
        });

        QMI.direct_csr().write(|_reg| {});

        // Enable writes to PSRAM
        XIP_CTRL.ctrl().modify(|reg| reg.set_writable_m1(true));

        info!("PSRAM Initialized!");
    });

    info!("Writing!");

    let reg_addr = 0x1d000000;

    let words = size / 4;

    let mut k: u32 = 0x11111100;

    for i in 0..words {
        unsafe {
            let ptr = (reg_addr as *mut u32).add(i);
            *ptr = k;
            ptr
        };

        // info!("{} -> {:x}", ptr, k);
        k += 1;
        // k += 0x01010101;
    }

    info!("Reading!");

    let mut k = 0x11111100;

    for i in 0..words {
        let data = unsafe { *(reg_addr as *mut u32).add(i) };
        let equal = data == k as u32;

        if !equal {
            panic!("{}: {:x} -> {:x}", i, k, data);
        }
        k += 1;
    }

    info!("Reading again!");

    let mut k = 0x11111100;

    for i in 0..words {
        let data = unsafe { *(reg_addr as *mut u32).add(i) };
        let equal = data == k as u32;

        if !equal {
            panic!("{}: {:x} -> {:x}", i, k, data);
        }
        k += 1;
    }
    info!("Memory tests passed!");

    size
}
