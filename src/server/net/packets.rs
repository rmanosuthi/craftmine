use crate::imports::*;
use crate::server::symbols::*;

pub trait JePacket: Sized {
    fn get_packet_id() -> JeVarInt;
    fn try_from_raw(be_bytes: &[u8]) -> Result<Self, ()>;
    fn to_vec_u8(&self) -> Vec<u8>;
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
            fn get_packet_id() -> JeVarInt {
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
                            counter += bytes_read
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