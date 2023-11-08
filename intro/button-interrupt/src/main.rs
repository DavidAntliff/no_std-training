#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    gpio::{Event, Gpio23, Input, Floating, IO},
    interrupt,
    peripherals::{self, Peripherals},
    prelude::*,
    xtensa_lx, Delay,
};

static BUTTON: Mutex<RefCell<Option<Gpio23<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Press button to trigger interrupt");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // Set GPIO7 as an output, and set its state high initially.
    let mut led = io.pins.gpio4.into_push_pull_output();
    led.set_high().unwrap();

    // Set GPIO9 as an input
    let mut button = io.pins.gpio23.into_floating_input();

    button.listen(Event::FallingEdge);

    // Move the button into the static BUTTON
    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    // Enable the interrupt
    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();
    unsafe { xtensa_lx::interrupt::enable(); }

    let mut delay = Delay::new(&clocks);
    loop {
        delay.delay_ms(500_u32);
        led.toggle().unwrap();
    }
}

// Interrupt name must match the interrupt in interrupt::enable()
#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
