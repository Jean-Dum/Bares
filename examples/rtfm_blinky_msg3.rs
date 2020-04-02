#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use rtfm::cyccnt::{Instant, U32Ext as _};
use stm32f4xx_hal::stm32;

#[rtfm::app(device = stm32f4xx_hal::stm32, monotonic = rtfm::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) {
        let mut core = cx.core;
        let device = cx.device;

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        // semantically, the monotonic timer is frozen at time "zero" during `init`
        // NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
        let now = cx.start; // the start time of the system

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        cx.schedule
            .toggle(now + 8_000_000.cycles(), true, device.GPIOA)
            .ok();
    }

    #[task(schedule= [toggle])]
    fn toggle(cx: toggle::Context, toggle: bool, gpioa: stm32::GPIOA) {
        hprintln!("toggle  @ {:?}", Instant::now()).unwrap();

        if toggle {
            gpioa.bsrr.write(|w| w.bs5().set_bit());
        } else {
            gpioa.bsrr.write(|w| w.br5().set_bit());
        }

        cx.schedule
            .toggle(cx.scheduled + 8_000_000.cycles(), !toggle, gpioa)
            .ok();
    }

    extern "C" {
        fn EXTI0();
    }
};
