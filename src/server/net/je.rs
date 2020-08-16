use crate::{server::net::legacy, imports::*, server::symbols::*};
use uuid::Uuid;
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
    let (len, len_bytes_read) = match legacy::var_int_to_int(&mut len_peek, len_peek_read) {
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
    let (id, id_bytes_read) = legacy::var_int_to_int(&mut id_peek, id_peek_read).map_err(|_| ())?;

    // advance stream by id_bytes_read
    stream.take(id_bytes_read as u64).read(&mut id_peek).await.map_err(|_| ())?;

    let mut data = Vec::with_capacity(len as usize - id_bytes_read);
    stream.take(len as u64 - id_bytes_read as u64).read_to_end(&mut data).await.map_err(|_| ())?;
    Ok((len as usize, id, data))
}

pub async fn write_to_je_raw(stream: &mut tokio::net::TcpStream, packet_id: i32, data: &[u8]) -> Result<usize, Box<dyn Error>> {
    let packet_id_varint = legacy::int_to_var_int(packet_id);
    let len_msgs: i32 = data.len() as i32;
    let buf: Vec<u8> = [
        legacy::int_to_var_int(len_msgs + packet_id_varint.len() as i32),
        legacy::int_to_var_int(packet_id),
        data.to_owned()
    ].iter().flatten().map(|e| *e).collect();
    stream.write(&buf).await.map_err(|e| e.into())
}