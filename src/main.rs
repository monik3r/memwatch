use clap::Arg;
use clap::Command as ClapCommand;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::System;

fn check_free_memory(threshold: u64) -> bool {
    let mut system = System::new();
    system.refresh_memory();
    let free_memory = system.free_memory();
    free_memory < threshold * 1024 * 1024 * 1024 
}

fn main() {
    let matches = ClapCommand::new("memwatch")
        .about("Runs a command and ends it if free memory goes below a threshold.")
        .arg(
            Arg::new("threshold")
                .short('g')
                .long("threshold")
                .value_name("GB")
                .default_value("1")
                .help("Free memory threshold in gigabytes")
                .value_parser(clap::value_parser!(u64)), // Parse threshold as u64
        )
        .arg(
            Arg::new("command")
                .required(true)
                .num_args(1..)
                .help("Command to run with its arguments"),
        )
        .get_matches();

    let threshold: u64 = *matches.get_one("threshold").unwrap();
    let command_args: Vec<&String> = matches
        .get_many::<String>("command")
        .unwrap()
        .collect();

    let command = &command_args[0];
    let command_args = &command_args[1..];

    let mut child = match Command::new(command)
        .args(command_args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("Failed to start process: {}", err);
            std::process::exit(1);
        }
    };

    loop {
        if let Some(status) = child.try_wait().unwrap() {
            println!("Child exited with status: {}", status);
            break;
        }

        if check_free_memory(threshold) {
            println!("Free memory below {} GB. Sending SIGINT to the child", threshold);
            #[cfg(unix)]
            unsafe {
                libc::kill(child.id() as i32, libc::SIGINT);
            }

            #[cfg(windows)]
            {
                child.kill().expect("Failed to kill the child");
            }
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }

    if let Err(err) = child.wait() {
        eprintln!("Error waiting for child process: {}", err);
    }
}

