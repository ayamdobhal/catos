use crate::api::console::Style;
use crate::api::fs;
use crate::api::fs::DeviceType;
use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::{api, sys, usr};

use alloc::format;
use alloc::string::String;

pub fn copy_files(verbose: bool) {
    create_dir("/bin", verbose); // Binaries
    create_dir("/dev", verbose); // Devices
    create_dir("/ini", verbose); // Initializers
    create_dir("/lib", verbose); // Libraries
    create_dir("/net", verbose); // Network
    create_dir("/src", verbose); // Sources
    create_dir("/tmp", verbose); // Temporaries
    create_dir("/usr", verbose); // User directories
    create_dir("/var", verbose); // Variables

    copy_file("/bin/clear", include_bytes!("../../dsk/bin/clear"), verbose);
    copy_file("/bin/halt", include_bytes!("../../dsk/bin/halt"), verbose);
    copy_file("/bin/print", include_bytes!("../../dsk/bin/print"), verbose);

    create_dir("/dev/ata", verbose); // Drives
    create_dir("/dev/ata/0", verbose);
    create_dev("/dev/ata/0/0", DeviceType::Drive, verbose);
    create_dev("/dev/ata/0/1", DeviceType::Drive, verbose);
    create_dir("/dev/ata/1", verbose);
    create_dev("/dev/ata/1/0", DeviceType::Drive, verbose);
    create_dev("/dev/ata/1/1", DeviceType::Drive, verbose);
    create_dir("/dev/clk", verbose); // Clock
    create_dev("/dev/clk/uptime", DeviceType::Uptime, verbose);
    create_dev("/dev/clk/realtime", DeviceType::Realtime, verbose);
    create_dev("/dev/rtc", DeviceType::RTC, verbose);
    create_dev("/dev/null", DeviceType::Null, verbose);
    create_dev("/dev/random", DeviceType::Random, verbose);
    create_dev("/dev/console", DeviceType::Console, verbose);
    create_dir("/dev/net", verbose); // Network
    create_dev("/dev/net/tcp", DeviceType::TcpSocket, verbose);
    create_dev("/dev/net/udp", DeviceType::UdpSocket, verbose);

    copy_file(
        "/ini/banner.txt",
        include_bytes!("../../dsk/ini/banner.txt"),
        verbose,
    );
    copy_file(
        "/ini/boot.sh",
        include_bytes!("../../dsk/ini/boot.sh"),
        verbose,
    );
    copy_file(
        "/ini/shell.sh",
        include_bytes!("../../dsk/ini/shell.sh"),
        verbose,
    );
    copy_file(
        "/ini/version.txt",
        include_bytes!("../../dsk/ini/version.txt"),
        verbose,
    );

    create_dir("/ini/palettes", verbose);
    copy_file(
        "/ini/palettes/gruvbox-dark.sh",
        include_bytes!("../../dsk/ini/palettes/gruvbox-dark.sh"),
        verbose,
    );
    copy_file(
        "/ini/palettes/gruvbox-light.sh",
        include_bytes!("../../dsk/ini/palettes/gruvbox-light.sh"),
        verbose,
    );

    create_dir("/ini/fonts", verbose);
    copy_file(
        "/ini/fonts/zap-light-8x16.psf",
        include_bytes!("../../dsk/ini/fonts/zap-light-8x16.psf"),
        verbose,
    );
    copy_file(
        "/ini/fonts/zap-vga-8x16.psf",
        include_bytes!("../../dsk/ini/fonts/zap-vga-8x16.psf"),
        verbose,
    );

    copy_file(
        "/tmp/alice.txt",
        include_bytes!("../../dsk/tmp/alice.txt"),
        verbose,
    );
    copy_file(
        "/tmp/machines.txt",
        include_bytes!("../../dsk/tmp/machines.txt"),
        verbose,
    );

    create_dir("/tmp/beep", verbose);
    copy_file(
        "/tmp/beep/tetris.sh",
        include_bytes!("../../dsk/tmp/beep/tetris.sh"),
        verbose,
    );
    copy_file(
        "/tmp/beep/starwars.sh",
        include_bytes!("../../dsk/tmp/beep/starwars.sh"),
        verbose,
    );
    copy_file(
        "/tmp/beep/mario.sh",
        include_bytes!("../../dsk/tmp/beep/mario.sh"),
        verbose,
    );
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Welcome to CATOS v{} installation program!{}",
        csi_color,
        env!("CARGO_PKG_VERSION"),
        csi_reset
    );
    println!();

    let mut has_confirmed = false;
    for &arg in args {
        match arg {
            "-y" | "--yes" => has_confirmed = true,
            _ => continue,
        }
    }
    if !has_confirmed {
        print!("Proceed? [y/N] ");
        has_confirmed = io::stdin().read_line().trim() == "y";
        println!();
    }

    if has_confirmed {
        if !sys::fs::is_mounted() {
            println!("{}Listing disks ...{}", csi_color, csi_reset);
            usr::shell::exec("disk list").ok();
            println!("/dev/mem        RAM DISK");
            println!();

            println!("{}Formatting disk ...{}", csi_color, csi_reset);
            print!("Enter path of disk to format: ");
            let path = io::stdin().read_line();
            if path.trim_end() == "/dev/mem" {
                usr::shell::exec(&format!("memory format"))?;
            } else {
                usr::shell::exec(&format!("disk format {}", path.trim_end()))?;
            }
            println!();
        }

        println!("{}Populating filesystem...{}", csi_color, csi_reset);
        let verbose = true;
        copy_files(verbose);

        if sys::process::user().is_none() {
            println!();
            println!("{}Creating user...{}", csi_color, csi_reset);
            let res = usr::user::main(&["user", "create"]);
            if res == Err(ExitCode::Failure) {
                return res;
            }
        }

        println!();
        println!("{}Installation successful!{}", csi_color, csi_reset);
        println!();
        println!("Quit the console or reboot to apply changes");
    }

    Ok(())
}

