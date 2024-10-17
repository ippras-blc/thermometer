use crate::{crc8::check, error::Error};

pub(crate) const NINE: u8 = 0b00011111;
pub(crate) const TEN: u8 = 0b00111111;
pub(crate) const ELEVEN: u8 = 0b01011111;
pub(crate) const TWELVE: u8 = 0b01111111;

/// Max conversion time, up to 750 ms.
const CONVERSION_TIME_NS: u32 = 750_000_000;

/// Scratchpad
#[derive(Clone, Copy, Debug, Default)]
pub struct Scratchpad {
    pub temperature: f32,
    pub configuration_register: ConfigurationRegister,
    pub triggers: Triggers,
    pub crc: u8,
}

impl TryFrom<[u8; 9]> for Scratchpad {
    type Error = Error;

    fn try_from(value: [u8; 9]) -> Result<Self, Self::Error> {
        check(&value)?;
        let configuration_register = ConfigurationRegister::try_from(value[4])?;
        Ok(Scratchpad {
            temperature: to_temperature(value[1], value[0], configuration_register.resolution),
            triggers: Triggers {
                high: value[2] as _,
                low: value[3] as _,
            },
            configuration_register,
            crc: value[8],
        })
    }
}

/// Configuration register
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfigurationRegister {
    pub resolution: Resolution,
}

impl TryFrom<u8> for ConfigurationRegister {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            NINE => Ok(Self {
                resolution: Resolution::Nine,
            }),
            TEN => Ok(Self {
                resolution: Resolution::Ten,
            }),
            ELEVEN => Ok(Self {
                resolution: Resolution::Eleven,
            }),
            TWELVE => Ok(Self {
                resolution: Resolution::Twelve,
            }),
            configuration_register => Err(Error::UnexpectedConfigurationRegister {
                configuration_register,
            }),
        }
    }
}

impl From<ConfigurationRegister> for u8 {
    fn from(value: ConfigurationRegister) -> Self {
        match value.resolution {
            Resolution::Nine => NINE,
            Resolution::Ten => TEN,
            Resolution::Eleven => ELEVEN,
            Resolution::Twelve => TWELVE,
        }
    }
}

/// Temperature resolution: 9, 10, 11 or 12 bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Resolution {
    Nine,
    Ten,
    Eleven,
    #[default]
    Twelve,
}

impl Resolution {
    /// Conversion time (ns)
    pub fn conversion_time(&self) -> u32 {
        match self {
            Resolution::Nine => CONVERSION_TIME_NS / 8,
            Resolution::Ten => CONVERSION_TIME_NS / 4,
            Resolution::Eleven => CONVERSION_TIME_NS / 2,
            Resolution::Twelve => CONVERSION_TIME_NS,
        }
    }
}

/// Temperature triggers: high and low.
#[derive(Clone, Copy, Debug, Default)]
pub struct Triggers {
    pub high: i8,
    pub low: i8,
}

pub fn to_temperature(msb: u8, lsb: u8, resolution: Resolution) -> f32 {
    let divider = match resolution {
        Resolution::Nine => 2.0,
        Resolution::Ten => 4.0,
        Resolution::Eleven => 8.0,
        Resolution::Twelve => 16.0,
    };
    i16::from_be_bytes([msb, lsb]) as f32 / divider
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn configuration_register() {
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b0_00_11110
            }),
            ConfigurationRegister::try_from(0b0_00_11110),
        );
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b0_00_11101
            }),
            ConfigurationRegister::try_from(0b0_00_11101),
        );
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b0_00_11011
            }),
            ConfigurationRegister::try_from(0b0_00_11011),
        );
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b0_00_10111
            }),
            ConfigurationRegister::try_from(0b0_00_10111),
        );
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b0_00_01111
            }),
            ConfigurationRegister::try_from(0b0_00_01111),
        );
        assert_eq!(
            Err(Ds18b20Error::UnexpectedConfigurationRegister {
                configuration_register: 0b1_00_11111
            }),
            ConfigurationRegister::try_from(0b1_00_11111),
        );
    }

    #[test]
    fn temperature() {
        // Temperature
        assert_eq!(125.0, to_temperature(0x07, 0xD0, Default::default()));
        assert_eq!(85.0, to_temperature(0x05, 0x50, Default::default()));
        assert_eq!(25.0625, to_temperature(0x01, 0x91, Default::default()));
        assert_eq!(10.125, to_temperature(0x00, 0xA2, Default::default()));
        assert_eq!(0.5, to_temperature(0x00, 0x08, Default::default()));
        assert_eq!(0.0, to_temperature(0x00, 0x00, Default::default()));
        assert_eq!(-0.5, to_temperature(0xFF, 0xF8, Default::default()));
        assert_eq!(-10.125, to_temperature(0xFF, 0x5E, Default::default()));
        assert_eq!(-25.0625, to_temperature(0xFE, 0x6F, Default::default()));
        assert_eq!(-55.0, to_temperature(0xFC, 0x90, Default::default()));
    }
}
