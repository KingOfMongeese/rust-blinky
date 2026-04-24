#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::delay::Delay;
use esp_hal::time::Duration;
use esp_println::println;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.2.0
    println!("init");

    // get periphs
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // init delay
    let delay = Delay::new();
    println!("delay");

    // configure GPIO

    let led_pin_confg = OutputConfig::default()
        .with_drive_mode(esp_hal::gpio::DriveMode::PushPull)
        .with_drive_strength(esp_hal::gpio::DriveStrength::_10mA)
        .with_pull(esp_hal::gpio::Pull::None);

    // create output pin
    let mut led_pin = Output::new(peripherals.GPIO3, Level::Low, led_pin_confg);
    println!("running");

    loop {
        // turn on led for 1 sec
        led_pin.set_high();
        println!("led on");
        delay.delay(Duration::from_millis(500));

        // turn off led for 1 sec
        led_pin.set_low();
        println!("led off");
        delay.delay(Duration::from_millis(500));
    }
}