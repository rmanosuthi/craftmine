#[macro_use]
extern crate log;
extern crate env_logger;

mod common;
mod interop;
mod init_flags;
mod server;

pub mod imports {
    pub use sysinfo::{DiskExt, NetworkExt, NetworksExt, ProcessorExt, SystemExt};
    pub use std::{
        error::{Error},
        io::{BufRead, Cursor, Read, Write},
        net::{SocketAddr},
        path::{Path, PathBuf},
        pin::{Pin},
        time::{Duration, Instant}
    };
    pub use env_logger::fmt::{Color, Style};
    pub use structopt::StructOpt;
    pub use rayon::prelude::*;
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
    pub use hashbrown::HashMap;
    pub use tokio::prelude::*;
}

use crate::imports::*;
use crate::server::symbols::*;
use std::io::Write;
use log::Level;

fn main() {
    let start_inst = std::time::Instant::now();
    get_logger(start_inst);
    let opt = init_flags::InitFlags::from_args();
    debug!("raw opt {:?}", opt);

    let init = ServerInitializer(opt);
    let mut init_result = init.start();
    for info in &init_result.info {
        info!("{}", info);
    }
    for (warn, interaction) in &init_result.warn {
        warn!("{}", warn);
    }
    match init_result.instance {
        Ok((mut gs, mut chans)) => {
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
            let (ctrl_c_send, ctrl_c_recv) = crossbeam::bounded(1);
            ctrlc::set_handler(move || {
                ctrl_c_send.send(Instant::now());
            });
            std::thread::spawn(move || {

                let mut stdin_lock = stdin.lock();
                for line in stdin_lock.lines() {
                    stdin_send.send(line.unwrap());
                }
            });
            std::thread::spawn(move || {
                let mut gs = gs;
                gs.run();
            });
            
            loop {
                crossbeam::channel::select! {
                    recv(stdin_recv) -> in_line => {
                        debug!("stdin {}", in_line.unwrap());
                    },
                    recv(&chans.cli_recv) -> gs_cli_msg => {

                    },
                    recv(ctrl_c_recv) -> _ => {
                        info!("ctrl-c received, shutting down");
                        info!("Signalling shutdown...");
                        //std::process::exit(0);
                        chans.send_status.send(ServerStatus::Stop).unwrap();
                        let mut wait_game_done = true;
                        while wait_game_done {
                            if let Ok(ServerStatus::Stop) = chans.recv_status.recv() {
                                wait_game_done = false;
                            }
                        }
                        info!("Game server shut down. Terminating.");
                        std::process::exit(0);
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

#[cfg(debug_assertions)]
pub fn get_logger(start_inst: Instant) {
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
}

#[cfg(not(debug_assertions))]
pub fn get_logger(start_inst: Instant) {
    env_logger::builder()
    .filter_level(log::LevelFilter::Info)
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
}