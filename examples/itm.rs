//! Sends "Hello, again!" over the ITM port 0
//!
//! ITM is much faster than semihosting. Like 4 orders of magnitude or so.
//!
//! You'll need [`itmdump`] to view the output.
//!
//! [`itmdump`]: https://docs.rs/itm/0.3.1/itm/
//!
//! ---
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m::{iprintln, Peripherals};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let stim = &mut p.ITM.stim[0];

    iprintln!(stim, "Hello, again!");
    loop {
        continue;
    }
}
