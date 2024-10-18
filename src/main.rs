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
    Ds18b20Driver, Error, MemoryCommands, Result,
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

    let mut search = thermometer.search()?;
    let address = search
        .filter_map(|address| address.ok())
        .find(|address| address.address() == 0x230000046eafbc28)
        .unwrap();
    println!(
        "Found address: {address:x?}, family code = {}",
        address.family_code(),
    );
    let scratchpad = thermometer.read_scratchpad(&address)?;
    println!("scratchpad: {scratchpad:?}");
    thermometer.write_scratchpad(
        &address,
        &Scratchpad {
            triggers: Triggers { low: 1, high: 30 },
            configuration_register: ConfigurationRegister {
                resolution: Resolution::Twelve,
            },
            ..Default::default()
        },
    )?;
    let scratchpad = thermometer.read_scratchpad(&address)?;
    println!("scratchpad: {scratchpad:?}");
    let scratchpad = thermometer.read_rom()?;
    println!("read_rom: {scratchpad:?}");

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
}

// mod onewire;