fn create_dir(pathname: &str, verbose: bool) {
    if syscall::info(pathname).is_none() {
        if let Some(handle) = api::fs::create_dir(pathname) {
            syscall::close(handle);
            if verbose {
                println!("Created '{}'", pathname);
            }
        }
    }
}

fn create_dev(pathname: &str, dev: DeviceType, verbose: bool) {
    if syscall::info(pathname).is_none() {
        let mut buf = dev.buf();
        // NOTE: The first byte of `buf` contains the device type
        match pathname {
            "/dev/ata/0/0" => {
                buf[1] = 0;
                buf[2] = 0
            }
            "/dev/ata/0/1" => {
                buf[1] = 0;
                buf[2] = 1
            }
            "/dev/ata/1/0" => {
                buf[1] = 1;
                buf[2] = 0
            }
            "/dev/ata/1/1" => {
                buf[1] = 1;
                buf[2] = 1
            }
            _ => {}
        }
        if let Some(handle) = fs::create_device(pathname, &buf) {
            syscall::close(handle);
            if verbose {
                println!("Created '{}'", pathname);
            }
        }
    }
}

fn copy_file(pathname: &str, buf: &[u8], verbose: bool) {
    if fs::exists(pathname) {
        return;
    }
    if pathname.ends_with(".txt") {
        if let Ok(text) = String::from_utf8(buf.to_vec()) {
            let text = text.replace("{x.x.x}", env!("CARGO_PKG_VERSION"));
            fs::write(pathname, text.as_bytes()).ok();
        } else {
            fs::write(pathname, buf).ok();
        }
    } else {
        fs::write(pathname, buf).ok();
    }
    // TODO: add File::write_all to split buf if needed
    if verbose {
        println!("Copied '{}'", pathname);
    }
}
