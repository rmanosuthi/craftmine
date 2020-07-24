use uuid::Uuid;
use super::{int_to_var_int, var_int_to_int};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write, Cursor};

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

pub fn read_from_je(stream: &mut std::net::TcpStream) -> (usize, i32, Vec<u8>) {
    let mut len_peek = [0u8; 5];
    let len_peek_read = stream.peek(&mut len_peek).unwrap();
    let (len, len_bytes_read) = var_int_to_int(&mut len_peek, len_peek_read).unwrap();

    // advance stream by len_bytes_read
    stream.take(len_bytes_read as u64);

    let mut id_peek = [0u8; 5];
    let id_peek_read = stream.peek(&mut len_peek).unwrap();
    let (id, id_bytes_read) = var_int_to_int(&mut id_peek, id_peek_read).unwrap();

    // advance stream by id_bytes_read
    stream.take(id_bytes_read as u64);

    let mut data = Vec::with_capacity(len as usize - id_bytes_read);
    stream.take(len as u64 - id_bytes_read as u64).read_to_end(&mut data);
    (len as usize, id, data)
}

pub fn write_to_je(stream: &mut std::net::TcpStream, packet_id: i32, msgs: Vec<JeNetVal>) {
    let packet_id_varint = int_to_var_int(packet_id).unwrap();
    let len_msgs: i32 = msgs.iter().map(|e| e.size()).sum();
    let buf: Vec<u8> = [
        int_to_var_int(len_msgs + packet_id_varint.len() as i32).unwrap(),
        int_to_var_int(packet_id).unwrap(),
        msgs.iter().map(|e| e.to_vec_u8()).flatten().collect()
    ].iter().flatten().map(|e| *e).collect();
    println!("== WRITE TO JE ==");
    println!("{:?}", &buf);
    stream.write(&buf);
}

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
    Array(Vec<u8>)
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
            _ => unimplemented!()
        }
    }
}
