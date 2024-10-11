use self::error::Result;
use ds18b20::{
    scratchpad::{ConfigurationRegister, Resolution, Triggers},
    Driver as Ds18b20Driver, MemoryCommands, RomCommands, Scratchpad,
};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{IOPin, InputOutput, Pin, PinDriver},
    peripheral::Peripheral,
};
use log::debug;

// https://github.com/esp-rs/esp-idf-hal/blob/4f4478718e88344082b82af455192ba10efd41c8/src/onewire.rs

const RESOLUTION: Resolution = Resolution::Twelve;
const LOW: i8 = 19;
const HIGH: i8 = 30;

/// Thermometer
pub struct Thermometer<'a, T: Pin> {
    driver: Ds18b20Driver<PinDriver<'a, T, InputOutput>, Delay>,
}

impl<'a, T: IOPin> Thermometer<'a, T> {
    pub fn new(pin: impl Peripheral<P = T> + 'a) -> Result<Self> {
        let pin_driver = PinDriver::input_output(pin)?;
        let delay = Delay::new_default();
        let mut driver = Ds18b20Driver::new(pin_driver, delay)?;
        // ensure!(driver.initialization()?, "device not presence");
        driver.initialization()?;
        driver.skip_rom()?;
        driver.write_scratchpad(Scratchpad {
            configuration_register: ConfigurationRegister {
                resolution: RESOLUTION,
            },
            triggers: Triggers {
                low: LOW,
                high: HIGH,
            },
            ..Default::default()
        })?;
        Ok(Self { driver })
    }

    // PinDriver<'_, impl Pin, InputOutput>, Delay
    pub async fn scratchpad(&mut self) -> Result<Scratchpad> {
        debug!("scratchpad");
        self.driver.initialization()?;
        self.driver.skip_rom()?;
        self.driver.convert_temperature()?;
        self.driver.delay(RESOLUTION.conversion_time());
        self.driver.initialization()?;
        self.driver.skip_rom()?;
        Ok(self.driver.read_scratchpad()?)
    }

    pub async fn temperature(&mut self) -> Result<f32> {
        debug!("temperature");
        let scratchpad = self.scratchpad().await?;
        Ok(scratchpad.temperature)
    }
}

pub mod error;

// async fn temperature(
//     thermometer: &mut Ds18b20Driver<PinDriver<'_, impl Pin, InputOutput>, Delay>,
// ) -> Result<f32, ds18b20::Error<GpioError>> {
//     error!("temperature");
//     // thermometer.initialization()?;
//     // thermometer.skip_rom()?;
//     // let scratchpad = thermometer.read_scratchpad()?;
//     thermometer.initialization()?;
//     thermometer.skip_rom()?;
//     thermometer.convert_temperature()?;
//     thermometer.delay(RESOLUTION.conversion_time());
//     thermometer.initialization()?;
//     thermometer.skip_rom()?;
//     let scratchpad = thermometer.read_scratchpad()?;
//     debug!("scratchpad: {scratchpad:?}");
//     Ok(scratchpad.temperature)
// }

// // 230000046EAFBC28
// // TEMPERATURE LSB 1111_1100
// // TEMPERATURE MSB 0000_0001
// // 0000_0001_1111_1100 (1FC)
// fn initialize_thermometer(
//     pin: PinDriver<'_, impl Pin, InputOutput>,
// ) -> Result<Ds18b20Driver<PinDriver<'_, impl Pin, InputOutput>, Delay>> {
//     info!("initialize thermometer");
//     let delay = Delay::new_default();
//     let mut ds18b20_driver = Ds18b20Driver::new(pin, delay)?;
//     ensure!(ds18b20_driver.initialization()?, "device not presence");
//     ds18b20_driver.skip_rom()?;
//     ds18b20_driver.write_scratchpad(Scratchpad {
//         configuration_register: ConfigurationRegister {
//             resolution: RESOLUTION,
//         },
//         triggers: Triggers {
//             low: LOW,
//             high: HIGH,
//         },
//         ..Default::default()
//     })?;
//     Ok(ds18b20_driver)
// }

// /// Command
// trait Command {
//     fn command<T>(&mut self, f: impl Fn(&mut Self) -> Result<T>);
// }

// impl<T: InputPin + OutputPin + ErrorType, U: DelayNs> Command for Ds18b20Driver<T, U> {
//     fn command<V>(&mut self, rom: Option<Rom>, f: impl Fn(&mut Self) -> Result<V>) {
//         self.initialization()?;
//         if let Some(rom) = rom {
//             self.match_rom(rom)?;
//         } else {
//             self.skip_rom()?;
//         }
//         f(self)
//     }
// }

// pub fn command(
//     &mut self,
//     command: u8,
//     address: Option<&Address>,
//     delay: &mut impl DelayUs<u16>,
// ) -> OneWireResult<(), E> {
//     self.reset(delay)?;
//     self.skip_address(delay)?;
//     self.write_byte(command, delay)?;
//     Ok(())
// }
