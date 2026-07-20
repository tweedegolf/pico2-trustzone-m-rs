// Nightly features necessary for TrustZone-M
#![feature(abi_cmse_nonsecure_call, cmse_nonsecure_entry)]
#![no_std]
#![no_main]

mod nonsecure_entry;

use cortex_m::peripheral::sau::{SauRegion, SauRegionAttribute};
use cortex_m::peripheral::scb::Exception;
use cortex_m::peripheral::SAU;
use cortex_m_rt::exception;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rp235x_hal as hal;

// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

// These symbols come from the `cortex-m-rt`-supplied linker file and are used to set up the SAU.
unsafe extern "C" {
    static __veneer_base: u32;
    static __veneer_limit: u32;
}

// Clock frequency necessary for to set up the timer
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[hal::entry]
fn main() -> ! {
    hprintln!("Hello, Secure World!");

    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut sau = core_peripherals.SAU;

    set_up_sau(&mut sau);

    // Enable SecureFault in System Control Block
    let mut scb = core_peripherals.SCB;
    scb.enable(Exception::SecureFault);

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // Store timer0 in nonsecure_entry module
    unsafe {
        let t = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);
        nonsecure_entry::set_timer(t);
    }

    // Jump to non-secure world
    unsafe {
        let ns_vector = 0x10100000usize; // start of non-secure flash
        let ns_reset: extern "cmse-nonsecure-call" fn() -> ! =
            core::mem::transmute(*((ns_vector + 4) as *const u32));
        cortex_m::register::msp::write_ns(*(ns_vector as *const u32));
        ns_reset();
    }
}

/// Program the security attribution unit (SAU) to enable TrustZone-M functionality.
fn set_up_sau(sau: &mut SAU) {
    // Non-Secure Flash (starting at 256KB offset)
    sau.set_region(
        0,
        SauRegion {
            base_address: 0x10040000,
            limit_address: 0x103FFFFF, // End of 4MB Flash
            attribute: SauRegionAttribute::NonSecure,
        },
    )
    .unwrap();

    // Non-Secure RAM (splitting the 512KB SRAM in half)
    sau.set_region(
        1,
        SauRegion {
            base_address: 0x20040000,
            limit_address: 0x20081FFF,
            attribute: SauRegionAttribute::NonSecure,
        },
    )
    .unwrap();

    // NSC Region (Veneers) - Use linker symbols
    let base_nsc = unsafe { &__veneer_base as *const u32 as u32 };
    let limit_nsc = unsafe { &__veneer_limit as *const u32 as u32 };

    sau.set_region(
        2,
        SauRegion {
            base_address: base_nsc,
            limit_address: limit_nsc | 0x1F,
            attribute: SauRegionAttribute::NonSecureCallable,
        },
    )
    .unwrap();

    // enable the SAU to enforce S<->NS separation
    sau.enable();
}

/// This exception is triggered if the non-secure world tries to access anything it is not permitted to
#[exception]
unsafe fn SecureFault() -> ! {
    hprintln!("SECURE FAULT!");
    loop {
        cortex_m::asm::wfe()
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Blinky Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
