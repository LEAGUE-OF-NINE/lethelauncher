use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use tracing::{error, info};
use tracing_subscriber;

#[derive(Deserialize)]
struct Config {
    paths: PathConfig,
}

#[derive(Deserialize)]
struct PathConfig {
    limbus: String,
    winhttp: String,
    renamed: String,
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("Config.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

fn check_file_exists(path: &Path) -> bool {
    if path.exists() {
        true
    } else {
        error!("{} not found.", path.display());
        false
    }
}

fn check_associated_files_exists(path_one: &Path, path_two: &Path) -> bool {
    if path_one.exists() || path_two.exists() {
        true
    } else {
        error!(
            "{} and {} not found.",
            path_one.display(),
            path_two.display()
        );
        false
    }
}

fn rename_file(from: &Path, to: &Path) -> io::Result<()> {
    if from.exists() && !to.exists() {
        fs::rename(from, to)?;
        info!("Renamed {} to {}", from.display(), to.display());
    } else if to.exists() {
        info!("No renaming needed: {} already exists.", to.display());
    } else {
        error!("Source file not found for renaming: {}", from.display());
    }
    Ok(())
}

fn prompt_exit() {
    info!("Press Enter to exit.");
    let _ = io::stdin().read_line(&mut String::with_capacity(0));
}

fn main() {
    tracing_subscriber::fmt().init();
    info!("Lethelauncher v0.1.0 starting...");
    info!("Reading Config.toml...");

    let config = match read_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to read Config.toml: {}", e);
            prompt_exit();
            return;
        }
    };

    let limbus_path = PathBuf::from(&config.paths.limbus);
    let winhttp_path = PathBuf::from(&config.paths.winhttp);
    let renamed_path = PathBuf::from(&config.paths.renamed);

    let limbus_ok = check_file_exists(&limbus_path);
    let dll_ok = check_associated_files_exists(&winhttp_path, &renamed_path);

    if !limbus_ok || !dll_ok {
        error!("Ensure all required files are present and paths are correct.");
        prompt_exit();
        return;
    }

    println!("Select an option to launch Limbus:");
    println!("[0] Mod disabled (rename winhttp.dll)");
    println!("[1] Mod enabled (restore winhttp.dll)");
    println!("[2] Exit launcher");
    print!("Enter your choice: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "0" => {
            if let Err(e) = rename_file(&winhttp_path, &renamed_path) {
                error!("Failed to rename winhttp.dll: {}", e);
                prompt_exit();
            }
        }
        "1" => {
            if let Err(e) = rename_file(&renamed_path, &winhttp_path) {
                error!("Failed to restore winhttp.dll: {}", e);
                prompt_exit();
            }
        }
        "2" => exit(0),
        _ => {
            error!("Invalid option selected");
            prompt_exit();
            return;
        }
    }

    if let Err(e) = Command::new(&limbus_path).spawn() {
        error!("Failed to launch LimbusCompany.exe: {}", e);
    } else {
        info!("Successfully launched {}", limbus_path.display());
    }
    prompt_exit();
}
