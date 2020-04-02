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
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
    }
    #[init(schedule = [toggle])]
    fn init(cx: init::Context) -> init::LateResources {
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

        // Schedule `toggle` to run 8e6 cycles (clock cycles) in the future
        cx.schedule.toggle(now + 8_000_000.cycles(), true).unwrap();

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
        }
    }

    #[task(resources = [GPIOA], schedule = [toggle])]
    fn toggle(cx: toggle::Context, toggle: bool) {
        hprintln!("foo  @ {:?}", Instant::now()).unwrap();

        if toggle {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        cx.schedule
            .toggle(cx.scheduled + 8_000_000.cycles(), !toggle)
            .unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
