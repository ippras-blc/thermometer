use crate::error::CrcError;

/// Calculates the crc8 of the input data.
pub fn calculate(data: &[u8]) -> u8 {
    append(0, data)
}

/// Calculates the crc8 of the input data with init value.
pub fn append(mut crc: u8, data: &[u8]) -> u8 {
    // `CRC = X^8 + X^5 + X^4 + X^0`
    for byte in data {
        crc ^= byte;
        for _ in 0..u8::BITS {
            let bit = crc & 0x01;
            crc >>= 1;
            if bit != 0 {
                // 0b1000_1100
                crc ^= 0x8C;
            }
        }
    }
    crc
}

/// Checks to see if data (including the crc byte) passes the crc check.
pub fn check(data: &[u8]) -> Result<(), CrcError> {
    match calculate(data) {
        0 => Ok(()),
        crc => Err(CrcError { crc }),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn calculate() {
        use super::calculate;

        assert_eq!(calculate(&[99, 1, 75, 70, 127, 255, 13, 16]), 21);
        assert_eq!(calculate(&[99, 1, 75, 70, 127, 255, 13, 16, 21]), 0);

        assert_eq!(calculate(&[97, 1, 75, 70, 127, 255, 15, 16]), 2);
        assert_eq!(calculate(&[97, 1, 75, 70, 127, 255, 15, 16, 2]), 0);

        assert_eq!(calculate(&[95, 1, 75, 70, 127, 255, 1, 16]), 155);
        assert_eq!(calculate(&[95, 1, 75, 70, 127, 255, 1, 16, 155]), 0);
    }

    #[test]
    fn check() {
        use super::check;

        assert_eq!(
            check(&[99, 1, 75, 70, 127, 255, 13, 16]),
            Err(CrcError { crc: 21 })
        );
        assert!(check(&[99, 1, 75, 70, 127, 255, 13, 16, 21]).is_ok());

        assert_eq!(
            check(&[97, 1, 75, 70, 127, 255, 15, 16]),
            Err(CrcError { crc: 2 })
        );
        assert!(check(&[97, 1, 75, 70, 127, 255, 15, 16, 2]).is_ok());

        assert_eq!(
            check(&[95, 1, 75, 70, 127, 255, 1, 16]),
            Err(CrcError { crc: 155 })
        );
        assert!(check(&[95, 1, 75, 70, 127, 255, 1, 16, 155]).is_ok());
    }
}
