use crate::*;
use super::JeValError;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn var_int_to_int(stream: &mut std::net::TcpStream) -> Result<(i32, usize), JeValError> {
    let mut _tmp = vec![0; 1];
    match stream.peek(&mut _tmp) {
        Ok(size) => {
            if _tmp[0] == 0 {
                Err(JeValError::EmptySocket)
            } else {
                let mut result: i32 = 0;
                let mut byte = 0b1000_0000;
                let mut cont = 1;
                let mut iteration = 0;
                while byte & 0b1000_0000 != 0 {
                    println!("vi_i");
                    byte = stream.read_u8().map_err(|e| JeValError::EndOfSocket)?;
                    let masked_val = byte & 0b0111_1111;
                    result = result | (((masked_val as i32) << (7 * iteration)) as i32);
                    iteration += 1;
                    if iteration > 5 {
                        return Err(JeValError::OversizedVarInt);
                    }
                }
                Ok((result, iteration))
            }
        },
        Err(e) => Err(JeValError::SocketRead(e))
    }
}

pub fn int_to_var_int(val: i32) -> Result<Vec<u8>, JeValError> {
    println!("== WRITE VARINT {} ==", val);
    let mut val = val;
    let mut buf = Vec::new();
    if val == 0 {
        println!("W VARINT {}", 0);
        buf.push(0);
    } else {
        while val != 0 {
            let mut temp = (val as u8) & 0b0111_1111;
            val = val >> 7;
            if val != 0 {
                temp = temp | 0b1000_0000;
            }
            println!("W VARINT {}", temp);
            buf.push(temp);
        }
    }
    Ok(buf)
}