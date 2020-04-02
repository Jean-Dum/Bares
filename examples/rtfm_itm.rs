#![no_main]
#![no_std]

use cortex_m::{iprint, iprintln};

use pac::Interrupt;
use panic_semihosting as _;
use rtfm::app;
use stm32f4xx_hal::stm32 as pac;

#[app(device = stm32f4xx_hal::stm32, peripherals = true )]
const APP: () = {
    struct Resources {
        itm: cortex_m::peripheral::ITM,
    }
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut itm = cx.core.ITM;
        let stim = &mut itm.stim[0];
        iprintln!(stim, "in init");

        rtfm::pend(Interrupt::EXTI0);
        init::LateResources { itm }
    }

    #[idle (resources = [itm])]
    fn idle(mut cx: idle::Context) -> ! {
        cx.resources.itm.lock(|itm| {
            let stim = &mut itm.stim[0];
            iprintln!(stim, "idle");
        });

        loop {
            continue;
        }
    }

    #[task(binds = EXTI0, resources = [itm])]
    fn exti0(cx: exti0::Context) {
        let stim = &mut cx.resources.itm.stim[0];
        iprintln!(stim, "exti0");
    }
};
