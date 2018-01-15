use std::io::prelude::*;
use std::mem::transmute;
use std::fmt;
use std::error;

use super::sample::*;

pub struct WAV {
    pub format: u16,
    pub channels: u16,
    pub samplerate: u32,
    pub bytepersec: u32,
    pub blockalign: u16,
    pub bitswidth: u16,
    pub raw_data: Vec<u8>,
}

type Result<T> = ::std::result::Result<T, Error>;

impl WAV {
    pub fn new<R: Read>(mut inner: R) -> Result<WAV> {
        let mut riff = [0; 4];
        inner.read_exact(&mut riff)?;
        if riff != [0x52, 0x49, 0x46, 0x46] {
            return Err(Error::WAVFormat);
        }

        let mut riff_size = [0; 4];
        inner.read_exact(&mut riff_size)?;
        let riff_size = u32::from_le(unsafe { transmute(riff_size) });
        let _ = riff_size;

        let mut riff_type = [0; 4];
        inner.read_exact(&mut riff_type)?;
        if riff_type != [0x57, 0x41, 0x56, 0x45] {
            return Err(Error::WAVFormat);
        }

        let mut fmt_id = [0; 4];
        inner.read_exact(&mut fmt_id)?;
        if fmt_id != [0x66, 0x6d, 0x74, 0x20] {
            return Err(Error::WAVFormat);
        }

        let mut fmt_size = [0; 4];
        inner.read_exact(&mut fmt_size)?;
        let fmt_size = u32::from_le(unsafe { transmute(fmt_size) });
        let _ = fmt_size;

        let mut fmt_format = [0; 2];
        inner.read_exact(&mut fmt_format)?;
        let fmt_format = u16::from_le(unsafe { transmute(fmt_format) });

        let mut fmt_channels = [0; 2];
        inner.read_exact(&mut fmt_channels)?;
        let fmt_channels = u16::from_le(unsafe { transmute(fmt_channels) });

        let mut fmt_samplerate = [0; 4];
        inner.read_exact(&mut fmt_samplerate)?;
        let fmt_samplerate = u32::from_le(unsafe { transmute(fmt_samplerate) });

        let mut fmt_bytepersec = [0; 4];
        inner.read_exact(&mut fmt_bytepersec)?;
        let fmt_bytepersec = u32::from_le(unsafe { transmute(fmt_bytepersec) });

        let mut fmt_blockalign = [0; 2];
        inner.read_exact(&mut fmt_blockalign)?;
        let fmt_blockalign = u16::from_le(unsafe { transmute(fmt_blockalign) });

        let mut fmt_bitswidth = [0; 2];
        inner.read_exact(&mut fmt_bitswidth)?;
        let fmt_bitswidth = u16::from_le(unsafe { transmute(fmt_bitswidth) });

        let mut data_id = [0; 4];
        inner.read_exact(&mut data_id)?;
        if data_id != [0x64, 0x61, 0x74, 0x61] {
            return Err(Error::WAVFormat);
        }

        let mut data_size = [0; 4];
        inner.read_exact(&mut data_size)?;
        let data_size = u32::from_le(unsafe { transmute(data_size) });

        let mut data_data = Vec::new();
        inner
            .take(data_size as u64)
            .read_to_end(&mut data_data)?;

        Ok(WAV {
            format: fmt_format,
            channels: fmt_channels,
            samplerate: fmt_samplerate,
            bytepersec: fmt_bytepersec,
            blockalign: fmt_blockalign,
            bitswidth: fmt_bitswidth,
            raw_data: data_data,
        })
    }

    pub fn get_sample_as<T>(&self, index: usize) -> Option<T>
    where
        T: Sample,
        T::Member: FromSampleType<i16>,
    {
        match (self.channels, self.bitswidth) {
            (2, 16) => {
                if self.raw_data.len() < (index + 1) * 4 {
                    return None;
                }
                let l = [self.raw_data[index * 4 + 0], self.raw_data[index * 4 + 1]];
                let l = i16::from_le(unsafe { transmute(l) });
                let r = [self.raw_data[index * 4 + 2], self.raw_data[index * 4 + 3]];
                let r = i16::from_le(unsafe { transmute(r) });
                T::from_raw(&[l.into_sampletype(), r.into_sampletype()])
            }
            (2, 8) => {
                if self.raw_data.len() < (index + 1) * 2 {
                    return None;
                }
                let l = self.raw_data[index * 2 + 0] as i8;
                let l = (l as i16) << 8;
                let r = self.raw_data[index * 2 + 1] as i8;
                let r = (r as i16) << 8;
                T::from_raw(&[l.into_sampletype(), r.into_sampletype()])
            }
            (1, 16) => {
                if self.raw_data.len() < (index + 1) * 2 {
                    return None;
                }
                let s = [self.raw_data[index * 2 + 0], self.raw_data[index * 2 + 1]];
                let s = i16::from_le(unsafe { transmute(s) });
                T::from_raw(&[s.into_sampletype()])
            }
            (1, 8) => {
                if self.raw_data.len() < (index + 1) {
                    return None;
                }
                let s = self.raw_data[index] as i8;
                let s = (s as i16) << 8;
                T::from_raw(&[s.into_sampletype()])
            }
            _ => panic!(),
        }
    }
}



#[derive(Debug)]
pub enum Error {
    Io(::std::io::Error),
    WAVFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::WAVFormat => write!(f, "WAV format error"),
        }
    }

}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::WAVFormat => "WAV format error", 
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::WAVFormat => None,
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}
