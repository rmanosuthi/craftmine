use uuid::Uuid;
use super::{int_to_var_int, var_int_to_int};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{error::Error, io::{Read, Write, Cursor, Seek, SeekFrom}, convert::TryFrom};
use futures::future::poll_fn;

#[derive(Debug)]
pub enum JeValError {
    OversizedVarInt
}

#[derive(Debug)]
pub enum JeNetError {
    IoError(std::io::Error),
    VarIntOverflow,
    EmptySocket
}

pub fn server_response_json(
    server_name: &str,
    server_protocol: u16,
    max_players: u64,
    online_players: u64,
    sample_players: &[(&str, Uuid)],
    desc: &str,
    favicon: &str,
) -> String {
    let mut result = serde_json::to_string(&serde_json::json!({
        "version": {
            "name": server_name,
            "protocol": server_protocol
        },
        "players": {
            "max": max_players,
            "online": online_players,
            "sample": []
        },
        "description": {
            "extra": [
                {
                    "text": desc
                }
            ],
            "text": ""
        },
        //"favicon": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
    }))
    .unwrap();
    result
}

pub async fn read_from_je(stream: &mut tokio::net::TcpStream) -> Result<(usize, i32, Vec<u8>), ()> {
    let mut poll_test = [0u8; 1];
    poll_fn(|cx| {
        stream.poll_peek(cx, &mut poll_test)
    }).await.map_err(|_| ())?;
    let mut len_peek = [0u8; 5];
    let len_peek_read = stream.peek(&mut len_peek).await.map_err(|_| ())?;
    let (len, len_bytes_read) = match var_int_to_int(&mut len_peek, len_peek_read) {
        Ok(tup) => tup,
        Err(_) => (0, 9999)
    };

    if len_bytes_read == 9999 {
        // error parsing varint
        stream.take(1).read_exact(&mut poll_test).await.map_err(|_| ())?;
        return Err(())
    }

    // advance stream by len_bytes_read
    stream.take(len_bytes_read as u64).read(&mut len_peek).await.map_err(|_| ())?;

    let mut id_peek = [0u8; 5];
    let id_peek_read = stream.peek(&mut id_peek).await.map_err(|_| ())?;
    let (id, id_bytes_read) = var_int_to_int(&mut id_peek, id_peek_read).map_err(|_| ())?;

    // advance stream by id_bytes_read
    stream.take(id_bytes_read as u64).read(&mut id_peek).await.map_err(|_| ())?;

    let mut data = Vec::with_capacity(len as usize - id_bytes_read);
    stream.take(len as u64 - id_bytes_read as u64).read_to_end(&mut data).await.map_err(|_| ())?;
    Ok((len as usize, id, data))
}

pub async fn write_to_je(stream: &mut tokio::net::TcpStream, packet_id: i32, msgs: &[JeNetVal]) -> Result<usize, Box<dyn Error>> {
    let packet_id_varint = int_to_var_int(packet_id).unwrap();
    let len_msgs: i32 = msgs.iter().map(|e| e.size()).sum();
    let buf: Vec<u8> = [
        int_to_var_int(len_msgs + packet_id_varint.len() as i32).unwrap(),
        int_to_var_int(packet_id).unwrap(),
        msgs.iter().map(|e| e.to_vec_u8()).flatten().collect()
    ].iter().flatten().map(|e| *e).collect();
    stream.write(&buf).await.map_err(|e| e.into())
}

pub async fn je_kick(stream: &mut tokio::net::TcpStream, reason: Box<dyn Error>) {
    write_to_je(stream, 0x1b, &[
        JeNetVal::Chat(format!("{}", reason))
    ]).await;
}

