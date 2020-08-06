use crate::imports::*;
use crate::server::symbols::*;
pub trait JeType: Sized {
    fn to_vec_u8(&self) -> Vec<u8>;
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()>;
}

// Boolean
impl JeType for bool {
    fn to_vec_u8(&self) -> Vec<u8> {
        if *self {0b1u8.to_be_bytes().to_vec()} else {0b0u8.to_be_bytes().to_vec()}
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_u8(&mut cursor) {
            Ok(1) => Ok((true, 1)),
            Ok(0) => Ok((false, 1)),
            Ok(_) | Err(_) => Err(())
        }
    }
}

// Byte
impl JeType for i8 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_i8(&mut cursor) {
            Ok(val) => Ok((val, 1)),
            Err(_) => Err(())
        }
    }
}

// UByte
impl JeType for u8 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_u8(&mut cursor) {
            Ok(val) => Ok((val, 1)),
            Err(_) => Err(())
        }
    }
}

// Short
impl JeType for i16 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_i16::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 2)),
            Err(_) => Err(())
        }
    }
}

// UShort
impl JeType for u16 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_u16::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 2)),
            Err(_) => Err(())
        }
    }
}

// Int
impl JeType for i32 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_i32::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 4)),
            Err(_) => Err(())
        }
    }
}

// Long
impl JeType for i64 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_i64::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 8)),
            Err(_) => Err(())
        }
    }
}

// Float
impl JeType for f32 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_f32::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 4)),
            Err(_) => Err(())
        }
    }
}

// Double
impl JeType for f64 {
    fn to_vec_u8(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(be_bytes);
        match byteorder::ReadBytesExt::read_f64::<byteorder::NetworkEndian>(&mut cursor) {
            Ok(val) => Ok((val, 8)),
            Err(_) => Err(())
        }
    }
}

// String
impl JeType for String {
    fn to_vec_u8(&self) -> Vec<u8> {
        [
            JeVarInt(self.len() as i32).to_vec_u8(),
            self.as_bytes().to_vec()
        ].iter().flatten().map(|e| *e).collect()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        let (str_len, len_read) = JeVarInt::try_from_raw(be_bytes)?;
        debug!("STRING get VARINT ok len {:?} len_read {}", &str_len, len_read);
        let (_, be_bytes) = be_bytes.split_at(len_read);
        debug!("STRING DECODE ARRAY {:?}", &be_bytes);
        match String::from_utf8(be_bytes[0..str_len.0 as usize].to_owned()) {
            Ok(str) => {
                debug!("STRING DECODE OK {}", &str);
                Ok((str, str_len.0 as usize + len_read))
            },
            Err(_) => Err(())
        }
    }
}

// VarInt
impl JeType for JeVarInt {
    fn to_vec_u8(&self) -> Vec<u8> {
        crate::server::net::legacy::int_to_var_int(self.0)
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        match crate::server::net::legacy::var_int_to_int(be_bytes, 5) {
            Ok((vi, read)) => return Ok((JeVarInt(vi), read)),
            Err(_) => return Err(())
        }
    }
}

// VarLong
impl JeType for JeVarLong {
    fn to_vec_u8(&self) -> Vec<u8> {
        unimplemented!()
    }
    fn try_from_raw(be_bytes: &[u8]) -> Result<(Self, usize), ()> {
        unimplemented!()
    }
}

pub enum JeTypeError {}

#[derive(Debug, Default)]
pub struct JeVarInt(pub i32);
pub struct JeVarLong(pub i64);