use crate::*;
use super::JeValError;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::error::Error;

/// Convert a `VarInt` to an `i32`.
/// Returns `(result, VarInt length)` or `JeValError`.
/// Peek the stream to get an array, then call this function.
pub fn var_int_to_int(val: &[u8], read_len: usize) -> Result<(i32, usize), Box<dyn Error>> {
    if val.is_empty() {
        return Err("Empty varint array".into());
    }
    if val[0] == 0 && read_len > 0 {
        Ok((0, 1))
    } else {
        let mut result: i32 = 0;
        let mut byte = 0b1000_0000;
        let mut cont = 1;
        let mut iteration = 0;
        while byte & 0b1000_0000 != 0 {
            byte = val[iteration];
            let masked_val = byte & 0b0111_1111;
            result = result | (((masked_val as i32) << (7 * iteration)) as i32);
            iteration += 1;
            if iteration > {
                if read_len < 5 { read_len } else { 5 }
            } {
                return Err("Invalid VarInt".into());
            }
        }
        Ok((result, iteration))
    }
}

pub fn int_to_var_int(val: i32) -> Result<Vec<u8>, JeValError> {
    let mut val = val;
    let mut buf = Vec::new();
    if val == 0 {
        buf.push(0);
    } else {
        while val != 0 {
            let mut temp = (val as u8) & 0b0111_1111;
            val = val >> 7;
            if val != 0 {
                temp = temp | 0b1000_0000;
            }
            buf.push(temp);
        }
    }
    Ok(buf)
}