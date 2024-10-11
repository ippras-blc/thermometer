use ds18b20::error::Ds18b20Error;
use esp_idf_svc::{hal::gpio::GpioError, sys::EspError};
use thiserror::Error;

/// Result
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Esp(#[from] EspError),
    #[error(transparent)]
    Ds18b20(#[from] ds18b20::Error<GpioError>),
}

impl Error {
    pub fn is_crc(&self) -> bool {
        matches!(
            self,
            Self::Ds18b20(ds18b20::Error::Ds18b20(Ds18b20Error::UnexpectedCrc { .. })),
        )
    }
}
