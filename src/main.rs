#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use structopt::StructOpt;
use rayon::prelude::*;

mod common;
mod interop;
mod init_flags;
mod server;

pub(crate) use self::common::*;
pub(crate) use self::interop::*;
pub(crate) use self::init_flags::*;
pub(crate) use self::server::*;

fn main() {
    let opt = init_flags::InitFlags::from_args();
    println!("{:?}", opt);
}
