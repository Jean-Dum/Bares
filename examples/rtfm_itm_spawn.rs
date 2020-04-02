#![no_main]
#![no_std]

use cortex_m::iprintln;

use panic_semihosting as _;
use rtfm::app;

#[app(device = stm32f4xx_hal::stm32, peripherals = true )]
const APP: () = {
    struct Resources {
        itm: cortex_m::peripheral::ITM,
    }
    #[init(spawn = [task1])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut itm = cx.core.ITM;
        let stim = &mut itm.stim[0];
        iprintln!(stim, "in init");

        cx.spawn.task1("from init").unwrap();
        init::LateResources { itm }
    }

    #[idle (resources = [itm], spawn = [task1])]
    fn idle(cx: idle::Context) -> ! {
        let (mut itm, spawn) = (cx.resources.itm, cx.spawn);
        itm.lock(|itm| {
            let stim = &mut itm.stim[0];
            spawn.task1("from idle, itm locked").unwrap();
            iprintln!(stim, "idle");
        });
        cx.spawn.task1("from idle").unwrap();

        loop {
            continue;
        }
    }

    #[task (resources = [itm])]
    fn task1(cx: task1::Context, called_from: &'static str) {
        let stim = &mut cx.resources.itm.stim[0];
        iprintln!(stim, "task1 {}", called_from);
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI0();
    }
};
