#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{DriveMode, Event, Input, InputConfig, Io, Level, Output, OutputConfig, Pull};
use esp_hal::time::{Duration, Instant};
use esp_hal::{handler, main};
use heapless::Vec;
use log::info;
use series_blink::{LedDirection, toggle_direction};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static BUTTON_INPUT_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static LED_DIRECTION: Mutex<Cell<LedDirection>> = Mutex::new(Cell::new(LedDirection::Down));

#[handler]
fn button_press() {
    info!("ISR");
    critical_section::with(|cs| {
        let direction = LED_DIRECTION.borrow(cs);
        direction.set(toggle_direction(direction.get()));

        BUTTON_INPUT_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}

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

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(button_press);

    // when button is high, is pushed
    let button_config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(peripherals.GPIO6, button_config);
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| {
        BUTTON_INPUT_PIN.borrow_ref_mut(cs).replace(button);
    });

    let led_config = OutputConfig::default().with_drive_mode(DriveMode::PushPull);

    let top_led = Output::new(peripherals.GPIO5, Level::High, led_config);
    let middle_led = Output::new(peripherals.GPIO4, Level::Low, led_config);
    let bottom_led = Output::new(peripherals.GPIO3, Level::Low, led_config);

    let mut leds = Vec::<Output, 3>::new();
    let _ = leds.push(top_led);
    let _ = leds.push(middle_led);
    let _ = leds.push(bottom_led);

    let mut current_led_idx: usize = 0;
    info!("Booted starting main loop...");
    loop {
        // wait 250 ms between blinks
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(750) {}

        // toggle off current led
        leds[current_led_idx].set_low();

        critical_section::with(|cs| {
            let direction = LED_DIRECTION.borrow(cs).get();
            // move to next led
            match direction {
                LedDirection::Up => {
                    if current_led_idx == 0 {
                        current_led_idx = leds.len() - 1;
                    } else {
                        current_led_idx -= 1;
                    }
                }
                LedDirection::Down => {
                    current_led_idx += 1;
                    if current_led_idx > 2 {
                        current_led_idx = 0;
                    }
                }
            }
        });

        // toggle led on
        leds[current_led_idx].set_high();
    }
}
