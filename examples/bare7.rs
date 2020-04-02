//! bare7.rs
//!
//! Serial echo
//!
//! What it covers:
//! - changing the clock using Rust code
//! - working with the svd2rust API
//! - working with the HAL (Hardware Abstraction Layer)
//! - USART polling (blocking wait)

#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::iprintln;
use cortex_m_rt::entry;
use nb::block;

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Serial};

#[entry]
fn main() -> ! {
    let mut c = hal::stm32::CorePeripherals::take().unwrap();
    let stim = &mut c.ITM.stim[0];

    let p = hal::stm32::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();

    // 16 MHz (default, all clocks)
    let clocks = rcc.cfgr.freeze();
    // 84 MHz (with valid config for pclk1 and pclk2)
    // let clocks = rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(84.mhz()).freeze();

    let gpioa = p.GPIOA.split();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7(); // try comment out
                                             
    // let rx = gpioa.pa3.into_alternate_af6(); // try uncomment

    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        Config::default().baudrate(115_200.bps()),
        clocks,
    )
    .unwrap();
    iprintln!(stim, "bare7");

    // Separate out the sender and receiver of the serial port
    let (mut tx, mut rx) = serial.split();

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

// Optional assignment
// 0. Background reading:
//    STM32F401xD STM32F401xE, section 3.11
//    We have two AMBA High-performance Bus (AHB)
//    APB1 low speed bus (max freq 42 MHz)
//    APB2 high speed bus (max frex 84 MHz)
//
//    RM0368 Section 6.2
//    Some important/useful clock acronyms and their use:
//
//    SYSCLK - the clock that drives the `core`
//    HCLK   - the clock that drives the AMBA bus(es), memory, DMA, trace unit, etc.
//
//    Typically we set HCLK = SYSCLK / 1 (no prescale) for our applications
//
//    FCLK   - Free running clock runing at HCLK
//
//    CST    - CoreSystemTimer drives the SysTick counter, HCLK/(1 or 8)
//    PCLK1  - The clock driving the APB1 (<= 42 MHz)
//             Timers on the APB1 bus will be triggered at PCLK1 * 2
//    PCLK2  - The clock driving the APB2 (<= 84 MHz)
//             Timers on the APB2 bus will be triggered at PCLK2
//
//    Compliation:
//    > cargo build --example bare7 --features "stm32fxx-hal"
//    (or use the vscode build task)
//
//
//    Cargo.toml:
//
//    [dependencies.stm32f4]
//    version = "0.5.0"
//    features = ["stm32f413", "rt"]
//    optional = true
//
//    [dependencies.stm32f4xx-hal]
//    version         = "0.6.0"
//    features        = ["stm32f401", "rt"]
//    optional        = true
//
//    Notice, stm32f4xx-hal internally enables the dependency to stm32f4,
//    so we don't need to explicitly enable it.
//
//    The HAL provides a generic abstraction over the whole stm32f4 family.
//
// 1. The rcc.cfgr.x.freeze() sets the clock according to the configuration x given.
//
//    rcc.cfgr.freeze(); sets a default configuration.
//    sysclk = hclk = pclk1 = pclk2 = 16MHz
//
//    What is wrong with the following configurations?
//
//    rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze();
//
//    ** your answer here **
//
//    rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();
//
//    ** your answer here **
//
//    Commit your answers (bare7_1)
//
//    Tip: You may use `stm32cubemx` to get a graphical view for experimentation.
//
// 2. Now give the system with a valid clock, sysclk of 84 MHz.
//
//    Include the code for outputting the clock to MCO2.
//
//    Repeat the experiment bare6_2.
//
//    What is the frequency of MCO2 read by the oscilloscope.
//
//    ** your answer here **
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    ** your answer here **
//
//    What is the peak to peak reading of the signal.
//
//    ** your answer here **
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_7_84mhz_high_speed"
//
//    Commit your answers (bare7_2)
//
// 3. Now reprogram the PC9 to be "Low Speed", and re-run at 84Mz.
//
//    Did the frequency change in comparison to assignment 6?
//
//    ** your answer here **
//
//    What is the peak to peak reading of the signal (and why did it change)?
//
//    ** your answer here **
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_7_84mhz_low_speed".
//
//    Commit your answers (bare7_3)
//
// 4. Revisit the `README.md` regarding serial communication.
//    start a terminal program, e.g., `moserial`.
//    Connect to the port
//
//    Device       /dev/ttyACM0
//    Baude Rate   115200
//    Data Bits    8
//    Stop Bits    1
//    Parity       None
//
//    This setting is typically abbreviated as 115200 8N1.
//
//    Run the example, make sure your ITM is set to 84MHz.
//
//    Send a single character (byte), (set the option No end in moserial).
//    Verify that sent bytes are echoed back, and that ITM tracing is working.
//
//    If not go back check your ITM setting, clocks etc.
//
//    Try sending: "abcd" as a single sequence, don't send the quotation marks, just abcd.
//
//    What did you receive, and what was the output of the ITM trace.
//
//    ** your answer here **
//
//    commit your answers (bare7_4)
//
// 5. Now, set the CPU to run at 16MHz.
//    Repeat the experiment 7.4 (make sure your ITM is set to 16MHz)
//
//    Try sending: "abcd" as a single sequence, don't send the quotation marks, just abcd.
//
//    What did you receive, and what was the output of the ITM trace.
//
//    ** your answer here **
//
//    Explain why the buffer overflows.
//
//    ** your answer here **
//
//    commit your answers (bare7_4)
//
//    Discussion:
//    Common to all MCUs is that they have multiple clocking options.
//    Understanding the possibilities and limitations of clocking is fundamental
//    to designing both the embedded hardware and software. Tools like
//    `stm32cubemx` can be helpful to give you the big picture.
//
//    The `stm32f4xx-hal` gives you an abstraction for programming,
//    setting up clocks, assigning pins, etc.
//
//    The hal overs basic functionality like serial communication.
//    Still, in order to fully understand what is going on under the hood you need to
//    check the documentation (data sheets, user manuals etc.)
//
//    Your crate can be documented by:
//
//    > cargo doc --open --features "stm32f4xx-hal"
//
//    This will document both your crate and its dependencies besides the `core` library.
//
//    You can open the `core` library documentation by
//
//    > rustup doc
//
//    or just show the path to the doc (to open it manually)
//
//    > rustup doc --path
