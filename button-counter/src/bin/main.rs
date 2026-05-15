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
use esp_hal::{clock::CpuClock, gpio::Io};
use esp_hal::gpio::{Event, Input, InputConfig, Pull};
use esp_hal::main;
use esp_hal_procmacros::handler;
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();


// ISR can be called anytime so these are defined as static and accessed in both the ISR and main thread
// static var to hold GPIO that our ISR is called from
static G_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

// static var to hold the flag that the ISR sets
static G_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

// Define ISR
#[handler]
fn gpio() {
    info!("ISR");
    critical_section::with(|cs| {
        G_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        G_FLAG.borrow(cs).set(true);
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

    let button_config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(peripherals.GPIO3, button_config);

    // when it goes from HIGH to LOW
    // because its in PUll UP, its default high, then goes to low on press
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| {
        G_PIN.borrow_ref_mut(cs).replace(button);
    });

    let mut count  = 0;
    info!("Running");

    loop {
        critical_section::with(|cs|{
            if G_FLAG.borrow(cs).get() {
                G_FLAG.borrow(cs).set(false);

                count += 1;

                info!("Button has been pressed {count} times");
            }
        });
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
