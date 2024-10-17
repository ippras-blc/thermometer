pub use self::error::{Error, Result};

use crate::scratchpad::{Resolution, Scratchpad};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{IOPin, InputOutput, Pin, PinDriver},
    onewire::{DeviceSearch, OWAddress, OWCommand, OWDriver},
    peripheral::Peripheral,
    rmt::RmtChannel,
};
use log::debug;
use std::time::Duration;

pub const FAMILY_CODE: u8 = 0x28;

/// Max conversion time, up to 750 ms.
const CONVERSION_TIME_NS: u32 = 750_000_000;
const HIGH: i8 = 30;
const LOW: i8 = 19;
const RESOLUTION: Resolution = Resolution::Twelve;
// const DEFAULT_SCRATCHPAD: Scratchpad = Scratchpad {
//     configuration_register: ConfigurationRegister {
//         resolution: RESOLUTION,
//     },
//     triggers: Triggers { low: 19, high: 30 },
//     ..Default::default()
// };

/// Thermometer
pub struct Thermometer<'a> {
    driver: OWDriver<'a>,
}

impl<'a> Thermometer<'a> {
    pub fn new(
        pin: impl Peripheral<P = impl IOPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a,
    ) -> Result<Self> {
        let mut driver: OWDriver = OWDriver::new(pin, channel)?;
        let delay = Delay::new_default();
        // driver.initialization()?;
        // driver.skip_rom()?;
        // driver.write_scratchpad(Scratchpad {
        //     configuration_register: ConfigurationRegister {
        //         resolution: RESOLUTION,
        //     },
        //     triggers: Triggers {
        //         low: LOW,
        //         high: HIGH,
        //     },
        //     ..Default::default()
        // })?;
        Ok(Self { driver })
    }

    /// Receive temperature
    pub fn temperature(&self, address: &OWAddress) -> Result<f32> {
        self.convert_temperature(address)?;
        let scratchpad = self.read_scratchpad(address)?;
        Ok(scratchpad.temperature)
    }

    /// Read scratchpad
    pub fn read_scratchpad(&self, address: &OWAddress) -> Result<Scratchpad> {
        self.driver.reset()?;
        self.memory_command(address, MemoryCommand::ReadScratchpad)?;
        let mut buffer = [0u8; 9];
        self.driver.read(&mut buffer)?;
        buffer.try_into()
    }

    pub fn write_scratchpad(&self, address: &OWAddress) -> Result<Scratchpad> {
        self.driver.reset()?;
        self.memory_command(address, MemoryCommand::ReadScratchpad)?;
        let mut buffer = [0u8; 9];
        self.driver.read(&mut buffer)?;
        buffer.try_into()
    }

    /// Convert temperature
    pub fn convert_temperature(&self, address: &OWAddress) -> Result<()> {
        self.driver.reset()?;
        self.memory_command(address, MemoryCommand::ConvertTemperature)?;
        // delay proper time for temp conversion,
        // assume max resolution (12-bits)
        std::thread::sleep(Duration::from_millis(800));
        Ok(())
    }

    /// Start a search for devices attached to the OneWire bus.
    pub fn search(&mut self) -> Result<DeviceSearch<'_, 'a>> {
        Ok(self.driver.search()?)
    }

    // pub fn device(&mut self) -> Result<OWAddress> {
    //     let search = self.search()?;
    //     let address = search.next().ok_or(Error::DeviceNotFound)?;
    //     Ok(address)
    // }

    // Send memory command
    fn memory_command(&self, address: &OWAddress, memory_command: MemoryCommand) -> Result<()> {
        let mut buffer = [0; 10];
        buffer[0] = OWCommand::MatchRom as _;
        let address = address.address().to_le_bytes();
        buffer[1..9].copy_from_slice(&address);
        buffer[9] = memory_command as _;
        Ok(self.driver.write(&buffer)?)
    }

    // pub async fn scratchpad(&mut self) -> Result<Scratchpad> {
    //     debug!("scratchpad");
    //     self.driver.initialization()?;
    //     self.driver.skip_rom()?;
    //     self.driver.convert_temperature()?;
    //     self.driver.delay(RESOLUTION.conversion_time());
    //     self.driver.initialization()?;
    //     self.driver.skip_rom()?;
    //     Ok(self.driver.read_scratchpad()?)
    // }

    // pub async fn temperature(&mut self) -> Result<f32> {
    //     debug!("temperature");
    //     let scratchpad = self.scratchpad().await?;
    //     Ok(scratchpad.temperature)
    // }
}

#[allow(dead_code)]
#[repr(u8)]
enum MemoryCommand {
    WriteScratchpad = 0x4E,
    ReadScratchpad = 0xBE,
    CopyScratchpad = 0x48,
    ConvertTemperature = 0x44,
    RecallE2Memory = 0xB8,
    ReadPowerSupply = 0xB4,
}

pub mod crc8;
pub mod error;
pub mod scratchpad;

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
