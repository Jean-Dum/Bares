//! bare2.rs
//!
//! Measuring execution time
//!
//! What it covers
//! - Generating documentation
//! - Using core peripherals
//! - Measuring time using the DWT
//! - ITM tracing using `iprintln`
//! - Panic halt
//!

#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m::{iprintln, peripheral::DWT, Peripherals};
use cortex_m_rt::entry;

// burns CPU cycles by just looping `i` times
#[inline(never)]
fn wait(i: u32) {
    for _ in 0..i {
        // no operation (ensured not optimized out)
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let stim = &mut p.ITM.stim[0];
    let mut dwt = p.DWT;

    iprintln!(stim, "bare2");

    dwt.enable_cycle_counter();

    // Reading the cycle counter can be done without `owning` access
    // the DWT (since it has no side effect).
    //
    // Look in the docs:
    // pub fn enable_cycle_counter(&mut self)
    // pub fn get_cycle_count() -> u32
    //
    // Notice the difference in the function signature!

    let start = DWT::get_cycle_count();
    wait(1_000_000);
    let end = DWT::get_cycle_count();

    // notice all printing outside of the section to measure!
    iprintln!(stim, "Start {:?}", start);
    iprintln!(stim, "End {:?}", end);
    iprintln!(stim, "Diff {:?}", end - start);

    loop {}
}

// 0. Setup
//    > cargo doc --open
//
//    This will document your crate, and open the docs in your browser.
//    If it does not auto-open, then copy paste the path in your browser.
//    (Notice, it will try to document all dependencies, you may have only one
//    one panic handler, so comment out all but one in `Cargo.toml`.)
//
//    In the docs, search (`S`) for DWT, and click `cortex_m::peripheral::DWT`.
//    Read the API docs.
//
// 1. Build and run the application (debug build).
//    Setup ITM tracing (see `bare1.rs`) and `openocd` (if not using vscode).
//
//    > cargo run --example bare2
//    (or use the vscode build task)
//
//    What is the output in the ITM console?
//
        // bare2
        // Start 13866373
        // End 574866540
        // Diff 561000167
//  --> The debug optimized code takes 561000167 clock cycles to execute
//
//    Rebuild and run in release mode
//
//    > cargo build --example bare2 --release

//      --> Output of ITM console:
        // bare2
        // Start 1038508976
        // End 1042508984
        // Diff 4000008

//    --> The release optimized code takes 4000008 clock cycles to execute
//    --> The code execution is really faster, it enter almost directly in the last loop. With the debug mode it takes a long time to execute the wait function
//
//    Compute the ratio between debug/release optimized code
//    (the speedup).
//
//    --> debug/release = 561000167/4000008 = 140
//    --> The release optimized code is 140 times faster than debug optimized code
//
//    commit your answers (bare2_1)
//
// 3. *Optional
//    Inspect the generated binaries, and try stepping through the code
//    for both debug and release binaries. How do they differ?
//

// Assembly code for release mode:
/*
Dump of assembler code for function bare2::wait:
   0x08000738 <+0>:	movw	r0, #16960	; 0x4240
   0x0800073c <+4>:	movt	r0, #15
=> 0x08000740 <+8>:	subs	r0, #1
   0x08000742 <+10>:	nop
   0x08000744 <+12>:	bne.n	0x8000740 <bare2::wait+8>
   0x08000746 <+14>:	bx	lr
End of assembler dump.
*/

// Assembly code for debug mode:
/*
Dump of assembler code for function bare2::wait:
   0x080008e4 <+0>:	push	{r7, lr}
   0x080008e6 <+2>:	mov	r7, sp
   0x080008e8 <+4>:	sub	sp, #48	; 0x30
   0x080008ea <+6>:	str	r0, [sp, #36]	; 0x24
   0x080008ec <+8>:	movs	r1, #0
=> 0x080008ee <+10>:	str	r1, [sp, #12]
   0x080008f0 <+12>:	str	r0, [sp, #16]
   0x080008f2 <+14>:	ldr	r0, [sp, #12]
   0x080008f4 <+16>:	ldr	r1, [sp, #16]
   0x080008f6 <+18>:	bl	0x8000e16 <<I as core::iter::traits::collect::IntoIterator>::into_iter>
   0x080008fa <+22>:	str	r0, [sp, #8]
   0x080008fc <+24>:	str	r1, [sp, #4]
   0x080008fe <+26>:	b.n	0x8000900 <bare2::wait+28>
   0x08000900 <+28>:	ldr	r0, [sp, #8]
   0x08000902 <+30>:	str	r0, [sp, #20]
   0x08000904 <+32>:	ldr	r1, [sp, #4]
   0x08000906 <+34>:	str	r1, [sp, #24]
   0x08000908 <+36>:	b.n	0x800090a <bare2::wait+38>
   0x0800090a <+38>:	add	r0, sp, #20
   0x0800090c <+40>:	bl	0x8000d7c <core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::Range<A>>::next>
   0x08000910 <+44>:	str	r1, [sp, #32]
   0x08000912 <+46>:	str	r0, [sp, #28]
   0x08000914 <+48>:	b.n	0x8000916 <bare2::wait+50>
   0x08000916 <+50>:	ldr	r0, [sp, #28]
   0x08000918 <+52>:	cmp	r0, #0
   0x0800091a <+54>:	beq.n	0x8000920 <bare2::wait+60>
   0x0800091c <+56>:	b.n	0x800091e <bare2::wait+58>
   0x0800091e <+58>:	b.n	0x8000926 <bare2::wait+66>
   0x08000920 <+60>:	add	sp, #48	; 0x30
   0x08000922 <+62>:	pop	{r7, pc}
   0x08000924 <+64>:	udf	#254	; 0xfe
   0x08000926 <+66>:	ldr	r0, [sp, #32]
   0x08000928 <+68>:	str	r0, [sp, #40]	; 0x28
   0x0800092a <+70>:	str	r0, [sp, #44]	; 0x2c
   0x0800092c <+72>:	bl	0x80011d4 <cortex_m::asm::nop>
   0x08000930 <+76>:	b.n	0x8000932 <bare2::wait+78>
   0x08000932 <+78>:	b.n	0x800090a <bare2::wait+38>
End of assembler dump.
*/

//    --> We can see that the assembly code of the debug mode is way longer than the release mode. This partially explains the execution time difference.