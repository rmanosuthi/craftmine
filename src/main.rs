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
use tokio::runtime;

fn main() {
    let opt = init_flags::InitFlags::from_args();
    println!("{:?}", opt);
    let threaded_rt = runtime::Builder::new()
    .threaded_scheduler()
    .build().unwrap();
    let init = ServerInitializer(opt);
    init.start();
}
