//! bare8.rs
//!
//! The RTFM framework
//!
//! What it covers:
//! - utilizing the RTFM framework for serial communication
//! - singletons (entities with a singe instance)
//! - owned resources
//! - peripheral access in RTFM
//! - polling in `idle`

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::iprintln;
use nb::block;

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Rx, Serial, Tx};
use hal::stm32::{ITM, USART2};

use rtfm::app;

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // Late resources
        TX: Tx<USART2>,
        RX: Rx<USART2>,
        ITM: ITM,
    }

    // init runs in an interrupt free section
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        let device = cx.device;

        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "bare8");

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();

        let serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Late resources
        init::LateResources {
            TX: tx,
            RX: rx,
            ITM: core.ITM,
        }
    }

    // idle may be interrupted by other interrupts/tasks in the system
    #[idle(resources = [RX, TX, ITM])]
    fn idle(cx: idle::Context) -> ! {
        let rx = cx.resources.RX;
        let tx = cx.resources.TX;
        let stim = &mut cx.resources.ITM.stim[0];

        loop {
            match block!(rx.read()) {
                Ok(byte) => {
                    iprintln!(stim, "Ok {:?}", byte);
                    tx.write(byte).unwrap();
                }
                Err(err) => {
                    iprintln!(stim, "Error {:?}", err);
                }
            }
        }
    }
};

// Optional assignment
// 0. Compile and run the example. Notice, we use the default 16MHz clock.
//
//    > cargo build --example bare8 --features "rtfm"
//    (or use the vscode build task)
//
// 1. What is the behavior in comparison to bare7.4 and bare7.5
//
//    ** your answer here **
//
//    Commit your answer (bare8_1)
//
// 2. Add a local variable `received` that counts the number of bytes received.
//    Add a local variable `errors` that counts the number of errors.
//
//    Adjust the ITM trace to include the additional information.
//
//    Commit your development (bare8_2)
//
// 3. The added tracing, how did that effect the performance,
//    (are you know loosing more data)?
//
//    ** your answer here **
//
//    Commit your answer (bare8_3)
//
// 4. *Optional
//    Compile and run the program in release mode.
//    If using vscode, look at the `.vscode` folder `task.json` and `launch.json`,
//    you may need to add a new "profile" (a bit of copy paste).
//
//    How did the optimized build compare to the debug build (performance/lost bytes)
//
//    ** your answer here **
//
//    Commit your answer (bare8_4)