#[derive(Debug, Clone)]
pub enum JeNetVal {
    Boolean(bool),
    Byte(i8),
    UByte(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Chat(String),
    Identifier(String),
    VarInt(i32),
    VarLong(i64),
    Array(Vec<u8>),
    FixedString(String, usize)
}

impl JeNetVal {
    pub fn size(&self) -> i32 {
        match self {
            JeNetVal::Boolean(_) | JeNetVal::Byte(_) | JeNetVal::UByte(_) => 1,
            JeNetVal::Short(_) | JeNetVal::UShort(_) => 2,
            JeNetVal::Int(_) | JeNetVal::Float(_) => 4,
            JeNetVal::Long(_) | JeNetVal::Double(_) => 8,
            JeNetVal::String(s) | JeNetVal::Chat(s) | JeNetVal::Identifier(s) => {
                int_to_var_int(s.len() as i32).unwrap().len() as i32 + (s.len() as i32)
            },
            JeNetVal::VarInt(vi) => int_to_var_int(*vi).unwrap().len() as i32,
            JeNetVal::VarLong(vl) => unimplemented!(),
            JeNetVal::Array(vec) => vec.len() as i32,
            JeNetVal::FixedString(s, l) => {
                int_to_var_int(s.len() as i32).unwrap().len() as i32 + *l as i32
            },
            _ => unimplemented!(),
        }
    }
    pub fn to_vec_u8(&self) -> Vec<u8> {
        match self {
            JeNetVal::VarInt(vi) => {
                int_to_var_int(*vi).unwrap()
            },
            JeNetVal::String(s) => {
                [int_to_var_int(s.len() as i32).unwrap(),
                s.as_bytes().to_owned()].iter().flatten().map(|e| *e).collect()
            },
            JeNetVal::Array(vec) => vec.clone(),
            JeNetVal::Float(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Double(v) => v.to_be_bytes().to_vec(),
            JeNetVal::UByte(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Byte(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Int(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Long(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Short(v) => v.to_be_bytes().to_vec(),
            JeNetVal::Boolean(v) => {
                if *v {[0b1].to_vec()} else {[0b0].to_vec()}
            },
            JeNetVal::FixedString(v, l) => {
                let mut s_w = v.as_bytes().to_owned();
                let pad_target = *l - s_w.len();
                info!("padding target {}", pad_target);
                s_w.truncate(*l);
                [
                    int_to_var_int(*l as i32).unwrap(),
                    if pad_target != 0 {
                        vec![0u8; pad_target]
                    } else {vec![]},
                    s_w
                ].iter().flatten().map(|e| *e).collect()
            }
            _ => unimplemented!()
        }
    }
    // TODO finish implementation
    pub fn try_from_je_type(data: &[u8], at: usize, type_hint: &JeNetType) -> Result<(Self, usize), ()> {
        let mut cursor = Cursor::new(data.split_at(at).1);
        match type_hint {
            JeNetType::Boolean => {
                match byteorder::ReadBytesExt::read_u8(&mut cursor) {
                    Ok(1) => Ok((JeNetVal::Boolean(true), type_hint.size().unwrap())),
                    Ok(0) => Ok((JeNetVal::Boolean(false), type_hint.size().unwrap())),
                    Ok(_) | Err(_) => Err(())
                }
            },
            JeNetType::Byte => {
                match byteorder::ReadBytesExt::read_i8(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Byte(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::UByte => {
                match byteorder::ReadBytesExt::read_u8(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::UByte(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::Short => {
                match byteorder::ReadBytesExt::read_i16::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Short(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::UShort => {
                match byteorder::ReadBytesExt::read_u16::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::UShort(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::Int => {
                match byteorder::ReadBytesExt::read_i32::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Int(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::Long => {
                match byteorder::ReadBytesExt::read_i64::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Long(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::Float => {
                match byteorder::ReadBytesExt::read_f32::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Float(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::Double => {
                match byteorder::ReadBytesExt::read_f64::<byteorder::NetworkEndian>(&mut cursor) {
                    Ok(val) => Ok((JeNetVal::Double(val), type_hint.size().unwrap())),
                    Err(_) => Err(())
                }
            },
            JeNetType::String => {
                // scan varint
                let (fp, sp) = data.split_at(at);
                let (str_len, varint_len) = var_int_to_int(sp, sp.len()).map_err(|_| ())?;
                cursor.seek(SeekFrom::Current(varint_len as i64)).map_err(|_| ())?;
                let mut try_string = Vec::new();
                let str_read = std::io::Read::take(cursor, str_len as u64).read_to_end(&mut try_string).map_err(|_| ())?;
                if str_read != str_len as usize {
                    return Err(());
                }
                //try_string.reverse();
                match String::from_utf8(try_string) {
                    Ok(string) => Ok((JeNetVal::String(string), varint_len + str_read)),
                    Err(_) => Err(())
                }
            },
            JeNetType::VarInt => {
                let (_, sp) = data.split_at(at);
                match var_int_to_int(sp, sp.len()) {
                    Ok((varint, read_len)) => {
                        Ok((JeNetVal::VarInt(varint), read_len))
                    },
                    Err(_) => Err(())
                }
            },
            o_t => {
                error!("Parsing {:?} not implemented yet", o_t);
                Err(())
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum JeNetType {
    Boolean,
    Byte,
    UByte,
    Short,
    UShort,
    Int,
    Long,
    Float,
    Double,
    String,
    Chat,
    Identifier,
    VarInt,
    VarLong,
    Array
}

impl JeNetType {
    pub fn size(&self) -> Option<usize> {
        match self {
            JeNetType::Boolean | JeNetType::Byte | JeNetType::UByte => Some(1),
            JeNetType::Short | JeNetType::UShort => Some(2),
            JeNetType::Int | JeNetType::Float => Some(4),
            JeNetType::Long | JeNetType::Double => Some(8),
            JeNetType::String | JeNetType::Chat | JeNetType::Identifier => None,
            JeNetType::VarInt | JeNetType::VarLong | JeNetType::Array => None
        }
    }
}

// TODO CHECK CHECK CHECK HUGE SECURITY RISK
/// Parses a je packet byte array to a rust format.
/// This function and its callees need special attention due to their security sensitive nature.
/// The iterator will terminate as soon as an `Err` is yield, which is intended.
pub fn parse_je_data(data_len: usize, data: &[u8], type_hints: &[JeNetType]) -> Result<Vec<JeNetVal>, JePacketError> {
    type_hints.iter().scan(0, |counter, type_hint| {
        Some(match JeNetVal::try_from_je_type(data, *counter, type_hint){
            Ok((val, len)) => {
                *counter += len;
                if *counter <= data_len {
                    Ok(val)
                } else {
                    Err(JePacketError::OversizedData(*counter, data_len))
                }
            },
            Err(_) => Err(JePacketError::InvalidFieldContent(*type_hint))
        })
    }).collect()
}

#[derive(Debug)]
pub enum JePacketError {
    OversizedData(usize, usize),
    UndersizedData(usize, usize),
    InvalidFieldContent(JeNetType)
}

/*impl TryFrom<&[u8]> for JePacketHandshake {
    type Error = Box<dyn Error>;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // TODO replace parse_je_data(0 <--
        /*let packet_maybe = parse_je_data(0, &packet_data, &[
            JeNetType::VarInt,
            JeNetType::String,
            JeNetType::UShort,
            JeNetType::VarInt
        ]);
        match fields_maybe {
            Ok(fields) => {
                if let JeNetVal::VarInt(protocol_ver) = fields[0] &&
                let JeNetVal::String(server_addr) = fields[1] &&
                let JeNetVal::UShort(server_port) = fields[2] &&
                let JeNetVal::VarInt(next_state) = fields[3] {
    
                }
            },
            Err(e) => {
    
            }
        }*/
        todo!()
    }
    
}*/