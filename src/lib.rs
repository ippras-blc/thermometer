pub use self::error::{Error, Result};

use crate::{
    crc8::check,
    scratchpad::{temperature, ConfigurationRegister, Resolution, Scratchpad, Triggers},
};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{IOPin, InputOutput, Pin, PinDriver},
    onewire::{DeviceSearch, OWAddress, OWCommand, OWDriver},
    peripheral::Peripheral,
    rmt::RmtChannel,
};
use log::debug;
use std::{mem::transmute, time::Duration};

pub const FAMILY_CODE: u8 = 0x28;

/// Max conversion time, up to 750 ms.
const CONVERSION_TIME_NS: u64 = 750_000_000;
const HIGH: i8 = 30;
const LOW: i8 = 19;
const RESOLUTION: Resolution = Resolution::Twelve;

/// The ds18b20 driver for esp32
pub struct Ds18b20Driver<'a> {
    driver: OWDriver<'a>,
}

impl<'a> Ds18b20Driver<'a> {
    pub fn new(
        pin: impl Peripheral<P = impl IOPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a,
    ) -> Result<Self> {
        let driver: OWDriver = OWDriver::new(pin, channel)?;
        // let delay = Delay::new_default();
        Ok(Self { driver })
    }

    /// Receive temperature
    pub fn temperature(&self, address: &OWAddress) -> Result<f32> {
        self.convert_temperature(address)?;
        let scratchpad = self.read_scratchpad(address)?;
        Ok(scratchpad.temperature)
    }

    pub fn read_rom(&self) -> Result<OWAddress> {
        self.driver.reset()?;
        self.driver.write(&[OWCommand::ReadRom as _])?;
        let mut buffer = [0u8; 8];
        self.driver.read(&mut buffer)?;
        check(&buffer)?;
        let address = u64::from_le_bytes(buffer);
        Ok(unsafe { transmute(address) })
        // Ok(OWAddress {
        //     family_code: buffer[0],
        //     serial_number: [
        //         buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
        //     ],
        //     crc: buffer[7],
        // })
        // Ok(())
    }

    /// Start a search for devices attached to the OneWire bus
    pub fn search(&mut self) -> Result<DeviceSearch<'_, 'a>> {
        Ok(self.driver.search()?)
    }

    // pub fn device(&mut self) -> Result<OWAddress> {
    //     let search = self.search()?;
    //     let address = search.next().ok_or(Error::DeviceNotFound)?;
    //     Ok(address)
    // }
}

/// Rom commands
pub trait RomCommands {
    // /// Read ROM command
    // ///
    // /// This command allows the bus master to read the DS18B20’s 8-bit family
    // /// code, unique 48-bit serial number, and 8-bit CRC. This command can only
    // /// be used if there is a single DS18B20 on the bus. If more than one slave
    // /// is present on the bus, a data collision will occur when all slaves try
    // /// to transmit at the same time (open drain will produce a wired AND
    // /// result).
    // fn read_rom(&self) -> Result<OWAddress>;

    /// Match ROM command
    ///
    /// The match ROM command, followed by a 64-bit ROM sequence, allows the bus
    /// master to address a specific DS18B20 on a multidrop bus. Only the
    /// DS18B20 that exactly matches the 64-bit ROM sequence will respond to the
    /// following memory function command. All slaves that do not match the
    /// 64-bit ROM sequence will wait for a reset pulse. This command can be
    /// used with a single or multiple devices on the bus.
    fn match_rom(&self, address: &OWAddress) -> Result<()>;

    /// Skip ROM command
    ///
    /// This command can save time in a single drop bus system by allowing the
    /// bus master to access the memory functions without providing the 64-bit
    /// ROM code. If more than one slave is present on the bus and a Read
    /// command is issued following the Skip ROM command, data collision will
    /// occur on the bus as multiple slaves transmit simultaneously (open drain
    /// pulldowns will produce a wired AND result).
    fn skip_rom(&self) -> Result<()>;

    // /// Search ROM command
    // ///
    // /// When a system is initially brought up, the bus master might not know the
    // /// number of devices on the 1-Wire bus or their 64-bit ROM codes. The
    // /// search ROM command allows the bus master to use a process of elimination
    // /// to identify the 64-bit ROM codes of all slave devices on the bus.
    // fn search_rom(&mut self) -> Result<DeviceSearch<'_, '_>>;

    /// Search alarm command
    ///
    /// When a system is initially brought up, the bus master might not know the
    /// number of devices on the 1-Wire bus or their 64-bit ROM codes. The
    /// search ROM command allows the bus master to use a process of elimination
    /// to identify the 64-bit ROM codes of all slave devices on the bus.
    fn search_alarm(&self) -> Result<()>;
}

