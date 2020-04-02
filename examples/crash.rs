//! Debugging a crash (exception)

// #![deny(unsafe_code)] // this example is using unsafe
#![deny(warnings)]
#![no_main]
#![no_std]

//use panic_halt as _;
use panic_semihosting as _;

use core::ptr;

use cortex_m_rt::{entry, exception};

#[entry]
#[inline(never)]
fn main() -> ! {
    unsafe {
        // read an address outside of the RAM region to cause a HardFault exception
        ptr::read_volatile(0x2FFF_FFFF as *const u32);
    }

    loop {
        continue;
    }
}

#[exception]
#[inline(never)]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("Exception frame {:?}", ef);
}
