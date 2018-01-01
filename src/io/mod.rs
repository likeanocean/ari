pub mod stdin;


use std::io::{Read, Seek, SeekFrom};


pub trait ReadExt: Read {
    #[inline]
    fn read_as_string(&mut self) -> Result<String, std::io::Error> {
        let mut string = String::new();

        self.read_to_string(&mut string)?;
        Ok(string)
    }

    #[inline]
    fn read_as_bytes(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut data = vec![];

        self.read_to_end(&mut data)?;
        Ok(data)
    }

    #[inline]
    fn read_vec(&mut self, count: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut data = Vec::with_capacity(count);

        unsafe {
            let ptr = data.as_mut_ptr();
            let slice = std::slice::from_raw_parts_mut(ptr, count);

            self.read_exact(slice)?;
            data.set_len(count);
        }

        Ok(data)
    }


    // :: const generics workarounds
    //     when const generics arrive, refactor into generic code.

    #[inline]
    fn read_bytes_16(&mut self) -> Result<[u8; 16], std::io::Error> {
        unsafe {
            let mut data = std::mem::uninitialized::<[u8; 16]>();

            self.read_exact(&mut data)?;
            Ok(data)
        }
    }

    #[inline]
    fn read_bytes_32(&mut self) -> Result<[u8; 32], std::io::Error> {
        unsafe {
            let mut data = std::mem::uninitialized::<[u8; 32]>();

            self.read_exact(&mut data)?;
            Ok(data)
        }
    }
}

impl<T> ReadExt for T where T: Read + ?Sized
{
}


pub trait SeekExt: Seek {
    fn position(&mut self) -> Result<u64, std::io::Error> {
        self.seek(SeekFrom::Current(0))
    }
}

impl<T> SeekExt for T where T: Seek + ?Sized
{
}
