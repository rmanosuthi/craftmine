use super::JeNetVal;
use std::any::Any;
use serde::{Serialize, Deserialize};

pub trait JePacket: Sized {
    fn try_raw(from: &[JeNetVal]) -> Result<Self, ()>;
}

/// Declare packet macro.
/// Extremely inefficient, makes unnecessary copies, but without the copies it doesn't work.
macro_rules! declare_packet {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl JePacket for $name {
            fn try_raw(from: &[JeNetVal]) -> Result<Self, ()> {
                // TODO verify this isn't a disaster waiting to happen
                let mut result: $name = unsafe {
                    std::mem::MaybeUninit::zeroed().assume_init()
                };
                let mut counter = 0;
                    $(
                        match from.get(counter).unwrap().to_owned() {
                            JeNetVal::Boolean(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Byte(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::UByte(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Short(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::UShort(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Int(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Long(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Float(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Double(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::String(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Chat(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Identifier(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::VarInt(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::VarLong(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            JeNetVal::Array(b) => {
                                let b = &b as &dyn Any;
                                result.$field_name = b.downcast_ref::<$field_type>().unwrap().to_owned();
                            },
                            _ => unimplemented!()
                        }
                        counter += 1;
                    )*
                Ok(result)
            }
        }
    };
}

declare_packet!(struct JePacketHandshake {
    protocol_ver: i32,
    server_addr: String,
    server_port: u16,
    next_state: i32,
});

declare_packet!(struct JeLoginStart {
    name: String,
});