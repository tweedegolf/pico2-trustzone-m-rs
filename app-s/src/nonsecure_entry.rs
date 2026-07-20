use cortex_m::cmse::{AccessType, TestTarget};
use cortex_m_semihosting::hprintln;
use embedded_hal::delay::DelayNs;
use rp235x_hal::timer::CopyableTimer0;
use rp235x_hal::Timer;

static mut SECURE_TIMER: Option<Timer<CopyableTimer0>> = None;

/// Stores the timer singleton that the veneer will use.
pub(crate) unsafe fn set_timer(t: Timer<CopyableTimer0>) {
    unsafe { SECURE_TIMER = Some(t) };
}

/// Non-secure-callable delay routine
#[unsafe(no_mangle)]
pub extern "cmse-nonsecure-entry" fn nsc_timer(ms: u32) {
    hprintln!("Secure timer called by the non-secure world!");

    // SAFETY: we only access the static timer from this one NSC function.
    unsafe {
        if let Some(ref mut timer) = SECURE_TIMER {
            timer.delay_ms(ms)
        }
    }
}

/// Non-secure-callable callback function
#[unsafe(no_mangle)]
pub extern "cmse-nonsecure-entry" fn secure_callback(callback: unsafe extern "C" fn()) {
    let callback: unsafe extern "cmse-nonsecure-call" fn() =
        unsafe { core::mem::transmute(callback) };
    unsafe {
        callback();
    }
}

/// Non-secure-callable function to convert a string into uppercase
#[unsafe(no_mangle)]
pub extern "cmse-nonsecure-entry" fn make_uppercase(lowercase: *mut u8, len: usize) -> usize {
    // TODO: lock the data (important for nonsecure interrupts)
    let test = TestTarget::check_range(lowercase as *mut u32, len, AccessType::NonSecure);

    // test if the non-secure world which calls this function has full ownership of the shared data
    match test {
        None => return 0,
        Some(test) => {
            if !test.ns_read_and_writable() {
                return 0;
            }
        }
    }

    // SAFETY: We need to ensure the pointer is valid and the length is correct
    let slice = unsafe { core::slice::from_raw_parts_mut(lowercase, len) };

    // TODO: *lowercase.make_ascii_uppercase();
    for ch in slice.iter_mut() {
        if *ch >= b'a' && *ch <= b'z' {
            *ch -= 0x20;
        }
    }

    len
}
