use crate::imports::*;
use crate::server::symbols::*;
use crate::init_flags::*;
use std::net::TcpListener;
use crate::SrAllocator;

pub struct ServerInitializer(pub InitFlags);

impl ServerInitializer {
    pub fn start(&self) -> ServerInitResult {
        let flags = &self.0;
        let mut infos = Vec::new();
        let mut warns = Vec::new();
        let mut errs = Vec::new();

        // TODO Check init flags valid
        let validated_flags_maybe = flags.to_validated();

        if let Err(mut e) = validated_flags_maybe.clone() {
            errs.append(&mut e);
        }

        if errs.is_empty() {
            // can safely unwrap validated_flags
            let mut validated_flags = validated_flags_maybe.unwrap().clone();

            info!("Startup args validated");
            debug!("Validated flags {:?}", &validated_flags);

            infos.append(&mut validated_flags.prefix.1);
            infos.append(&mut validated_flags.config_path.1);
            infos.append(&mut validated_flags.je_port.1);
            infos.append(&mut validated_flags.be_port.1);
            infos.append(&mut validated_flags.bind_addr.1);

            let mut prefix_errors = validated_flags.try_create_prefix();

            if prefix_errors.len() != 0 {
                errs.append(&mut prefix_errors);
            }

            // Check ports bindable
            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.je_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to JE port {}", validated_flags.je_port.0)
                )
            } else {
                infos.push(format!("JE: port {} available", validated_flags.je_port.0));
            }

            if let Err(_) = TcpListener::bind(
                format!("127.0.0.1:{}", validated_flags.be_port.0)
            ) {
                errs.push(
                    format!("Failed to bind to BE port {}", validated_flags.be_port.0)
                )
            } else {
                infos.push(format!("BE: port {} available", validated_flags.be_port.0));
            }

            // Check configs exist and valid
            let cc_maybe = match ConfigFolder::load_or_new_all(&validated_flags) {
                Ok(cc) => Some(cc),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            // Check worlds
            let worlds_maybe = match WorldFolder::load_or_default(&validated_flags) {
                Ok(worlds) => Some(worlds),
                Err(errors) => {
                    for e in errors {
                        errs.push(e);
                    }
                    None
                }
            };

            if errs.is_empty() {
                let cc = cc_maybe.unwrap();
                let async_net_instance = NetServer::new(validated_flags.clone(), cc.clone());

                let (cli_send, gs_cli_recv) = crossbeam::unbounded();
                let (gs_cli_send, cli_recv) = crossbeam::unbounded();

                let web_ws = cc.net.web_addr_port.clone();

                let sra = SrAllocator::new(&cc);
                sra.report();

                let (status_to_gs, recv_status_to_gs) = crossbeam::bounded(5);
                let (status_from_gs, recv_status_from_gs) = crossbeam::bounded(5);
                let (pfx, pfx_info) = ServerPrefix::load_or_new(&validated_flags.prefix.0);

                for (path, maybe_error) in pfx_info {
                    debug!("prefix into {:?} {:?}", path, maybe_error);
                }

                let instance = GameServer {
                    prefix: pfx,
                    init_flags: validated_flags.clone(),
                    worlds: worlds_maybe.unwrap(),
                    cli_recv: gs_cli_recv,
                    cli_send: gs_cli_send,
                    async_net_instance,
                    tick: crossbeam::tick(Duration::from_secs_f64(cc.perf.target_tick_s_f64)),
                    cc,
                    recv_status: recv_status_to_gs,
                    send_status: status_from_gs,
                    users: HashMap::new()
                };
                ServerInitResult {
                    instance: if errs.is_empty() {
                        Ok((instance, ServerInitChannels {
                            cli_recv: cli_recv,
                            cli_send: cli_send,
                            web_ws,
                            send_status: status_to_gs,
                            recv_status: recv_status_from_gs
                        }))
                    } else {
                        Err(errs)
                    },
                    warn: warns,
                    info: infos
                }
            } else {
                ServerInitResult {
                    instance: Err(errs),
                    warn: warns,
                    info: infos
                }
            }
        } else {
            // early exit because can't do much without proper init flags
            ServerInitResult {
                instance: Err(errs),
                warn: warns,
                info: infos
            }
        }
    }
}

pub struct ServerInitResult {
    pub instance: Result<(GameServer, ServerInitChannels), Vec<String>>,
    pub warn: Vec<(String, InteractionNeeded)>,
    pub info: Vec<String>
}

pub struct ServerInitChannels {
    pub cli_recv: crossbeam::Receiver<String>,
    pub cli_send: crossbeam::Sender<String>,
    pub web_ws: String,
    pub send_status: crossbeam::Sender<ServerStatus>,
    pub recv_status: crossbeam::Receiver<ServerStatus>
}

#[derive(Debug)]
pub enum ServerStatus {
    Start,
    Stop,
    Pause
}

#[derive(Debug)]
pub enum ServerInitWarn {
    NoConfig,
    EmptyPrefix,
    UsingCurrentDir
}

impl ServerInitWarn {
    pub fn to_kv(self) -> (ServerInitWarn, InteractionNeeded) {
        match self {
            ServerInitWarn::NoConfig => (self, false),
            _ => unimplemented!()
        }
    }
}