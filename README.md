# `app`

> Examples and exercises for the Nucleo STM32F401re/STM32F11re devkits.

---

## Dependencies

- Rust 1.40, or later. Run the following commands to update you Rust tool-chain and add the target for Arm Cortex M4 with hardware floating point support.

``` console
> rustup update
> rustup target add thumbv7em-none-eabihf
```

- For programming (flashing) and debugging
  - `openocd` debug host, (install using your package manager). 
  
  If you are under `Ubuntu` or `ÒSX` you may need to change the `openocd.cfg` and `.vscode/launch.json` to use the deprecated (older) script (`source [find interface/stlink-v2-1.cfg]`, and `"configFiles": [
                "interface/stlink-v2-1.cfg",
                ...
            ],` respectively).

  - `arm-none-eabi` tool-chain (install using your package manager). In the following we refer the `arm-none-eabi-gdb` as just `gdb` for brevity. 
  
  If using OSX you can download and install the latest official [GNU ARM](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads) toolchain for OSX. You need to unpack the toolchain to your computer and make sure the `bin` directory is in your path. Under OSX the path is defined in `~/.bash_profile`, add `export PATH="<INSTALL>/bin/:$PATH"`, where `<INSTALL>` is the path to the GNU tools (`$HOME/Prog/ARM/gcc-arm-none-eabi-9-2019-q4-major` or similar). (This setting will take effect for all new terminals opened.)

  - `stlink` (optional) tools for erasing and programming ST microcontrollers (install using your package manager).

- `itm` tools for ITM trace output, install by:

``` console
> cargo install itm
```

- `vscode` editor/ide and `cortex-debug` plugin. Install `vscode` using your package manager and follow the instructions at [cortex-debug](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug) (optional for an integrated debugging experience).

- `rust-analyzer` install following instructions at [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer) (optional for Rust support in `vscode`). You may first need to install `npm` (through you package manager).

---

## Examples

---

### Hello World! Building and Debugging an Application

1. Connect your devkit using USB. To check that it is found you can run:

``` console
> lsusb
...
Bus 001 Device 004: ID 0483:374b STMicroelectronics ST-LINK/V2.1
...
```

(Bus/Device/ID may vary.)

2. Run in a terminal (in the `app` project folder):

``` console
> openocd -f openocd.cfg
...
Info : Listening on port 6666 for tcl connections
Info : Listening on port 4444 for telnet connections
Info : clock speed 2000 kHz
Info : STLINK V2J20M4 (API v2) VID:PID 0483:374B
Info : Target voltage: 3.254773
Info : stm32f4x.cpu: hardware has 6 breakpoints, 4 watchpoints
Info : Listening on port 3333 for gdb connections
```

`openocd` should connect to your target using the `stlink` programmer (onboard your Nucleo devkit). See the `Trouble Shooting` section if you run into trouble. 

3. In another terminal (in the same `app` folder) run:

``` console
> cargo run --example hello
``` 
The `cargo` sub-command `run` looks in the `.cargo/config` file on the configuration (`runner = "arm-none-eabi-gdb -q -x openocd.gdb"`), if you use `gdb-multiarch`, you can change the configuration to `runner = "gdb-multiarch -q -x openocd.gdb"`.

We can also do this manually.

``` console
> cargo build --example hello
> arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/examples/hello -x openocd.gdb
```

This starts gdb with `file` being the `hello` (elf) binary, and runs the `openocd.gdb` script, which loads (flashes) the binary to the target (our devkit). The script connects to the `openocd` server, enables `semihosting` and `ITM` tracing, sets `breakpoint`s at `main` (as well as some exception handlers, more on those later), finally it flashes the binary and runs the first instruction (`stepi`). (You can change the startup behavior in the `openocd.gdb` script, e.g., to `continue` instead of `stepi`.)

4. You can now continue debugging of the program:

``` console
...
Note: automatically using hardware breakpoints for read-only addresses.
halted: PC: 0x08000a72
DefaultPreInit ()
    at /home/pln/.cargo/registry/src/github.com-1ecc6299db9ec823/cortex-m-rt-0.6.12/src/lib.rs:571
571     pub unsafe extern "C" fn DefaultPreInit() {}

(gdb) c
Continuing.

Breakpoint 1, main () at examples/hello.rs:12
12      #[entry]
```

The `cortex-m-rt` run-time initializes the system and your global variables (in this case there are none). After that it calls the `[entry]` function. Here you hit a breakpoint.

5. You can continue debugging:

``` console
(gdb) c
Continuing.
halted: PC: 0x0800043a
^C
Program received signal SIGINT, Interrupt.
hello::__cortex_m_rt_main () at examples/hello.rs:15
15          loop {
```

At this point, the `openocd` terminal should read something like:

``` console
 Thread
xPSR: 0x01000000 pc: 0x08000a1a msp: 0x20008000, semihosting
Info : halted: PC: 0x08000a72
Info : halted: PC: 0x0800043a
Hello, world!
```

Your program is now stuck in an infinite loop (doing nothing).

6. Press `CTRL-c` in the `gdb` terminal:

