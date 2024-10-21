use std::time::Duration;

use esp_idf_svc::{
    hal::{delay::Delay, prelude::Peripherals},
    log::EspLogger,
    sys::link_patches,
};
use log::error;
use std::thread::sleep;
use thermometer::{
    scratchpad::{ConfigurationRegister, Resolution, Scratchpad, Triggers},
    Ds18b20Driver, Error, Result,
};

// 0x230000046eafbc28
// 0: 0x4500000088204e28
// 1: 0x970000006a14fe28
fn main() -> Result<()> {
    link_patches();
    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();
    error!("Initialize");

    let peripherals = Peripherals::take()?;

    // let mut led = Led::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;
    let mut thermometer = Ds18b20Driver::new(peripherals.pins.gpio10, peripherals.rmt.channel0)?;
    error!("Thermometer initialized");

    let mut search = thermometer.driver.search()?;
    let address = search
        .filter_map(|address| address.ok())
        .find(|address| address.address() == 0x230000046eafbc28)
        .unwrap();
    println!(
        "Found address: {address:x?}, family code = {}",
        address.family_code(),
    );
    let scratchpad = thermometer
        .initialization()?
        .match_rom(&address)?
        .read_scratchpad()?;
    println!("scratchpad: {scratchpad:?}");
    thermometer
        .initialization()?
        .match_rom(&address)?
        .write_scratchpad(&Scratchpad {
            alarm_high_trigger_register: 30,
            alarm_low_trigger_register: 1,
            configuration_register: ConfigurationRegister {
                resolution: Resolution::Twelve,
            },
            ..Default::default()
        })?;
    let scratchpad = thermometer
        .initialization()?
        .match_rom(&address)?
        .read_scratchpad()?;
    println!("scratchpad: {scratchpad:?}");

    let t = thermometer.initialization()?;
    let a = thermometer.initialization()?.match_rom(&address)?;
    t.read_rom();

    // let scratchpad = thermometer.read_scratchpad(&address)?;
    // // thermometer.initialization()?;
    // println!("scratchpad: {scratchpad:?}");
    // let scratchpad = thermometer.read_rom()?;
    // println!("read_rom: {scratchpad:?}");

    // loop {
    //     let temperature = thermometer.temperature(&address)?;
    //     println!("temperature: {temperature}");
    //     Delay::new_default();
    // }
    // loop {
    //     trigger_temp_conversion(&bus, &address)?;
    //     let temperature = get_temperature(&bus, &address)?;
    //     println!("Temperature: {}", temperature);
    //     FreeRtos::delay_ms(3000);
    // }
    Ok(())
}

// mod onewire;
