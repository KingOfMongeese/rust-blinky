#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::DriveMode;
use esp_hal::gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig, Pull};
use esp_hal::time::{Duration, Instant};
use esp_hal::{handler, main};
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static MOTION_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static SENSOR_LED_PIN: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));
static COOLDOWN_LED_PIN: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));

#[handler]
fn gpio() {
    critical_section::with(|cs| {
        // get time of interrupt
        let motion_at_timestamp = Instant::now();
        MOTION_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        // turn on lights
        let mut sensor_led = SENSOR_LED_PIN.borrow_ref_mut(cs);
        let mut cooldown_led = COOLDOWN_LED_PIN.borrow_ref_mut(cs);
        sensor_led.as_mut().unwrap().set_high();
        cooldown_led.as_mut().unwrap().set_high();

        // keep sensor led on for 500ms
        while motion_at_timestamp.elapsed() < Duration::from_millis(1200) {}
        sensor_led.as_mut().unwrap().set_low();

        // PIR sensor has a cooldown of 6.2 seconds
        // its high for 5 secs
        // doesnt accept input for 1.2
        while motion_at_timestamp.elapsed() < Duration::from_millis(5000) {}
        cooldown_led.as_mut().unwrap().set_low();
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
    io.set_interrupt_handler(gpio);

    // Motion sensor setup
    let motion_sensor_conf = InputConfig::default().with_pull(Pull::Up);
    let mut motion_sensor = Input::new(peripherals.GPIO3, motion_sensor_conf);
    motion_sensor.listen(Event::RisingEdge);

    // Led setup
    let led_conf = OutputConfig::default().with_drive_mode(DriveMode::PushPull);
    let sensor_led = Output::new(peripherals.GPIO2, Level::Low, led_conf);

    let cooldown_led = Output::new(peripherals.GPIO4, Level::Low, led_conf);

    critical_section::with(|cs| {
        MOTION_PIN.borrow_ref_mut(cs).replace(motion_sensor);
        SENSOR_LED_PIN.borrow_ref_mut(cs).replace(sensor_led);
        COOLDOWN_LED_PIN.borrow_ref_mut(cs).replace(cooldown_led);
    });

    info!("Running");
    loop {}

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
