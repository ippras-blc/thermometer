use std::time::Duration;

use esp_idf_svc::{
    hal::{delay::Delay, prelude::Peripherals},
    log::EspLogger,
    sys::link_patches,
};
use log::error;
use std::thread::sleep;
use thermometer::{Error, Result, Thermometer};

fn main() -> Result<()> {
    link_patches();
    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();
    error!("Initialize");

    let peripherals = Peripherals::take()?;

    // let mut led = Led::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;
    let mut thermometer = Thermometer::new(peripherals.pins.gpio2, peripherals.rmt.channel0)?;
    error!("Thermometer initialized");
    let address = {
        let mut search = thermometer.search()?;
        let Some(address) = search.next() else {
            println!("No device found");
            return Err(Error::DeviceNotFound)?;
        };
        address?
    };
    // 0x230000046eafbc28
    println!(
        "Found Device: {address:x?}, family code = {}",
        address.family_code(),
    );

    loop {
        let temperature = thermometer.temperature(&address)?;
        println!("temperature: {temperature}");
        Delay::new_default();
    }
    // loop {
    //     trigger_temp_conversion(&bus, &address)?;
    //     let temperature = get_temperature(&bus, &address)?;
    //     println!("Temperature: {}", temperature);
    //     FreeRtos::delay_ms(3000);
    // }
    Ok(())
}

// mod onewire;
