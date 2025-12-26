#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::AtomicBool;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig},
    handler, main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static LED: AtomicBool = AtomicBool::new(true);

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Button Interrupt!");

    let mut io = Io::new(peripherals.IO_MUX);
    // Set the interrupt handler for GPIO interrupts.
    io.set_interrupt_handler(handler);

    // Set GPIO7 as an output, and set its state high initially.
    let mut led = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    // Set GPIO9 as an input
    let mut button = Input::new(peripherals.GPIO9, InputConfig::default());

    // Put the button into the static variable so the interrupt handler can access it.
    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    // Extra: toggle LED on button press via atomic flag
    loop {
        let led_level: Level = LED.load(core::sync::atomic::Ordering::Relaxed).into();
        led.set_level(led_level);
    }

    // Original solution:
    // let delay = Delay::new();
    // loop {
    //     led.toggle();
    //     delay.delay_millis(500u32);
    // }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        // Toggle LED state
        let led = LED.load(core::sync::atomic::Ordering::Relaxed);
        LED.store(!led, core::sync::atomic::Ordering::Relaxed);
    });
}
