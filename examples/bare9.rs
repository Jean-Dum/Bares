//! bare9.rs
//!
//! Heapless
//!
//! What it covers:
//! - Heapless Ringbuffer
//! - Heapless Producer/Consumer lockfree data access
//! - Interrupt driven I/O
//!

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, iprintln};

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Event, Rx, Serial, Tx};
use hal::stm32::ITM;

use heapless::consts::*;
use heapless::spsc::{Consumer, Producer, Queue};
use nb::block;

use rtfm::app;

#[app(device = hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // Late resources
        TX: Tx<hal::stm32::USART2>,
        RX: Rx<hal::stm32::USART2>,
        PRODUCER: Producer<'static, u8, U3>,
        CONSUMER: Consumer<'static, u8, U3>,
        ITM: ITM,
        // An initialized resource
        #[init(None)]
        RB: Option<Queue<u8, U3>>,
    }
    // init runs in an interrupt free section
    #[init(resources = [RB])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        let device = cx.device;
        // A ring buffer for our data
        *cx.resources.RB = Some(Queue::new());

        // Split into producer/consumer pair
        let (producer, consumer) = cx.resources.RB.as_mut().unwrap().split();

        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "bare9");

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();

        let mut serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // generate interrupt on Rxne
        serial.listen(Event::Rxne);
        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Late resources
        init::LateResources {
            // Our split queue
            PRODUCER: producer,
            CONSUMER: consumer,

            // Our split serial
            TX: tx,
            RX: rx,

            // For debugging
            ITM: core.ITM,
        }
    }

    // idle may be interrupted by other interrupt/tasks in the system
    #[idle(resources = [ITM, CONSUMER])]
    fn idle(cx: idle::Context) -> ! {
        let stim = &mut cx.resources.ITM.stim[0];

        loop {
            while let Some(byte) = cx.resources.CONSUMER.dequeue() {
                iprintln!(stim, "data {}", byte);
            }

            iprintln!(stim, "goto sleep");
            asm::wfi();

            iprintln!(stim, "woken..");
        }
    }

    // task run on USART2 interrupt (set to fire for each byte received)
    #[task(binds = USART2, resources = [RX, TX, PRODUCER])]
    fn usart2(cx: usart2::Context) {
        let rx = cx.resources.RX;
        let tx = cx.resources.TX;

        // at this point we know there must be a byte to read
        match rx.read() {
            Ok(byte) => {
                tx.write(byte).unwrap();

                match cx.resources.PRODUCER.enqueue(byte) {
                    Ok(_) => {}
                    Err(_) => asm::bkpt(),
                }
            }
            Err(_err) => asm::bkpt(),
        }
    }
};

// Optional
// 0. Compile and run the project at 16MHz in release mode
//    make sure its running (not paused).
//
//    > cargo build --example bare9 --features "rtfm" --release
//    (or use the vscode build task)
//
// 1. Start a terminal program, connect with 15200 8N1
//
//    You should now be able to send data and receive an echo from the MCU
//
//    Try sending: "abcd" as a single sequence (set the option No end in moserial),
//    don't send the quotation marks, just abcd.
//
//    What did you receive, and what was the output of the ITM trace.
//
//    ** your answer here **
//
//    Did you experience any over-run errors?
//
//    ** your answer here **
//
//    Why does it behave differently than bare7/bare8?
//
//    ** your answer here **
//
//    Commit your answers (bare9_1)
//
// 2. Compile and run the project at 16MHz in debug mode.
//
//    > cargo build --example bare9 --features "hal rtfm"
//    (or use the vscode build task)
//
//    Try sending: "abcd" as a single sequence (set the option No end in moserial),
//    don't send the quotation marks, just abcd.
//
//    What did you receive, and what was the output of the ITM trace.
//
//    ** your answer here **
//
//    Did you experience any over-run errors?
//
//    ** your answer here **
//
//    Why does it behave differently than in release mode?
//    Recall how the execution overhead changed with optimization level.
//
//    ** your answer here **
//
//    Commit your answers (bare9_2)
//
//    Discussion:
//
//    The concurrency model behind RTFM offers
//    1. Race-free resource access
//
//    2. Deadlock-free execution
//
//    3. Shared execution stack (no pre-allocated stack regions)
//
//    4. Bound priority inversion
//
//    5. Theoretical underpinning ->
//       + (pen and paper) proofs of soundness
//       + schedulability analysis
//       + response time analysis
//       + stack memory analysis
//       + ... leverages on >25 years of research in the real-time community
//         based on the seminal work of Baker in the early 1990s
//         (known as the Stack Resource Policy, SRP)
//
//    Our implementation in Rust offers
//    1. compile check and analysis of tasks and resources
//       + the API implementation together with the Rust compiler will ensure that
//          both RTFM (SRP) soundness and the Rust memory model invariants
//          are upheld (under all circumstances).
//
//    2. arguably the worlds fastest real time scheduler *
//       + task invocation 0-cycle OH on top of HW interrupt handling
//       + 2 cycle OH for locking a shared resource (on lock/claim entry)
//       + 1 cycle OH for releasing a shared resource (on lock/claim exit)
//
//    3. arguably the worlds most memory efficient scheduler *
//       + 1 byte stack memory OH for each (nested) lock/claim
//         (no additional book-keeping during run-time)
//
//       * applies to static task/resource models with single core
//         pre-emptive, static priority scheduling
//
//    In comparison "real-time" schedulers for threaded models (like FreeRTOS)
//       - CPU and memory OH magnitudes larger 
//       - ... and what's worse OH is typically unbound (no proofs of worst case)
//    And additionally threaded models typically imposes
//       - potential race conditions (up to the user to verify)
//       - potential dead-locks (up to the implementation)
//       - potential unbound priority inversion (up to the implementation)
//
//    However, Rust RTFM (currently) target ONLY STATIC SYSTEMS, 
//    there is no notion of dynamically creating new executions contexts/threads
//    so a direct comparison is not completely fair.
//
//    On the other hand, embedded applications are typically static by nature
//    so a STATIC model is to that end better suitable.
//
//    RTFM is reactive by nature, a task execute to end, triggered
//    by an internal or external event, (where an interrupt is an external event
//    from the environment, like a HW peripheral such as the USART2).
//
//    Threads on the other hand are concurrent and infinite by nature and
//    actively blocking/yielding awaiting stischedulers
//    The scheduler then needs to keep track of all threads and at some point choose
//    to dispatch the awaiting thread. So reactivity is bottle-necked to the point
//    of scheduling by queue management, context switching and other additional
//    book keeping.
//
//    In essence, the thread scheduler tries to re-establish the reactivity that
//    were there from the beginning (interrupts), a battle that cannot be won...
