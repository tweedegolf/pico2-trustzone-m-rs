use clap::{Parser, Subcommand};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Parser)]
#[clap(name = "xtask")]
#[clap(about = "Build and flash tasks for the TZ-M HSM", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Build in release mode
    #[clap(long, global = true)]
    release: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build both secure and non-secure applications
    Build {},

    /// Build and flash both secure and non-secure applications
    /// (useful for debugging purposes)
    Flash {},

    /// Build, flash, and run the application
    Run {},
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Build {} => {
            println!("xtask: building secure and non-secure applications");
            xtask_build_all(args.release);
        }
        Commands::Flash {} => {
            println!("xtask: building and flashing both applications");
            xtask_build_all(args.release);
            xtask_flash_ns(args.release);
            xtask_flash_s(args.release);
        }
        Commands::Run {} => {
            println!("xtask: building, flashing, and running application");
            xtask_build_all(args.release);
            xtask_flash_ns(args.release);
            xtask_run_s(args.release);
        }
    }
}

const SECURE_APP: &str = "app-s";
const NON_SECURE_APP: &str = "app-ns";
const CHIP: &str = "RP235x";
const TARGET: &str = "thumbv8m.main-none-eabihf";

fn xtask_build_all(release: bool) {
    xtask_build_s(release);
    xtask_build_ns(release);
}

fn xtask_build_s(release: bool) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);

    command.current_dir(project_root().join(SECURE_APP));
    command.arg("build");
    if release {
        command.arg("--release");
    }
    command.args(["--package", SECURE_APP]);
    command.args(["--target", TARGET]);

    println!("xtask: executing {:?}", command);
    let status = command.status().unwrap();
    if !status.success() {
        eprintln!("xtask: cargo build failed for secure app with {}", status);
        process::exit(1);
    }
}

fn xtask_build_ns(release: bool) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);

    command.current_dir(project_root().join(NON_SECURE_APP));
    command.arg("build");
    if release {
        command.arg("--release");
    }
    command.args(["--package", NON_SECURE_APP]);
    command.args(["--target", TARGET]);

    println!("xtask: executing {:?}", command);
    let status = command.status().unwrap();
    if !status.success() {
        eprintln!(
            "xtask: cargo build failed for non-secure app with {}",
            status
        );
        process::exit(1);
    }
}

fn xtask_flash_ns(release: bool) {
    let mut command = Command::new("probe-rs");
    command.args(["download", "--chip", CHIP]);

    let path = if release {
        project_root()
            .join("target")
            .join(TARGET)
            .join("release")
            .join(NON_SECURE_APP)
    } else {
        project_root()
            .join("target")
            .join(TARGET)
            .join("debug")
            .join(NON_SECURE_APP)
    };
    command.arg(path);

    println!("xtask: executing {:?}", command);
    let status = command.status().unwrap();
    if !status.success() {
        eprintln!("xtask: probe-rs download failed with {}", status);
        process::exit(1);
    }
}

fn xtask_flash_s(release: bool) {
    let mut command = Command::new("probe-rs");
    command.args(["download", "--chip", CHIP]);

    let path = if release {
        project_root()
            .join("target")
            .join(TARGET)
            .join("release")
            .join(SECURE_APP)
    } else {
        project_root()
            .join("target")
            .join(TARGET)
            .join("debug")
            .join(SECURE_APP)
    };
    command.arg(path);

    println!("xtask: executing {:?}", command);
    let status = command.status().unwrap();
    if !status.success() {
        eprintln!("xtask: probe-rs download failed with {}", status);
        process::exit(1);
    }
}

fn xtask_run_s(release: bool) {
    let mut command = Command::new("probe-rs");
    command.args(["run", "--chip", CHIP]);

    let path = if release {
        project_root()
            .join("target")
            .join(TARGET)
            .join("release")
            .join(SECURE_APP)
    } else {
        project_root()
            .join("target")
            .join(TARGET)
            .join("debug")
            .join(SECURE_APP)
    };
    command.arg(path);

    println!("xtask: executing {:?}", command);
    let status = command.status().unwrap();
    if !status.success() {
        eprintln!("xtask: probe-rs run failed with {}", status);
        process::exit(1);
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
