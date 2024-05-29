#![no_std]
#![no_main]

use catos::api::process::ExitCode;
use catos::api::syscall;
use catos::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
        } else {
            syscall::exit(ExitCode::DataError);
        }
    } else {
        syscall::exit(ExitCode::UsageError);
    }
}
