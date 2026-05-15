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
use esp_hal::gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// gives us 8 blinking modes
const BLINK_DELAY_STARTING: u32 = 4_000_000;
const BLINK_DELAY_INCREMENT: u32 = 500_000;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.2.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let led_config = OutputConfig::default().with_drive_mode(DriveMode::PushPull);

    // pull up is defualt high, not press GPIO reads high, so we use is_low below to read it
    let button_config = InputConfig::default().with_pull(Pull::Up);

    let mut led = Output::new(peripherals.GPIO10, Level::High, led_config);
    let button = Input::new(peripherals.GPIO3, button_config);

    let mut blink_delay = BLINK_DELAY_STARTING;

    led.set_low();
    info!("Blinking every: {blink_delay} periods");

    // This timing loop isnt great, the button could be held for a bit too
    // added a delay, all presses within the same peroid dont happen
    // this means that there is a cool down peroid the lengh of blink_delay where buttons dont register
    // but it gives the user finer grained control
    // the button could just be held too
    loop {
        let mut incremented_this_period = false;
        for _ in 1..blink_delay {
            if button.is_low() && !incremented_this_period {
                incremented_this_period = true;
                blink_delay -= BLINK_DELAY_INCREMENT;

                if blink_delay < BLINK_DELAY_INCREMENT {
                    blink_delay = BLINK_DELAY_STARTING;
                }

                info!("Button pushed! Now blinking every: {blink_delay} periods");
            }
        }

        led.toggle();
    }
}
