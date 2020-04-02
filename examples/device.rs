//! Using a device crate
//!

#![no_main]
#![no_std]

#[allow(unused_extern_crates)]
use panic_halt as _;

use cortex_m::{iprint, peripheral::syst::SystClkSource};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

// `svd2rust` generated Peripheral Access Crate (PAC).
use stm32f4::stm32f401::{interrupt, Interrupt, ITM, NVIC};

#[entry]
fn main() -> ! {
    let p = cortex_m::Peripherals::take().unwrap();

    let mut syst = p.SYST;
    let mut nvic = p.NVIC;

    nvic.enable(Interrupt::EXTI0);

    // configure the system timer to wrap around every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(16_000_000); // 1s
    syst.enable_counter();

    loop {
        // busy wait until the timer wraps around
        while !syst.has_wrapped() {}

        // trigger the `EXTI0` interrupt

        NVIC::pend(Interrupt::EXTI0);
    }
}

// try commenting out this line: you'll end in `default_handler` instead of in `exti0`
#[interrupt]
fn EXTI0() {
    hprintln!(".").unwrap();
    let itm = unsafe { &mut *ITM::ptr() };
    let stim = &mut itm.stim[0];
    iprint!(stim, ".");
}