``` console
Program received signal SIGINT, Interrupt.
0x08000624 in main () at examples/hello.rs:14
14          loop {}
(gdb)
```

You have now compiled and debugged a minimal Rust `hello` example. `gdb` is a very useful tool so lookup some tutorials/docs (e.g., [gdb-doc](https://sourceware.org/gdb/onlinedocs/gdb/), and the [GDB Cheat Sheet](https://darkdust.net/files/GDB%20Cheat%20Sheet.pdf).

---

### ITM Tracing

The `hello.rs` example uses the `semihosting` interface to emit the trace information (appearing in the `openocd` terminal). The drawback is that `semihosting` is incredibly slow as it involves a lot of machinery to process each character. (Essentially, it writes a character to a given position in memory, runs a dedicated break instruction, `openocd` detects the break, reads the character at the given position in memory and emits the character to the console.)

A better approach is to use the ARM ITM (Instrumentation Trace Macrocell), designed to more efficiently implement tracing. The onboard `stlink` programmer can put up to 4 characters into an ITM package, and transmit that to the host (`openocd`). `openocd` can process the incoming data and send it to a file or FIFO queue. The ITM package stream needs to be decoded (header + data). To this end we use the [itmdump](https://docs.rs/itm/0.3.1/itm/) tool.

In a separate terminal, create a named fifo. If you have started `openocd` without first creating the fifo, `openocd` would have created a file `tmp/itm.fifo` for you. In general a fifo is the better option since it is a zero sized "socket" that won't blow up over time, so first remove the `tmp/itm.fifo` (if existing).

``` console
> rm /tmp/itm.fifo
> mkfifo /tmp/itm.fifo
> itmdump -f /tmp/itm.fifo
Hello, again!
```

Now you can compile and run the `itm.rs` application using the same steps as the `hello` program. In the `itmdump` console you should now have the trace output.

``` console
> cargo run --example itm
```

Under the hood there is much less overhead, the serial transfer rate is set to 2Mbit/s in between the ITM (inside of the MCU) and `stlink` programmer (onboard the Nucleo devkit). So in theory we can transmit some 200kByte/s data over ITM. However, we are limited by the USB interconnection and `openocd` to receive and forward packages.

The `stlink` programmer, buffers packages but has limited buffer space. Hence in practice, you should keep tracing to short messages, else the buffer will overflow. See trouble shooting section if you run into trouble.

---

### Rust `panic` Handling

The `rust` compiler statically analyses your code, but in cases some errors cannot be detected at compile time (e.g., array indexing out of bounds, division by zero etc.). The `rust` compiler generates code checking such faults at run-time, instead of just crashing (or even worse, continuing with faulty/undefined values like a `C` program would) . A fault in Rust will render a `panic`, with an associated error message (useful to debugging the application). We can choose how such `panic`s should be treated, e.g., transmitting the error message using `semihosting`, `ITM`, some other channel (e.g. a serial port), or simply aborting the program.

The `panic` example demonstrates some possible use cases.

The `openocd.gdb` script sets a breakpoint at `rust_begin_unwind` (a function in the  `rust core` library, used to recover errors.)

When running the example (see above how to compile and run), the `gdb` terminal will show:

``` console
...
Breakpoint 2, main () at examples/panic.rs:27
27          panic!("Oops")
(gdb) c
Continuing.
halted: PC: 0x08000404

Breakpoint 1, rust_begin_unwind (_info=0x20017fb4)
    at /home/pln/.cargo/registry/src/github.com-1ecc6299db9ec823/panic-halt-0.2.0/src/lib.rs:33
33              atomic::compiler_fence(Ordering::SeqCst);
(gdb) p *_info
$1 = core::panic::PanicInfo {payload: core::any::&Any {pointer: 0x8000760 <.Lanon.21a036e607595cc96ffa1870690e4414.142> "\017\004\000", vtable: 0x8000760 <.Lanon.21a036e607595cc96ffa1870690e4414.142>}, message: core::option::Option<&core::fmt::Arguments>::Some(0x20017fd0), location: core::panic::Location {file: <error reading variable>, line: 27, col: 5}}
```

Here `p *_info` prints the argument to `rust_begin_unwind`, at the far end you will find `line: 27, col 5`, which corresponds to the source code calling `panic("Ooops")`. (`gdb` is not (yet) Rust aware enough to figure out how the `file` field should be interpreted, but at least we get some useful information).

Alternatively we can trace the panic message over `semihosting` (comment out `extern crate panic_halt` and uncomment `extern crate panic_semihosting`).

The `openocd` console should now show:

``` console
Info : halted: PC: 0x080011a0
panicked at 'Oops', examples/panic.rs:27:5
```

Under the hood, this approach involves *formatting* of the panic message, which implementation occupies a bit of flash memory (in our case we have 512kB so plenty enough, but for the smallest of MCUs this may be a problem). Another drawback is that it requires a debugger to be connected and active.

Another alternative is to use ITM (uncomment `extern crate panic_itm`), this is faster, but be aware, the message may overflow the `ITM` buffer, so it may be unreliable. Also it assumes, that the ITM stream is actively monitored.

A third alternative would be to store the panic message in some non-volatile memory (flash, eeprom, etc.). This allows for true post-mortem debugging of a unit put in production. This approach is used e.g. in automotive applications where the workshop can read-out error codes of your vehicle.

---

### Exception Handling and Core Peripheral Access

The ARM Cortex-M processors features a set of *core* peripherals and *exception* handlers. These offer basic functionality independent of vendor (NXP, STM, ...). The `SysTick` peripheral is a 24-bit countdown timer, that raises a `SysTick` exception when hitting 0 and reloads the set value. Seen as a real-time system, we can dispatch the `SysTick` task in a periodic fashion (without accumulated drift under some additional constraints).

In the `exception.rs` example a `.` is emitted by the `SysTick` handler using `semihosting`. Running the example should give you a periodic updated of the `openocd` console.

The `exception_itm.rs` and `exception_itm_raw.rs` uses the ITM instead. The difference is the way they gain access to the `ITM` peripheral. In the first case we *steal* the whole set of core peripherals, while the in the second case we use *raw* pointer access to the `ITM`. In both cases, the code is *unsafe*, as there is no guarantee that other tasks may access the peripheral simultaneously (causing a conflict/race). Later we will see how the concurrency problem is solved in RTFM to offer safe access to peripherals.

---

### Crash - Analyzing the Exception Frame

In case the execution of an instruction fails, a `HardFault` exception is raised by the hardware, and the `HardFault` handler is executed. We can define our own handler as in example `crash.rs`. In `main` we attempt to read an illegal address, causing a `HardFault`, and we hit a breakpoint (`openocd.gdb` script sets a breakpoint at the `HardFault` handler). From there you can print the exception frame, reflecting the state of the MCU when the error occurred. You can use `gdb` to give a `back trace` of the call-stack leading up to the error. See the example for detailed information.

Most crash conditions trigger a hard fault exception, whose handler is defined via

``` rust
#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
...
```

`cortex-m-rt` generates a trampoline, that calls into your user defined `HardFault` handler. We can use `cargo expand` to view the expanded code:

``` console
> cargo expand --example crash > crash_expand.rs
```

In the generated file we find:

``` rust
#[doc(hidden)]
#[export_name = "HardFault"]
#[link_section = ".HardFault.user"]
pub unsafe extern "C" fn __cortex_m_rt_HardFault_trampoline(frame: &::cortex_m_rt::ExceptionFrame) {
   __cortex_m_rt_HardFault(frame)
}
```

The `HardFault` handler has access to the exception `frame`, a
snapshot of the CPU registers at the moment of the exception.

To better see what is happening we make a `--release` build
(It reduces the amount of redundant code.)

``` text
> cargo run --example crash --release
...
Breakpoint 2, HardFault (frame=0x20007fe0) at examples/crash.rs:28
28      #[exception]
(gdb) p/x *frame
$1 = cortex_m_rt::ExceptionFrame {r0: 0x2fffffff, r1: 0xf00000, r2: 0x0, r3: 0x0, r12: 0x0, lr: 0x800051f, pc: 0x8000524, xpsr: 0x61000000}
(gdb) disassemble frame.pc
Dump of assembler code for function crash::__cortex_m_rt_main:
   0x08000520 <+0>:     mvn.w   r0, #3489660928 ; 0xd0000000
   0x08000524 <+4>:     ldr     r0, [r0, #0]
   0x08000526 <+6>:     b.n     0x8000526 <crash::__cortex_m_rt_main+6>
End of assembler dump.
```

The program counter (`frame.pc`) contains the address of the instruction that caused the exception. In GDB one can
disassemble the program around this address to observe the instruction that caused the
exception. In our case its the `ldr r0, [r0, #0]` caused the exception. This instruction tried to load (read) a 32-bit word
from the address stored in the register `r0`. Looking again at the contents of `ExceptionFrame`
we find that `r0` contained the address `0x2FFF_FFFF` when this instruction was executed.

Looking at the assembly `mvn.w   r0, #3489660928 ; 0xd0000000`.
This is a *move* and *not* instruction, so the resulting value here is actually `0x2fffffff`. Why did it not do it straight up then as 0x2FFF_FFFF?

Well a 32 bit constant cannot be stored in a 32 bit instruction.
So under the hood it stores 0xd0, bit shifts it and bit wise inversion. This is the level of optimization Rust + LLVM is capable of.

We can further backtrace the calls leading up to the fault.

``` text
(gdb) bt
#0  HardFault (frame=0x20007fe0) at examples/crash.rs:79
#1  <signal handler called>
#2  core::ptr::read_volatile (src=0x2fffffff)
    at /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/ptr/mod.rs:948
#3  crash::__cortex_m_rt_main () at examples/crash.rs:71
#4  0x08000404 in main () at examples/crash.rs:66
```

Here we see that on `frame #2` we are doing the read causing havoc.

We can also use `panic!("Exception frame {:?}", ef);` to format and print the exception frame, e.g., over `semihosting` or `ITM`. In the example we use `semihosting`, so when continuing debugging you will eventually the exception frame printed in the `openocd` console. In the `openocd.gdb` file we set breakpoints to the exception handlers:

``` text
# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind
```

So in case, you want to go directly to a `panic!` printout of the exception frame comment out the breakpoints.

Notice. `panic!("Exception frame {:?}", ef);` will bring in the formatting code from the `core` library (which is kind of large), so in case you are scarce on flash memory, you may want use some other method.

---

### Device Crates and System View Descriptions (SVDs)

Besides the ARM provided *core* peripherals the STM32F401re/STM32F411re MCUs has numerous vendor specific peripherals (GPIOs, Timers, USARTs etc.). The vendor provides a System View Description (SVD) specifying the register block layouts (fields, enumerated values, etc.). Using the `svd2rust` tool we can derive a `Peripheral Access Crate` (PAC) providing an API for the device that allow us to access each register according to the vendors specification. The `device.rs` example showcase how a PAC for the  STM32F401re/STM32F411re MCUs can be added. (These MCUs have the same set of peripherals, only the maximum clock rating differs.)

``` shell
> cargo run --example device --features stm32f4
```

The example output a `.` each second over `semihosting` and `ITM`.

#### The `Cargo.toml` file

Looking at the `Cargo.toml` file we find:

``` toml
...
[dependencies.stm32f4]
version         = "0.9.0"
features        = ["stm32f401", "rt"]
optional        = true

...

# Built options for different examples
[[example]]
name                = "device"
required-features   = ["stm32f4"]
...
```

We compile `stm32f4` (a generic library for all STMF4 MCUs) with `features = ["stm32f401", "rt"]`, which indicates the specific MCU with `rt` (so we get the interrupt vector etc.). By having the PAC as an optional dependency, we did not need to compile it (unless we need it, and as you might have experienced already compiling the PAC takes a bit of time to compile initially). (An SVD file is typically > 50k lines, amounting to the same (or more) lines of Rust code.)

By compiling with  `--features stm32f4` we "opt-in" this dependency.

---

### Hardware Abstraction Layer

For convenience common functionality can be implemented for a specific MCU (or family of MCUs). The `stm32f4xx-hal` is a Work In Progress, implementing a Hardware Abstraction Layer (hal) for the `stm32f4` family. It implements the `https://crates.io/search?q=embedded-hal` serial trait (interface), to read and write single bytes over a serial port. However, setting up communication is out of scope for the `embedded-hal`.

The `serial.rs` example showcase a simple echo application,
repeating back incoming data over a serial port (byte by byte). You will also get trace output over the ITM.

Looking closer at the example, `rcc` is a *singleton* (`constrain` consumes the `RCC` and returns a singleton. The `freeze` consumes the  singleton (`rcc`) and sets the MCU clock tree according to the (default) `cfgr`. (Later in the exercises you will change this.)

This pattern ensures that the clock configuration will remain unchanged (the `freeze` function cannot be called again, as the `rcc` is consumed, also you cannot get a new `rcc` as the `RCC` was consumed by `constrain`).

Why is this important you may ask? Well, this *pattern* allows the compiler to check and ensure that your code (or some library that you use) does not make changes to the system (in this case the clocking), which reduces the risk of errors and improves robustness.

Similarly, we `split` the `GPIOA` into its parts (pins), and select the operating mode to `af7` for `tx` (the transmit pin `pa2`), and `rx` (the receive pin `pa3`). For details see, RM0368, figure 17 (page 151), and table 9 in STM32F401xD STM32F401xE. The GPIO pins `pa2` and `pa3` are (by default) connected to the `stlink` programmer, see section 6.8 of the Nucleo64 user manual `UM1724`. When the `stlink` programmer is connected to a linux host, the device `\dev\ttyACM0` appears as a *virtual com port*.

Now we can call `Serial::usart2` to setup the serial communication, (according to table 9 in STM32F401xD STM32F401xE documentation it is USART2).

Following the *singleton* pattern it consumes the `USART2` peripheral (to ensure that only one configuration can be active at any time). The second parameter is the pair of pins `(tx, px)` that we setup earlier.
The third parameter is the USART configuration. By default it is set to 8 bit data, and one stop bit. We set the baudrate to 115200. We also pass the `clocks` (holding information about the MCU clock setup).

At this point `tx` and `rx` is owned by `serial`. We can get access to them again by `serial.split()`.

In the loop we match on the result of `block!(rx.read())`. `block!` repeatedly calls `rx.read()` until ether a byte is received or an error returned. In case `rx.read()` succeeded, we trace the received byte over the ITM, and echo it by `tx.write(byte)`, ignoring the result (we just assume sending will always succeed). In case `rx.read` returned with an error, we trace the error message.

As the underlying hardware implementation buffers only a single byte, the input buffer may overflow (resulting in `tx.read` returning an error).

You can now compile and run the example. Start `moserial` (or some other serial communication tool), connect to `/dev/ttyACM0` with 115200 8N1). Now write a single character `a` in the `Outgoing` pane, followed by pressing <Enter>. You should receive back an `a` from the target. In the ITM trace output you should see:

``` console
Ok 97
```

Depending if <Enter> was encoded as CR+LF, CR, LF, TAB, ... you will get additional bytes sent (and received). Try sending multiple characters at once, e.g. `abcd`, you will see that the you well get a buffer overflow.

This is an example of a bad programming pattern, typically leading to serious problems in (real-time) embedded programming (so it takes more than just Rust to get it right). Later in the exercises we will see how better patterns can be adopted.

---

### Real Time For the Masses (RTFM)

RTFM allows for safe concurrency, sharing resources between different tasks running at different priorities. The resource management and scheduling follow the Stack Resource Policy, which gives us outstanding properties of race- and deadlock free scheduling, single blocking, stack sharing etc.

We start by a simple example. For the full documentation see the [RTFM book](https://rtfm.rs/0.5/book/en/).

### RTFM ITM, with Interrupt

Key here is that we share the `ITM` peripheral in between the `init` task (that runs first), the `exti0` task (that runs preemptively with a default priority of 1), and the background task `idle` (that runs on priority 0). Since the highest priority task cannot be interrupted we can safely access the shared resource directly (in `exti0`). In `idle` however, we need to lock the resource (which returns `itm` (a reference to the `ITM`) peripheral to the closure).

By `rtfm::pend` we can *simulate* we trigger an interrupt, (more realistically, interrupts are triggered by the environment, e.g., a peripheral has received some data).

``` shell
> cargo run --example rtfm_itm --features rtfm
```

For more information see [app](https://rtfm.rs/0.5/book/en/by-example/app.html).

Looking at the `Cargo.toml` file we find:

``` toml
...

[dependencies.stm32f4xx-hal]
version         = "0.6.0"
features        = ["stm32f401", "rt"]
optional        = true

[dependencies.cortex-m-rtfm]
version         = "0.5.1"
optional        = true

[features]
rtfm            = ["cortex-m-rtfm", "stm32f4xx-hal"]
...

[[example]]
name                = "rtfm_itm"
required-features   = ["rtfm"]
```

The `rtfm` feature *opt-in* the dependencies to `cortex-m-rtfm` and `stm32f4xx-hal` (which in turn *opt-in* the dependency to `stm32f4` under the `stm32f401` and `rt` features). Through the `hal` we can get access to the underlying device/PAC (peripherals, interrupts etc.).

### RTFM ITM, using Spawn

In the previous example we triggered the `exti0` task manually. We can let RTFM do that for us using `spawn` with an optional payload. Thus we have simple way to do message passing.

``` shell
> cargo run --example rtfm_itm_spawn --features rtfm
```

The `spawn.unwrap()` panics if the message could not be delivered (i.e, the queue is full). The size (capacity) of queues are 1 by default, but can be for each task individually, see [spawn](https://rtfm.rs/0.5/book/en/by-example/tasks.html).

The example shows that the message to `task1` is queued while `idle` is holding the `itm` resource.

### RTFM Schedule

Similarly to `spawn`, RTFM allows for to `schedule` messages to be spawned at a specific point in time.

``` shell
$ cargo run --example rtfm_schedule --features rtfm
```

For this to work, we have to provide an implementation of a timer for RTFM to use (e.g, `monotonic = rtfm::cyccnt::CYCCNT`, which is a built in implementation using the ARM core DWT unit). You may opt to provide your own implementation as well. Since the underlying timer hardware may differ between platforms, it's up to you to make sure the timer is initialized. For more information see [schedule](https://rtfm.rs/0.5/book/en/by-example/timer-queue.html)

### RTFM Blinky

No example suit is complete without the mandatory `blinky` demo, so here we go! The examples `rtfm_blinky.rs`, `rtfm_blinky_msg1.rs`, `rtfm_blinky_msg2.rs`, and `rtfm_blinky_msg3.rs` showcase different approaches.

- `rtfm_blinky.rs` stores the `GPIOA` peripheral as a resource, and uses a state local variable to hold the `TOGGLE` state. The `toggle` task is bound the `SysTick` exception handler, setup in `init` to fire `SysTick` with a period of 1s.

The `msg` examples uses the scheduling primitives provided by RTFM.

- `rtfm_blinky_msg1.rs` stores the `GPIOA` peripheral as a resource, and uses a state local variable to hold the `TOGGLE` state.

- `rtfm_blinky_msg2.rs` stores the `GPIOA` peripheral as a resource, but uses the message payload to represent current state.

- `rtfm_blinky_msg3.rs` uses messages to pass around both current state and the *owned* peripheral.

For all cases, RTFM ensures memory safety. Which approach to take depends on the use case.

- If your intention/design requires concurrent tasks to access a shared resource (e.g., a peripheral) you need to use the `Resources` approach.

- If your intention is that a resource should be accessed sequential manner, the message passing of owned resources is the way.

The latter approach actually proves the sequential accesses pattern, so besides its simplicity it gives you a guarantee. This comes in handy e.g. when juggling buffers between owners of memory for DMA operations, handling of secure data etc, as you are in full control over resource ownership at all times.

Regarding the HW access.

- In `init` we:
  - configure the `RCC` (enabling the `GPIOA` peripheral),
  - configure the `PA5` pin (connected to the Green LED on the Nucleo) as an output, and finally
  - delegate access to `GPIO` as a "late" resource.

- In `toggle` we:
- either define a task local resource `TOGGLE` to hold the current state, or pass it along as boolean argument.
- either access `GPIOA` as a resource (provided by the context), or as an owned resources passed as parameter.
- set/clear the `PA5` pin correspondingly. (The `bs5` field sets the `PA5` high, while `br5` clears the corresponding bit controlling the led.)
- finally schedule a message to invoke `toggle` at a later time.

---

## Trouble Shooting

Working with embedded targets involves a lot of tooling, and many things can go wrong.

---

### `openocd` fails to connect

If you end up with a program that puts the MCU in a bad state.

- Hold the `RESET` button (black), while starting `openocd`. If that does not work, disconnect the USB cable, hold the `RESET` button, re-connect the USB, start `openocd` then then let go of the button.

- However even a reset might not help you. In that case you can erase the flash memory. `st-flash` connects to the target directly (bypassing `gdb` and `openocd`) and hence more likely to get access to the target even if its in a bad state.

``` console
> st-flash erase
```

- Make sure that the `st-link` firmware is up to date, you can use the java application: [stsw-link007](https://www.st.com/en/development-tools/stsw-link007.html) to check/update the firmware.

---

### `gdb` fails to connect

`openocd` acts as a *gdb server*, while `gdb` is a *gdb client*. By default they connect over port `:3333` (: indicates that the port is on the *localhost*, not a remote connection). In cases you might have another `gdb` connection blocking the port.

``` console
$ ps -all
F S   UID   PID  PPID  C PRI  NI ADDR SZ WCHAN  TTY          TIME CMD
0 S  1000  5659  8712  0  80   0 -  6139 -      pts/1    00:00:28 openocd
0 S  1000  7549 16215  0  80   0 - 25930 se_sys pts/4    00:00:00 arm-none-eabi-g
...
```

In this case you can try killing `gdb` by:

``` console
$ kill -9 7549
```

or even

``` console
$ killall -9 arm-none-eabi-g
```

Notice, the process name is truncated for some reason...

If this did not help you can check if some other client has acquired the port, and kill the intruder accordingly. (In this case it was the gdb process so the above method would have worked, but in general it could be another process blocking the port.)

``` console
$ lsof -i :3333
COMMAND    PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
openocd   5659  pln   12u  IPv4 387143      0t0  TCP localhost:dec-notes (LISTEN)
openocd   5659  pln   13u  IPv4 439988      0t0  TCP localhost:dec-notes->localhost:59560 (ESTABLISHED)
arm-none- 7825  pln   14u  IPv4 442734      0t0  TCP localhost:59560->localhost:dec-notes (ESTABLISHED)
$ kill -9 7825
```

---

### `itmdump` no tracing or faulty output

There can be a number of reasons ITM tracing fails.

- The `openocd.gdb` script (or `launch.json` if in vscode using external ITM logging) enables ITM tracing assuming that the `/tmp/itm.fifo` and `itmdump` has been correctly setup before `gdb` is launched. So the first thing is to check that you follow the sequence suggested above.

- `openocd.gdb`sets enables ITM tracing by:

``` txt
# 16000000 must match the core clock frequency
monitor tpiu config internal /tmp/itm.fifo uart off 16000000
monitor itm port 0 on
```

The transfer speed (baud rate) is automatically negotiated, however you can set it explicitly (maximum 2000000).  

``` txt
monitor tpiu config internal /tmp/itm.fifo uart off 16000000 2000000
```

You may try a lower value.

- The `stm32f401re/stm32f411re` defaults to 16000000 (16MHz) as the core clock frequency, based on an internal oscillator. If your application sets another core clock frequency the `openocd.gdb` script (`tpiu` setting) must be changed accordingly.

- `openocd` implements a number of `events` which might be called by `gdb`, e.g.:

``` console
(gdb) monitor reset init
adapter speed: 2000 kHz
target halted due to debug-request, current mode: Thread
xPSR: 0x01000000 pc: 0x08001298 msp: 0x20018000, semihosting
adapter speed: 8000 kHz
```

This invokes the `init` event, which sets the core clock to 64MHz. If you intend to run the MCU at 64MHz (using this approach), ITM will not work unless the `tpiu` setting matches 64MHz.

``` txt
monitor tpiu config internal /tmp/itm.fifo uart off 64000000 2000000
```

If you on the other hand want to use `monitor reset init` but not having the core clock set to 64MHz, you can use a custom `.cfg` (instead of the one shipped with `openocd`). The original  `/usr/share/openocd/scripts/target/stm32f4x.cfg` looks like this:

``` txt
...
$_TARGETNAME configure -event reset-init {
	# Configure PLL to boost clock to HSI x 4 (64 MHz)
	mww 0x40023804 0x08012008   ;# RCC_PLLCFGR 16 Mhz /8 (M) * 128 (N) /4(P)
	mww 0x40023C00 0x00000102   ;# FLASH_ACR = PRFTBE | 2(Latency)
	mmw 0x40023800 0x01000000 0 ;# RCC_CR |= PLLON
	sleep 10                    ;# Wait for PLL to lock
	mmw 0x40023808 0x00001000 0 ;# RCC_CFGR |= RCC_CFGR_PPRE1_DIV2
	mmw 0x40023808 0x00000002 0 ;# RCC_CFGR |= RCC_CFGR_SW_PLL

	# Boost JTAG frequency
	adapter_khz 8000
}
```

Make a copy of the original and comment out the clock configuration, and store the file as `stm32f4x.cfg`:

``` txt
...
$_TARGETNAME configure -event reset-init {
	# # Configure PLL to boost clock to HSI x 4 (64 MHz)
	# mww 0x40023804 0x08012008   ;# RCC_PLLCFGR 16 Mhz /8 (M) * 128 (N) /4(P)
	# mww 0x40023C00 0x00000102   ;# FLASH_ACR = PRFTBE | 2(Latency)
	# mmw 0x40023800 0x01000000 0 ;# RCC_CR |= PLLON
	# sleep 10                    ;# Wait for PLL to lock
	# mmw 0x40023808 0x00001000 0 ;# RCC_CFGR |= RCC_CFGR_PPRE1_DIV2
	# mmw 0x40023808 0x00000002 0 ;# RCC_CFGR |= RCC_CFGR_SW_PLL

	# Boost JTAG frequency
	adapter_khz 8000
}
```

You can start `openocd` to use these (local) settings by:

``` console
> openocd -f interface/stlink.cfg -f stm32f4x.cfg
```

A possible advantage of `monitor reset init` is that the `adapter speed` is set to 8MHz, which at least in theory gives better transfer rate between `openocd` and the `stlink` programmer (default is 2Mbit/s). I'm not sure the improvement is noticeable.

- ITM buffer overflow

In case the ITM buffer is saturated, ITM tracing stops working (and might be hard to recover). In such case:

  1. Correct and recompile the program,

  2. Erase the flash (using `st-flash`),

  3. Power cycle the Nucleo (disconnect-and-re-connect),

  4. Remove/re-make fifo, and finally re-start `openocd`/`gdb`.
  
This ensures 1) the program will not yet again overflow the ITM buffer, 2) the faulty program is gone (and not restarted accidentally on a `RESET`), 3) the programmer firmware is restarted and does not carry any persistent state, notice a `RESET` applies only to the target, not the programmer, so if the programmer crashes it needs to be power cycled), 4) the FIFO `/tmp/itm.fifo`, `openocd` and `gdb` will have fresh states.

- Check/update the Nucleo `st-link` firmware (as mentioned above).

---

## Visual Studio Code

`vscode` is highly configurable, (keyboard shortcuts, keymaps, plugins etc.) Besides `rust-analyzer`, there is also Rust support through the [rls-vscode](https://github.com/rust-lang/rls-vscode) plugin. Both should work nicely.

It is possible to run `arm-none-eabi-gdb` from within the `vscode` using the [cortex-debug](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug) plugin.

For general information regarding debugging in `vscode`, see [debugging](https://code.visualstudio.com/docs/editor/debugging).

Some useful (default) shortcuts:

- `CTRL+SHIFT+b` compilation tasks, (e.g., compile all examples `cargo build --examples`). Cargo is smart and just re-compiles what is changed.

- `CTRL+SHIFT+d` debug launch configurations, enter debug mode to choose a binary (e.g., `itm internal (debug)`)

- Select the application you wan't to debug as the active window in `vscode`.

- `F5` to start. It will run the progam up to `main` (your `entry` point). In the `launch.json` you can comment out the `"runToMain": true` if you want to the target to halt directly after reset. In this case it will open the `cortex_m_rt/src/lib.rs` file, which contains the startup code. From there you can continue `F5` again.

- `F6` to break. The program will now be in the infinite loop (for this example). In general it will just break wherever the program counter happens to be.

- You can view the ITM trace in the `OUTPUT` tab (if using the internal `ITM` viewer). Choose the dropdown `SWO: ITM [port 0, type console]`. It should now display:

``` txt
[2019-01-02T21:35:26.457Z]   Hello, world!
```

- `SHIFT-F5` shuts down the debugger.

You may step, view the current context `variables`, add `watches`, inspect the `call stack`, add `breakpoints`, inspect `peripherals` and `registers`. Read more in the documentation for the plugin.

### Caveats

Visual Studio Code is not an "IDE", its a text editor with plugin support, with an API somewhat limiting what can be done from within a plugin (in comparison to Eclipse, IntelliJ...) regarding panel layouts etc. E.g., as far as I know you cannot view the `adapter output` (`openocd`) at the same time as the ITM trace, they are both under the `OUTPUT` tab. Moreover, each time you re-start a debug session, you need to re-select the `SWO: Name [port 0, type console]` to view the ITM output. You can work around this problem:

- Never shut down the debug session. Instead use the `DEBUG CONSOLE` (`CTRL+SHIFT+Y`) to get to the `gdb` console. This is not the *full* `gdb` interactive shell with some limitations (no *tab* completion e.g.). Make sure the MCU is stopped (`F6`). The console should show something like:

``` txt
Program
 received signal SIGINT, Interrupt.
itm::__cortex_m_rt_main () at examples/itm.rs:26
26	    loop {
```

- Now you can edit an re-compile your program, e.g. changing the text:

``` Rust
  iprintln!(stim, "Hello, again!!!!");
```

- In the `DEBUG CONSOLE`, write `load` press `ENTER` write `monitor reset init` press `ENTER`.

``` txt
load
{"token":97,"outOfBandRecord":[{"isStream":false,"type":"status","asyncClass":"download","output":[]}]}
`/home/pln/rust/app/target/thumbv7em-none-eabihf/debug/examples/itm' has changed; re-reading symbols.
Loading section .vector_table, size 0x400 lma 0x8000000
Loading section .text, size 0x10c8 lma 0x8000400
Loading section .rodata, size 0x2a8 lma 0x80014d0
Start address 0x8001298, load size 6000
Transfer rate: 9 KB/sec, 2000 bytes/write.

mon reset init
{"token":147,"outOfBandRecord":[],"resultRecords":{"resultClass":"done","results":[]}}
adapter speed: 2000 kHz
target halted due to debug-request, current mode: Thread 
xPSR: 0x01000000 pc: 0x08001298 msp: 0x20018000
adapter speed: 8000 kHz
```

- The newly compiled binary is now loaded and you can continue (`F5`). Switching to the `OUTPUT` window now preserves the ITM view and displays both traces:

``` txt
[2019-01-02T21:43:27.988Z]   Hello, world!
[2019-01-02T22:07:29.090Z]   Hello, again!
```

- Using the `gdb` terminal (`DEBUG CONSOLE`) from within `vscode` is somewhat unstable/experimental. E.g., `CTRL+c` does not `break` the target (use `F6`, or write `interrupt`). The `contiune` command, indeed continues execution (and the *control bar* changes mode, but you cannot `break` using neither `F6` nor `interrupt`). So it seems that the *state* of the `cortex-debug` plugin is not correctly updated. Moreover setting breakpoints from the `gdb` terminal indeed informs `gdb` about the breakpoint, but the state in `vscode` is not updated, so be aware.

---

### Vscode Launch Configurations

Some example launch configurations from the `.vscode/launch.json` file:

``` json
        ...
        // Launch configuration for `examples`
        // - debug
        // - semihosting
        // - internal ITM/SWO tracing 
        // - run to main 
        {
            "type": "cortex-debug",
            "request": "launch",
            "servertype": "openocd",
            "name": "itm internal (debug)",
            "preLaunchTask": "cargo build --examples",
            "executable": "./target/thumbv7em-none-eabihf/debug/examples/${fileBasenameNoExtension}",
            "configFiles": [
                "interface/stlink.cfg",
                "target/stm32f4x.cfg"
            ],
            "postLaunchCommands": [
                "monitor arm semihosting enable",
            ],
            "swoConfig": {
                "enabled": true,
                "cpuFrequency": 16000000,
                "swoFrequency": 2000000,
                "source": "probe",
                "decoders": [
                    {
                        "type": "console",
                        "label": "ITM",
                        "port": 0
                    }
                ]
            },
            "runToMain": true,
            "cwd": "${workspaceRoot}"
        },
        // Launch configuration for `examples`
        // - debug
        // - semihosting
        // - ITM/SWO tracing to file/fifo `/tmp/itm.fifo`
        // - run to main   
        {
            "type": "cortex-debug",
            "request": "launch",
            "servertype": "openocd",
            "name": "itm fifo (debug)",
            "preLaunchTask": "cargo build --examples",
            "executable": "./target/thumbv7em-none-eabihf/debug/examples/${fileBasenameNoExtension}",
            "configFiles": [
                "interface/stlink.cfg",
                "target/stm32f4x.cfg"
            ],
            "postLaunchCommands": [
                "monitor arm semihosting enable",
                "monitor tpiu config internal /tmp/itm.fifo uart off 16000000",
                "monitor itm port 0 on"
            ],
            "runToMain": true,
            "cwd": "${workspaceRoot}"
        },
```

We see some similarities to the `openocd.gdb` file, we don't need to explicitly connect to the target (that is automatic). Also launching `openocd` is automatic (for good and bad, its re-started each time, unless you use the `gdb` prompt to `load`). 

`postLaunchCommands` allows arbitrary commands to be executed by `gdb` once the session is up. E.g. in the `app` case we enable `semihosting`, while in the `itm` case we run `monitor reset init` to get the MCU in 64MHz (first example) or 16MHz (third example), before running the application (continue). Notice the first example uses the "stock" `openocd` configuration files, while the third example uses our local configuration files (that does not change the core frequency).

---

### Rust-Analyzer

In order to properly track symbols from the `core` and/or `std` libraries, you need to add the Rust source. This can be done by:

``` shell
> rustup component add rust-src
````

---

## GDB Advanced Usage

There are numerous ways to automate `gdb`. Scripts can be run by the `gdb` command `source` (`so` for short). Scripting common tasks like setting breakpoints, dumping some memory region etc. can be really helpful.

---

For now:
Copyright Per Lindgren <per.lindgren@ltu.se>