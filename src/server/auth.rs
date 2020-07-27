use crate::server::net::*;
use std::error::Error;

// JE login process
/**
    C -> S 0x00 handshake
        protocol    varint
        addr        string(255)
        port        ushort
        nextstate   varint = 2
    C -> S 0x00 login
        name        string(16)
    S -> C 0x01 login
        serverid    string(20)  empty
        pubkey_len  varint
        pubkey      array
        vtoken_len  varint(4)
        vtoken      array
    c auth
    C -> S 0x01 login
        shared_secret_len   varint
        shared_secret       array
        vtoken_len          varint
        vtoken              array
    s auth
    enc enabled
    S -> C 0x03 login (set compression, optional)
        threshold           varint  (max size of packet before compressed)
    ***** COMP ENABLED S -> C 0x02 login
        uuid        string(36)
        username    string(16)
**/

/** Key generation
    rsa-priv -> rsa-keypair -> send der-rsa-pub
**/

/*pub async fn try_login(stream: &mut tokio::net::TcpStream, privkey: &[u8], pubkey: &[u8]) -> Result<JeSession, JeLoginError> {
    let (len, packet_id, data) = read_from_je(stream).await.map_err(|e| JeLoginError::Internal(e))?;
    if packet_id != 0x00 {
        return Err(JeLoginError::PacketError);
    }

    // TODO vtoken
    let vtoken = vec![0; 4];

    let username = jestring_to_string(&data);
    write_to_je(stream, 0x01, &[
        JeNetVal::String("".to_owned()),
        JeNetVal::VarInt(pubkey.len() as i32),
        JeNetVal::Array(pubkey.to_owned()),
        JeNetVal::VarInt(4),
        JeNetVal::Array(vtoken)
    ]).await.map_err(|e| JeLoginError::Internal(e))?;
    Ok(JeSession {})
}*/

pub enum JeLoginError {
    PacketError,
    Internal(Box<dyn Error>)
}

pub struct JeConnectionState {
    state: u8,
    enc: Option<JeSessionEncrypt>,
    status: JeConnectionStatus
}

impl Default for JeConnectionState {
    fn default() -> Self {
        Self {
            state: 0,
            enc: None,
            status: JeConnectionStatus::Unauthenticated
        }
    }
}

pub enum JeConnectionStatus {
    Unauthenticated,
    Offline {},
    Online
}

pub struct JeSessionEncrypt {}

pub fn jestring_to_string(data: &[u8]) -> String {
    todo!()
}