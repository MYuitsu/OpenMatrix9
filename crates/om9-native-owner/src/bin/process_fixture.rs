// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg_attr(windows, windows_subsystem = "windows")]
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("echo") => {
            print!("{}", args[1..].join("|"));
            eprint!("fixture diagnostics");
        }
        Some("hold") => loop {
            thread::sleep(Duration::from_secs(1));
        },
        Some("stdout-flood" | "stderr-flood") => {
            let mut output: Box<dyn Write> = if args[0] == "stdout-flood" {
                Box::new(io::stdout())
            } else {
                Box::new(io::stderr())
            };
            loop {
                if output.write_all(&[b'x'; 8192]).is_err() {
                    break;
                }
                let _ = output.flush();
            }
        }
        Some("allocate") => {
            let mut allocations = vec![];
            loop {
                let mut data = Vec::new();
                if data.try_reserve_exact(2 * 1024 * 1024).is_err() {
                    std::process::exit(86);
                }
                data.resize(2 * 1024 * 1024, 42u8);
                std::hint::black_box(&data);
                allocations.push(data);
            }
        }
        Some("spawn-child") => {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command.arg("hold");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }
            match command.spawn() {
                Err(_) => print!("child creation rejected"),
                Ok(mut child) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    std::process::exit(91);
                }
            }
        }
        Some("fail") => {
            print!("invalid partial result");
            eprint!("fixture failure");
            std::process::exit(42);
        }
        _ => std::process::exit(2),
    }
}
