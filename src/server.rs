use hyper_native_tls::NativeTlsServer;
use iron::{Chain, Iron, Listening};

use crate::signal_manager;
use crate::staticfile_middleware::HttpToHttpsRedirect;
use crate::staticfiles::*;
use crate::{config::Options, logger};

/// Struct for holding a reference to a running iron server instance
#[derive(Debug)]
struct RunningServer {
    listening: Listening,
    server_type: String,
}

fn on_server_running(server_name: &str, running_servers: &[RunningServer]) {
    // Notify when server is running
    running_servers.iter().for_each(|server| {
        logger::log_server(&format!(
            "{} Server ({}) is listening on {}",
            server.server_type, server_name, server.listening.socket
        ))
    });

    // Wait for incoming signals (E.g Ctrl+C (SIGINT), SIGTERM, etc
    signal_manager::wait_for_signal(|sig: signal::Signal| {
        let code = signal_manager::signal_to_int(sig);

        println!();
        warn!("Signal {} caught. Server execution exited.", code);
        std::process::exit(code)
    })
}

pub fn run(opts: Options) {
    logger::init(&opts.log_level);

    let addr = &format!("{}{}{}", opts.host, ":", opts.port);

    // Configure & launch the HTTP server

    let files = StaticFiles::new(StaticFilesOptions {
        root_dir: opts.root,
        assets_dir: opts.assets,
        page_50x_path: opts.page50x,
        page_404_path: opts.page404,
        cors_allow_origins: opts.cors_allow_origins,
    });

    let mut running_servers = Vec::new();
    if opts.tls {
        // Launch static HTTPS server
        let ssl = NativeTlsServer::new(opts.tls_pkcs12, &opts.tls_pkcs12_passwd).unwrap();

        match Iron::new(files.handle()).https(addr, ssl) {
            Ok(listening) => running_servers.push(RunningServer {
                listening,
                server_type: "HTTPS".to_string(),
            }),
            Err(err) => panic!("{:?}", err),
        }

        // Launch redirect HTTP server (if requested)
        if let Some(port_redirect) = opts.tls_redirect_from {
            let addr_redirect = &format!("{}{}{}", opts.host, ":", port_redirect);
            let host_redirect = match opts.tls_redirect_host.as_ref() {
                Some(host) => host,
                None => &opts.host,
            };
            let handler =
                Chain::new(HttpToHttpsRedirect::new(&host_redirect, opts.port).permanent());
            match Iron::new(handler).http(addr_redirect) {
                Ok(listening) => running_servers.push(RunningServer {
                    listening,
                    server_type: "Redirect HTTP".to_string(),
                }),
                Err(err) => panic!("{:?}", err),
            }
        }
    } else {
        // Launch static HTTP server
        match Iron::new(files.handle()).http(addr) {
            Ok(listening) => running_servers.push(RunningServer {
                listening,
                server_type: "HTTP".to_string(),
            }),
            Err(err) => panic!("{:?}", err),
        }
    }
    on_server_running(&opts.name, &running_servers);
}
