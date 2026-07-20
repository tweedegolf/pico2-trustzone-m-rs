#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use rp235x_hal::{self as hal};

unsafe extern "C" {
    unsafe fn secure_callback(callback: unsafe extern "C" fn());
    unsafe fn make_uppercase(lowercase: *mut u8, len: usize) -> usize;
    unsafe fn nsc_timer(ms: u32);
}

pub extern "C" fn callback_function() {
    hprintln!("Non-Secure World callback function called!");
}

#[hal::entry]
fn main() -> ! {
    hprintln!("Hello, Non-Secure World!");

    unsafe {
        secure_callback(callback_function);
    }

    // let the secure world convert a string owned by the non-secure world into uppercase
    let mut lowercase = *b"share this data 0123456789";
    let len = unsafe { make_uppercase(lowercase.as_mut_ptr(), lowercase.len()) };
    hprintln!(
        "{:?} is {} chars long",
        core::str::from_utf8(&lowercase).unwrap(),
        len
    );

    // Uncomment to generate a SecureFault, because address 0x10001000 is in the secure flash.
    // core::hint::black_box(unsafe { *((0x10001000) as *const u32) });

    loop {
        unsafe {
            nsc_timer(1000);
        }
    }
}
