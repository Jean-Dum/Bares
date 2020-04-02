//! bare0.rs
//!
//! Simple bare metal application
//! What it covers:
//! - constants
//! - global (static) variables
//! - checked vs. wrapping arithmetics
//! - safe and unsafe code
//! - making a safe API
//! - assertions
//! - panic handling

// build without the Rust standard library
#![no_std]
// no standard main, we declare main using [entry]
#![no_main]

// Panic handler, for textual output using semihosting
use panic_semihosting as _;

// import entry point
use cortex_m_rt::entry;

// a constant (cannot be changed at run-time)
const X_INIT: u32 = 10;
// const X_INIT: u32 = core::u32::MAX;

// global mutable variables (changed using unsafe code)
static mut X: u32 = X_INIT;
static mut Y: u32 = 0;

#[inline(always)]
fn read_x() -> u32 {
    unsafe { core::ptr::read_volatile(X as *const _) }
}

#[inline(always)]
fn write_x(val: u32) {
    unsafe { core::ptr::write_volatile(X as *mut _, val); }
}

#[inline(always)]
fn read_y() -> u32 {
    unsafe { core::ptr::read_volatile(Y as *const _) }
}

#[inline(always)]
fn write_y(val: u32) {
    unsafe { core::ptr::write_volatile(Y as *mut _, val); }
}

#[entry]
fn main() -> ! {
    // local mutable variable (changed in safe code)
    let mut x = unsafe { X };

    loop {
        x = x.wrapping_add(1);
        unsafe {
            write_x(read_x().wrapping_add(1));
            write_y(read_x());
            //X += 1;
            //Y = X;

            let _ = core::ptr::read_volatile(&X);
            let _ = core::ptr::read_volatile(&Y);
            assert!(x == X && X == Y);
        }
    }
}

// Here we assume you are using `vscode` with `cortex-debug`.
//
// 0. Compile/build and run the example in debug (dev) mode.
//
//    > cargo run --example bare0
//    (or use vscode)
//
// 1. Run the program in the debugger, let the program run for a while and
//    then press pause.
//
//    Look under Variables/Local what do you find.
//
//    --> x: 334981
//
//    In the Expressions (WATCH -vscode) view add X and Y
//    what do you find
//
//    --> X: 334981
//    --> Y: 334981
//
//    Step through one complete iteration of the loop
//    and see how the (Local) Variables are updated
//    can you foresee what will eventually happen?
//
// 	  --> If the variable get above the maximum value it can store, it would happen an overflow
//
//    Commit your answers (bare0_1)
//
// 2. Alter the constant X_INIT so that `x += 1` directly causes `x` to wrap.
// 	  What happens when `x` wraps
//    (Hint, look under OUTPUT/Adopter Output to see the `openocd` output.)
//
//    --> panicked at 'attempt to add with overflow', examples/bare0.rs:38:9
//
//    Commit your answers (bare0_2)
//
// 3. Place a breakpoint at `x += 1`
//
//    Change (both) += operations to use wrapping_add
//    load and run the program, what happens
//    --> X and Y goes from the maximum possible value to 0
//
//    Now continue execution, what happens
//    --> The programm still continue, the value goes from the maximum to 0 and then continue to increment 
//
//    Commit your answers (bare0_3)
//
//    (If the program did not succeed back to the breakpoint
//    you have some fault in the program and go back to 3.)
//
// 4. Change the assertion to `assert!(x == X && X == Y + 1)`, what happens?
//
//    --> panicked at 'assertion failed: x == X && X == Y + 1', examples/bare0.rs:46:13
//
//    Commit your answers (bare0_4)
//
// 5. Remove the assertion and implement "safe" functions for
//    reading and writing X and Y
//    e.g. read_x, read_y, write_x, write_y
//
//    Rewrite the program to use ONLY "safe" code besides the
//    read/write functions (which are internally "unsafe")
//
//    Commit your solution (bare0_5)
//
// 6. *Optional
//    Implement a read_u32/write_u32, taking a reference to a
//    "static" variable
//
//    Rewrite the program to use this abstraction instead of "read_x", etc.
//
//    Commit your solution (bare0_6)
//