impl RomCommands for Ds18b20Driver<'_> {
    // fn read_rom(&self) -> Result<()> {
    //     todo!();
    //     self.driver.write(&[OWCommand::ReadRom as _])?;
    //     let mut buffer = [0u8; 9];
    //     self.driver.read(&mut buffer)?;
    //     check(&buffer)?;
    //     Ok(())
    // }

    fn match_rom(&self, address: &OWAddress) -> Result<()> {
        let mut buffer = [0; 9];
        buffer[0] = OWCommand::MatchRom as _;
        let address = address.address().to_le_bytes();
        buffer[1..9].copy_from_slice(&address);
        Ok(self.driver.write(&buffer)?)
    }

    fn skip_rom(&self) -> Result<()> {
        Ok(self.driver.write(&[OWCommand::SkipRom as _])?)
    }

    // fn search_rom(&mut self) -> Result<DeviceSearch<'_, 'a>> {
    //     Ok(self.driver.search()?)
    // }

    fn search_alarm(&self) -> Result<()> {
        todo!()
    }
}

/// Memory commands
pub trait MemoryCommands {
    /// Writes bytes into scratchpad at addresses 2 through 4 (TH and TL
    /// temperature triggers and config).
    fn write_scratchpad(&self, address: &OWAddress, scratchpad: &Scratchpad) -> Result<()>;

    /// Reads bytes from scratchpad and reads CRC byte.
    fn read_scratchpad(&self, address: &OWAddress) -> Result<Scratchpad>;

    /// Copies scratchpad into nonvolatile memory (EEPROM) (addresses 2 through
    /// 4 only). Save config from scratchpad to EEPROM.
    fn copy_scratchpad(&self) -> Result<()>;

    /// This command begins a temperature conversion. No further data is
    /// required. The temperature conversion will be performed and then the
    /// DS18B20 will remain idle. If the bus master issues read time slots
    /// following this command, the DS18B20 will output 0 on the bus as long as
    /// it is busy making a temperature conversion; it will return a 1 when the
    /// temperature conversion is complete. If parasite-powered, the bus master
    /// has to enable a strong pullup for a period greater than tconv
    /// immediately after issuing this command.
    ///
    /// You should wait for the measurement to finish before reading the
    /// measurement. The amount of time you need to wait depends on the current
    /// resolution configuration
    fn convert_temperature(&self, address: &OWAddress) -> Result<()>;

    /// Recalls values stored in nonvolatile memory (EEPROM, electrically
    /// erasable programmable read-only memory) into scratchpad (temperature
    /// triggers). Load config from EEPROM to scratchpad.
    fn recall_eeprom(&self) -> Result<()>;

    /// Signals the mode of DS18B20 power supply to the master.
    fn read_power_supply(&self) -> Result<()>;
}

impl MemoryCommands for Ds18b20Driver<'_> {
    fn write_scratchpad(&self, address: &OWAddress, scratchpad: &Scratchpad) -> Result<()> {
        self.driver.reset()?;
        self.match_rom(address)?;
        self.driver.write(&[Command::WriteScratchpad as _])?;
        let buffer = [
            scratchpad.triggers.high as _,
            scratchpad.triggers.low as _,
            scratchpad.configuration_register.into(),
        ];
        Ok(self.driver.write(&buffer)?)
    }

    fn read_scratchpad(&self, address: &OWAddress) -> Result<Scratchpad> {
        self.driver.reset()?;
        self.match_rom(address)?;
        self.driver.write(&[Command::ReadScratchpad as _])?;
        let mut buffer = [0u8; 9];
        self.driver.read(&mut buffer)?;
        check(&buffer)?;
        let configuration_register = ConfigurationRegister::try_from(buffer[4])?;
        Ok(Scratchpad {
            temperature: temperature(buffer[1], buffer[0], configuration_register.resolution),
            triggers: Triggers {
                high: buffer[2] as _,
                low: buffer[3] as _,
            },
            configuration_register,
            crc: buffer[8],
        })
    }

    fn copy_scratchpad(&self) -> Result<()> {
        todo!()
    }

    fn convert_temperature(&self, address: &OWAddress) -> Result<()> {
        self.driver.reset()?;
        self.match_rom(address)?;
        self.driver.write(&[Command::ConvertTemperature as _])?;
        // delay proper time for temp conversion, assume max resolution
        // (12-bits)
        std::thread::sleep(Duration::from_nanos(CONVERSION_TIME_NS));
        Ok(())
    }

    fn recall_eeprom(&self) -> Result<()> {
        todo!()
    }

    fn read_power_supply(&self) -> Result<()> {
        todo!()
    }
}

#[allow(dead_code)]
#[repr(u8)]
enum Command {
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
