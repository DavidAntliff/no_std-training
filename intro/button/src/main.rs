#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, IO};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    println!("Button turns LED off");

    // Set GPIO23 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let button = io.pins.gpio23.into_floating_input();
    let mut led = io.pins.gpio4.into_push_pull_output();

    led.set_high().unwrap();

    // Check the button state and set the LED state accordingly.
    loop {
        // button high is button released, LED on
        if button.is_high().unwrap() {
            led.set_high()
        } else {
            led.set_low()
        }.unwrap();
    }
}
