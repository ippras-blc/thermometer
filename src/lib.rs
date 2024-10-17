use self::error::Result;
use ds18b20::{
    scratchpad::{ConfigurationRegister, Resolution, Triggers},
    Driver as Ds18b20Driver, MemoryCommands, RomCommands, Scratchpad,
};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{IOPin, InputOutput, Pin, PinDriver},
    onewire::{OWAddress, OWCommand, OWDriver},
    peripheral::Peripheral,
};
use log::debug;

// https://github.com/esp-rs/esp-idf-hal/commit/aa0e257ffe308273ad20cfb759ae9849fb02e19d
// https://github.com/esp-rs/esp-idf-hal/blob/4f4478718e88344082b82af455192ba10efd41c8/src/onewire.rs
// https://github.com/esp-rs/esp-idf-hal/blob/ff343b67f37331bf0ee335af8360a37fce99761e/examples/rmt_onewire_temperature.rs#L8

const RESOLUTION: Resolution = Resolution::Twelve;
const LOW: i8 = 19;
const HIGH: i8 = 30;

/// Thermometer
pub struct Thermometer<'a, T: Pin> {
    driver: OWDriver<'a>,
}

impl<'a> Thermometer<'a> {
    pub fn new(
        pin: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a,
    ) -> Result<Self> {
        let mut onewire_driver: OWDriver = OWDriver::new(pin, channel)?;

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

    pub fn read(&self, buff: &mut [u8]) -> Result<()> {
        Ok(self.driver.read(buff)?)
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        Ok(self.driver.write(data)?)
    }

    /// Send reset pulse to the bus, and check if there are devices attached to the bus
    ///
    /// If there are no devices on the bus, this will result in an error.
    pub fn reset(&self) -> Result<()> {
        Ok(self.driver.reset(data)?)
    }

    /// Start a search for devices attached to the OneWire bus.
    pub fn search(&mut self) -> Result<DeviceSearch<'_, 'a>> {
        Ok(self.driver.search(data)?)
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

fn send_command<'a>(bus: &OWDriver, address: &OWAddress, command: Command) -> Result<(), EspError> {
    let mut buf = [0; 10];
    buf[0] = OWCommand::MatchRom as _;
    let address = address.address().to_le_bytes();
    buf[1..9].copy_from_slice(&address);
    buf[9] = command as _;

    bus.write(&buf)
}

fn get_temperature<'a>(bus: &OWDriver, address: &OWAddress) -> Result<f32, EspError> {
    bus.reset()?;

    send_command(bus, address, Command::ReadScratch)?;

    let mut buf = [0u8; 10];
    bus.read(&mut buf)?;
    let lsb = buf[0];
    let msb = buf[1];

    let temperature: u16 = (u16::from(msb) << 8) | u16::from(lsb);
    Ok(f32::from(temperature) / 16.0)
}

fn trigger_temp_conversion<'a>(bus: &OWDriver, address: &OWAddress) -> Result<(), EspError> {
    // reset bus and check if the ds18b20 is present
    bus.reset()?;

    send_command(bus, address, Command::ConvertTemp)?;

    // delay proper time for temp conversion,
    // assume max resolution (12-bits)
    std::thread::sleep(Duration::from_millis(800));

    Ok(())
}

#[allow(dead_code)]
#[repr(u8)]
enum Command {
    ConvertTemp = 0x44,
    WriteScratch = 0x4E,
    ReadScratch = 0xBE,
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
