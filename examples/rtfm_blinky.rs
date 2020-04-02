#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use stm32f4xx_hal::stm32;

#[rtfm::app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        GPIOA: stm32::GPIOA,
    }
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut syst = cx.core.SYST;
        let device = cx.device;

        // configures the system timer to trigger a SysTick exception every second
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(16_000_000); // period = 1s
        syst.enable_counter();
        syst.enable_interrupt();

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // pass on late resources
        init::LateResources {
            GPIOA: device.GPIOA,
        }
    }

    #[task(binds = SysTick, resources = [GPIOA])]
    fn toggle(cx: toggle::Context) {
        static mut TOGGLE: bool = false;
        hprintln!("toggle {:?}", TOGGLE).unwrap();

        if *TOGGLE {
            cx.resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            cx.resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }
        *TOGGLE = !*TOGGLE;
    }
};
