use std::time::Duration;

use esp_idf_svc::{
    hal::{delay::Delay, prelude::Peripherals},
    log::EspLogger,
    sys::{link_patches, EspError},
};
use log::error;
use std::thread::sleep;
use thermometer::{
    scratchpad::{ConfigurationRegister, Resolution, Scratchpad},
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
    let mut thermometer = Ds18b20Driver::new(peripherals.pins.gpio2, peripherals.rmt.channel0)?;
    error!("Thermometer initialized");

    // {
    //     let mut search = thermometer.driver.search()?;
    //     error!("Search {:?}", ,);
    // }
    let addresses = thermometer
        .driver
        .search()?
        .collect::<Result<Vec<_>, EspError>>()?;
    // let mut search = thermometer.driver.search()?;
    // let address = search
    //     .filter_map(|address| address.ok())
    //     .find(|address| address.address() == 0x230000046eafbc28)
    //     .unwrap();
    // error!(
    //     "Found address: {address:x?}, family code = {}",
    //     address.family_code(),
    // );
    for address in &addresses {
        let scratchpad = thermometer
            .initialization()?
            .match_rom(&address)?
            .read_scratchpad()?;
        error!("{address:?}: {scratchpad:?}");
    }
    // thermometer
    //     .initialization()?
    //     .match_rom(&address)?
    //     .write_scratchpad(&Scratchpad {
    //         alarm_high_trigger_register: 30,
    //         alarm_low_trigger_register: 1,
    //         configuration_register: ConfigurationRegister {
    //             resolution: Resolution::Twelve,
    //         },
    //         ..Default::default()
    //     })?;
    // let scratchpad = thermometer
    //     .initialization()?
    //     .match_rom(&address)?
    //     .read_scratchpad()?;
    // error!("scratchpad: {scratchpad:?}");

    // let t = thermometer.initialization()?;
    // t.read_rom();
    // let a = thermometer.initialization()?.match_rom(&address)?;

    // let scratchpad = thermometer.read_scratchpad(&address)?;
    // // thermometer.initialization()?;
    // println!("scratchpad: {scratchpad:?}");
    // let scratchpad = thermometer.read_rom()?;
    // println!("read_rom: {scratchpad:?}");

    loop {
        for address in &addresses {
            let temperature = thermometer.temperature(&address)?;
            error!("{address:?}: {temperature}");
        }
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
