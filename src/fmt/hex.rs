use std::fmt::{Debug, Display, Formatter, LowerHex, UpperHex};

pub fn to_hex(data: &[u8]) -> String {
    format!("{:x}", HexSlice(data))
}

pub fn from_hex(string: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut data = Vec::with_capacity(string.len() / 2);
    let mut value = 0;
    let mut processed = 0;

    for byte in string.bytes() {
        value <<= 4;

        #[rustfmt::skip]
        match byte {
            b'A'..=b'F' => value |= byte - b'A' + 10,
            b'a'..=b'f' => value |= byte - b'a' + 10,
            b'0'..=b'9' => value |= byte - b'0',

            b' '  => { value >>= 4; continue; },
            b'\r' => { value >>= 4; continue; },
            b'\n' => { value >>= 4; continue; },
            b'\t' => { value >>= 4; continue; },
            _ => return Err(std::io::ErrorKind::InvalidInput.into()),
        }

        processed += 1;

        if processed == 2 {
            data.push(value);
            processed = 0;
        }
    }

    match processed {
        0 => Ok(data.into_iter().collect()),
        _ => Err(std::io::ErrorKind::InvalidInput.into()),
    }
}

pub struct HexSlice<'a>(pub &'a [u8]);

impl Debug for HexSlice<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        LowerHex::fmt(self, formatter)
    }
}

impl Display for HexSlice<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        LowerHex::fmt(self, formatter)
    }
}

impl LowerHex for HexSlice<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            write!(formatter, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl UpperHex for HexSlice<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            write!(formatter, "{:02X}", byte)?;
        }

        Ok(())
    }
}
