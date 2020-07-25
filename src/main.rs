use structopt::StructOpt;
use rayon::prelude::*;

mod common;
mod interop;
mod init_flags;
mod server;

pub use self::common::*;
pub use self::interop::*;
pub use self::init_flags::*;
pub use self::server::*;
use tokio::runtime;

fn main() {
    let opt = init_flags::InitFlags::from_args();
    println!("{:?}", opt);
    let init = ServerInitializer(opt);
    init.start();
}
