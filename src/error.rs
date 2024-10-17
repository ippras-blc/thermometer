use crate::{
    scratchpad::{ELEVEN, NINE, TEN, TWELVE},
    FAMILY_CODE,
};
use esp_idf_svc::{hal::gpio::GpioError, sys::EspError};
use thiserror::Error;

/// Result
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Esp(#[from] EspError),
    #[error("device not found")]
    DeviceNotFound,
    #[error("unexpected configuration register {{ configuration_register={configuration_register:b}, expected=[{NINE:b}, {TEN:b}, {ELEVEN:b}, {TWELVE:b}] }}")]
    UnexpectedConfigurationRegister { configuration_register: u8 },
    #[error("unexpected CRC {{ crc={crc}, expected={expected} }}")]
    UnexpectedCrc { crc: u8, expected: u8 },
}

impl Error {
    pub fn is_crc(&self) -> bool {
        matches!(self, Self::UnexpectedCrc { .. })
    }
}
