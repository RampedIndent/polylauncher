// use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use clap::Parser;
use log::{info, warn};
use rayon::prelude::*;
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use xshell::{cmd, Shell};

mod common;
use common::*;
// mod xrandr;

fn create_app_lock(port: u16) -> Result<TcpListener> {
    for _ in 0..3 {
        let sock = match TcpListener::bind(("127.0.0.1", port)) {
            Ok(socket) => Some(socket),
            Err(_) => {
                let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))?;
                stream.write_all(QUIT_COMMAND.as_bytes())?;
                drop(stream);
                std::thread::sleep(std::time::Duration::from_secs(1));
                None
            }
        };
        if let Some(sock) = sock {
            return Ok(sock);
        }
    }
    panic!("Couldn't lock port {port}: another instance already running?",);
}

fn main() -> Result<()> {
    let args = ArgsCli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let cfg: MyConfig = confy::load(APP_NAME, None).with_context(|| {
        format!(
            "Error reading the config file \nDefault path ~/.config/{APP_NAME}/default-config.yml"
        )
    })?;
    info!("{:?}", &cfg);
    let lock_socket = create_app_lock(12345)?;
    let lock_address = lock_socket.local_addr()?;

    ctrlc::set_handler(move || {
        warn!("received Ctrl+C!");

        let mut stream = TcpStream::connect(lock_address)
            .expect("Failed to connect to socket to terminate program");
        let _unused_result = stream.write(QUIT_COMMAND.as_bytes());
        drop(stream);
        std::thread::sleep(std::time::Duration::from_secs(10));
        std::process::exit(1)
    })
    .expect("Error setting Ctrl-C handler");

    let hostname: String = hostname::get()?.to_string_lossy().to_string();
    let monitors = IntoParallelIterator::into_par_iter(cfg.computers[&hostname].displays.clone());
    rayon::spawn(move || {
        monitors
            .filter(|monitor| !monitor.bar_name.is_empty())
            .try_for_each(|monitor| {
                let sh = Shell::new()?;
                let display = monitor.display;
                info!("{display:?}");
                let barname = monitor.bar_name;
                let config = &cfg.polybar_config;
                cmd!(sh, "polybar --reload {barname} -c {config}")
                    .env("MONITOR", display)
                    // .env("FC_DEBUG", "1")
                    .run()?;
                anyhow::Ok(())
            });
    });

    for stream in lock_socket.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = String::new();
                stream.read_to_string(&mut buf)?;
                match buf.as_str() {
                    QUIT_COMMAND => break,
                    _ => info!("Message recieved {buf:?}",),
                }
            }
            Err(e) => { /* connection failed */ }
        }
    }
    drop(lock_socket);
    Ok(())
}
