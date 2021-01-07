use std::ops::Deref;

mod auth;
mod io;

mod net {
    mod je;
    pub mod legacy;
    mod msg;
    mod packets;
    mod server;
    mod types;
    pub use self::je::*;
    pub use self::msg::*;
    pub use self::packets::*;
    pub use self::server::*;
    pub use self::types::*;
}

mod world {
    mod components;
    mod folder;
    mod generator;
    pub use self::components::*;
    pub use self::folder::*;
    pub use self::generator::*;
}

mod game {
    mod mode;
    mod server;
    pub use self::mode::*;
    pub use self::server::*;
}

mod init;

mod prefix;

pub mod symbols {
    pub use super::net::*;
    pub use super::world::*;
    pub use super::game::*;
    pub use super::init::*;
    pub use super::prefix::*;
    pub use super::config::*;
    pub use super::records::*;
    pub use super::auth::*;

    pub type InteractionNeeded = bool;
}

mod config;

mod records;

pub mod log;

/// A type which contains data loaded from the program memory.
/// The name is slightly misleading and is only a weak guarantee; `Live` does *not* take ownership.
/// It's only meant to be used in a function signature, and can be an out-of-sync clone of the actual object.
pub struct Live<T: Clone> {
    inner: T
}

impl<T: Clone> Live<T> {
    pub fn snapshot(&self) -> Snapshot<T> {
        Snapshot {inner: self.inner.clone()}
    }
}

impl<T: Clone> From<T> for Live<T> {
    fn from(t: T) -> Live<T> {
        Live {inner: t}
    }
}

impl<T: Clone> Deref for Live<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A type which contains data loaded from the the disk, or a copy of `Live<T>`.
/// It's only meant to be used in a function signature, and can be an out-of-sync clone of the actual object.
pub struct Snapshot<T> {
    inner: T
}

impl<T> Deref for Snapshot<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> From<T> for Snapshot<T> {
    fn from(t: T) -> Snapshot<T> {
        Snapshot {inner: t}
    }
}