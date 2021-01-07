use crate::imports::*;
use crate::server::symbols::*;
use async_trait::async_trait;
use futures::TryFutureExt;

#[async_trait]
pub trait JePacket: Sized {
    fn get_packet_id(&self) -> JeVarInt;
    fn try_from_raw(be_bytes: &[u8]) -> Result<Self, ()>;
    fn to_vec_u8(&self) -> Vec<u8>;
    async fn write_to_stream(&self, stream: &mut tokio::net::TcpStream) -> Result<usize, ()> {
        write_to_je_raw(stream, self.get_packet_id().0, &self.to_vec_u8())
            .map_err(|_| ()).await
    }
}

pub struct JeGenericPacket {
    pub id: i32,
    pub data: Vec<u8>
}

/// Make a struct a packet.
/// Field types must implement `JeType` and `Default` and end with a `,` in the declaration.
macro_rules! declare_packet {
    ($packet_id:expr, struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        #[derive(Default)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }
        impl JePacket for $name {
            fn get_packet_id(&self) -> JeVarInt {
                JeVarInt($packet_id)
            }
            fn try_from_raw(be_bytes: &[u8]) -> Result<Self, ()> {
                let mut result = Self::default();
                let mut counter = 0;
                $(
                    match <$field_type>::try_from_raw(be_bytes.split_at(counter).1) {
                        Ok((v, bytes_read)) => {
                            debug!("DECODE {:?} OK", std::any::type_name::<$field_type>());
                            result.$field_name = v;
                            counter += bytes_read;
                        },
                        Err(_) => {
                            debug!("DECODE {:?} ERR", std::any::type_name::<$field_type>());
                            return Err(());
                        }
                    }
                )*
                Ok(result)
            }
            fn to_vec_u8(&self) -> Vec<u8> {
                let mut result = Vec::with_capacity(200);
                $(
                    for b in self.$field_name.to_vec_u8() {
                        result.push(b);
                    }
                )*
                result
            }
        }
        impl std::convert::From<$name> for JeGenericPacket {
            fn from(input: $name) -> Self {
                unimplemented!()
            }
        }
    };
}

declare_packet!(0x00, struct JePacketHandshake {
    protocol_ver: JeVarInt,
    server_addr: String,
    server_port: u16,
    next_state: JeVarInt,
});

declare_packet!(0x01, struct JeLoginStart {
    name: String,
});

declare_packet!(0x00, struct JeHandshakeResponse {
    json: String,
});

declare_packet!(0x00, struct JeLoginDisconnect {
    reason: JeChat,
});

declare_packet!(0x1b, struct JePlayDisconnect {
    reason: JeChat,
});

declare_packet!(0x01, struct JePacketPing {
    val: i64,
});

declare_packet!(0x01, struct JePacketPong {
    val: i64,
});

declare_packet!(0x01, struct JeEncRequest {
    server_id: String,
    pubkey_len: JeVarInt,
    pubkey: Vec<u8>,
    vtoken_len: JeVarInt,
    vtoken: Vec<u8>,
});

declare_packet!(0x02, struct JeLoginSuccess {
    uuid: String,
    username: String,
});

declare_packet!(0x26, struct JeJoinGame {
    entity_id: i32,
    gamemode: u8,
    dimension: i32,
    hashed_seed: i64,
    max_players: u8,
    level_type: JeLevelType,
    view_distance: JeVarInt,
    reduced_debug_info: bool,
    enable_respawn_screen: bool,
});

// TODO
declare_packet!(0x22, struct JeChunk {});