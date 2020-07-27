#[macro_use]
extern crate log;
extern crate env_logger;

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
use std::io::{BufRead, Write};
use env_logger::fmt::{Color, Style};
use log::Level;
use sysinfo::{DiskExt, NetworkExt, NetworksExt, ProcessorExt, SystemExt};
use config::{ConfigCollection, ConfigPerf};


fn main() {
    let start_inst = std::time::Instant::now();
    env_logger::builder()
    .filter_level(log::LevelFilter::Debug)
    .format(move |f, r| {
        let elasped = start_inst.elapsed();
        let mut style = f.style();
        style.set_bold(true);
        writeln!(f, "[{:12.6}] {} {}", elasped.as_secs_f32(),
        match r.level() {
            Level::Error => {
                style.set_color(Color::Red);
                style.value("E")
            }
            Level::Warn => {
                style.set_color(Color::Rgb(255, 153, 51));
                style.value("W")
            }
            Level::Info => {
                style.set_color(Color::Green);
                style.value("I")
            }
            Level::Debug => {
                style.set_color(Color::Blue);
                style.value("D")
            }
            Level::Trace => {
                style.set_color(Color::Magenta);
                style.value("T")
            }
        }, r.args())
    }).init();
    let opt = init_flags::InitFlags::from_args();
    debug!("raw opt {:?}", opt);

    let init = ServerInitializer(opt);
    let init_result = init.start();
    for info in &init_result.info {
        info!("{}", info);
    }
    for (warn, interaction) in &init_result.warn {
        warn!("{}", warn);
    }
    match &init_result.instance {
        Ok((gs, chans)) => {
            info!("Startup validation passed");
            info!(">>> CraftMine {}-{}", env!("CARGO_PKG_VERSION"), {
                if cfg!(debug_assertions) {
                    "DEBUG"
                } else {
                    "RELEASE"
                }
            });
            let (stdin_send, stdin_recv) = crossbeam::unbounded();
            let stdin = std::io::stdin();
            std::thread::spawn(move || {
                

                let mut stdin_lock = stdin.lock();
                for line in stdin_lock.lines() {
                    stdin_send.send(line.unwrap());
                }
            });
            
            loop {
                crossbeam::channel::select! {
                    recv(stdin_recv) -> in_line => {
                        debug!("stdin {}", in_line.unwrap());
                    },
                    recv(gs.cli_recv) -> gs_cli_msg => {

                    }
                }
            }
        },
        Err(errors) => {
            for err in errors {
                error!("E: {}", err);
            }
        }
    }
}

pub struct SrAllocator {
    pub total_threads: usize,
    pub sys: sysinfo::System,
    pub config_perf: ConfigPerf
}

impl SrAllocator {
    pub fn new(cc: &ConfigCollection) -> SrAllocator {
        Self {
            total_threads: num_cpus::get(),
            sys: sysinfo::System::new_all(),
            config_perf: cc.perf.clone()
        }
    }
    pub fn report(&self) {
        let proc = self.sys.get_global_processor_info();
        info!(">>> System Report");
        info!("| CPU: {} {}",
            proc.get_brand(),
            proc.get_name(),
        );
        info!("| Free Memory: {} kB", self.sys.get_free_memory());
        if self.total_threads <= 4 {
            warn!(">> 4 or less logical cores available ({}): \
                CraftMine loves cores, performance may suffer", self.total_threads);
        }
        if self.config_perf.items_dropped_ttl.as_secs() > 600 {
            warn!(">> config/performance: \
                items_dropped_ttl longer than 10 minutes ({}s)", self.config_perf.items_dropped_ttl.as_secs());
        }
        if let Some(stt) = self.config_perf.smp_threads_tick {
            if stt > (self.total_threads - 2) as u64 {
                warn!(">> config/performance: \
                    smp_threads_tick manually set ({}) higher than recommended value for system ({})",
                    stt,
                    (self.total_threads - 2)
                );
            }
        }
    }
    pub fn get_target_cores_dist(&self) -> (usize, usize) {
        todo!()
    }
}