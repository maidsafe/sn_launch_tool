// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use directories::BaseDirs;
use log::debug;
use regex::Regex;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use structopt::StructOpt;

#[cfg(not(target_os = "windows"))]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault";

#[cfg(target_os = "windows")]
const SAFE_VAULT_EXECUTABLE: &str = "safe_vault.exe";

/// Tool to launch SAFE vaults to form a local single-section network
///
/// Currently, this tool runs vaults on localhost (since that's the default if no IP address is given to the vaults)
#[derive(StructOpt, Debug)]
#[structopt(name = "safe-nlt")]
struct CmdArgs {
    /// Verbosity level for this tool
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbosity: u8,

    /// Path where to locate safe_vault/safe_vault.exe binary. The SAFE_VAULT_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SAFE_VAULT_PATH")]
    vault_path: Option<PathBuf>,

    /// Interval in seconds between launching each of the vaults
    #[structopt(short = "i", long, default_value = "1")]
    interval: u64,

    /// Path where the output directories for all the vaults are written
    #[structopt(short = "d", long, default_value = "./vaults")]
    vaults_dir: PathBuf,

    /// Number of vaults to spawn with the first one being the genesis. This number should be greater than 0.
    #[structopt(short = "n", long, default_value = "8")]
    num_vaults: u8,

    /// Verbosity level for vaults logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    vaults_verbosity: u8,

    /// IP used to launch the vaults with.
    #[structopt(long = "ip")]
    ip: Option<String>,

    /// Run the section locally.
    #[structopt(long = "local")]
    is_local: bool,
}

/// Run a SAFE vault to join a network
#[derive(StructOpt, Debug)]
#[structopt(name = "safe-nlt-join")]
struct JoinCmdArgs {
    /// Verbosity level for this tool
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbosity: u8,

    /// Path where to locate safe_vault/safe_vault.exe binary. The SAFE_VAULT_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SAFE_VAULT_PATH")]
    vault_path: Option<PathBuf>,

    /// Path where to store the data for the running safe_vault
    #[structopt(short = "f", long, env = "SAFE_VAULT_DATA_PATH")]
    data_dir: Option<PathBuf>,

    /// Path where the output directories for all the vaults are written
    #[structopt(short = "d", long, default_value = "./vaults")]
    vaults_dir: PathBuf,

    /// Verbosity level for vaults logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    vaults_verbosity: u8,

    /// IP used to launch the vaults with.
    #[structopt(long = "ip")]
    ip: Option<String>,
}

pub fn run() -> Result<(), String> {
    run_with(None)
}

pub fn join() -> Result<(), String> {
    join_with(None)
}

pub fn join_with(cmd_args: Option<&[&str]>) -> Result<(), String> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = match cmd_args {
        None => JoinCmdArgs::from_args(),
        Some(cmd_args) => JoinCmdArgs::from_iter_safe(cmd_args).map_err(|err| err.to_string())?,
    };

    let vault_bin_path = get_vault_bin_path(args.vault_path)?;
    let msg = format!(
        "Launching with vault executable from: {}",
        vault_bin_path.display()
    );
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let mut common_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for vaults verbosity,
    // since the genesis vault logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.vaults_verbosity as usize));
    common_args.push(&verbosity);

    if let Some(ref data_dir) = args.data_dir {
        common_args.push("--data-dir");
        common_args.push(data_dir.to_str().unwrap());
    };

    if let Some(ref ip) = args.ip {
        let msg = format!("Network hardcoded contact: {}", ip);
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);

        // Construct current vault's command arguments
        let vault_dir = &args
            .vaults_dir
            .join("safe_vault_logs")
            .display()
            .to_string();

        let current_vault_args = build_vault_args(common_args.clone(), &vault_dir, Some(&ip));

        let msg = "Launching vault...";
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);
        run_vault_cmd(&vault_bin_path, &current_vault_args, args.verbosity)?;
    };
    Ok(())
}

pub fn run_with(cmd_args: Option<&[&str]>) -> Result<(), String> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = match cmd_args {
        None => CmdArgs::from_args(),
        Some(cmd_args) => CmdArgs::from_iter_safe(cmd_args).map_err(|err| err.to_string())?,
    };

    let vault_bin_path = get_vault_bin_path(args.vault_path)?;
    let msg = format!(
        "Launching with vault executable from: {}",
        vault_bin_path.display()
    );
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let msg = format!("Network size: {} vaults", args.num_vaults);
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let mut common_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for vaults verbosity,
    // since the genesis vault logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.vaults_verbosity as usize));
    common_args.push(&verbosity);

    if let Some(ref ip) = args.ip {
        common_args.push("--ip");
        common_args.push(ip);
    }

    if args.is_local {
        common_args.push("--local");
    }

    // Construct genesis vault's command arguments
    let genesis_vault_dir = &args.vaults_dir.join("safe-vault-genesis");
    let genesis_vault_dir_str = genesis_vault_dir.display().to_string();
    let genesis_vault_args = build_vault_args(
        common_args.clone(),
        &genesis_vault_dir_str,
        None, /* genesis */
    );

    // Let's launch genesis vault now
    let msg = "Launching genesis vault (#1)...";
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);
    run_vault_cmd(&vault_bin_path, &genesis_vault_args, args.verbosity)?;

    // Get port number of genesis vault to pass it as hard-coded contact to the other vaults
    let interval_duration = Duration::from_secs(args.interval);
    thread::sleep(interval_duration);
    let genesis_contact_info = grep_connection_info(&genesis_vault_dir.join("safe_vault.log"))?;
    let msg = format!("Genesis vault contact info: {}", genesis_contact_info);
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    // We can now run the rest of the vaults
    for i in 2..args.num_vaults + 1 {
        // Construct current vault's command arguments
        let vault_dir = &args
            .vaults_dir
            .join(&format!("safe-vault-{}", i))
            .display()
            .to_string();

        let current_vault_args =
            build_vault_args(common_args.clone(), &vault_dir, Some(&genesis_contact_info));

        let msg = format!("Launching vault #{}...", i);
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);
        run_vault_cmd(&vault_bin_path, &current_vault_args, args.verbosity)?;

        // We wait for a few secs before launching each new vault
        thread::sleep(interval_duration);
    }

    println!("Done!");
    Ok(())
}

#[inline]
fn get_vault_bin_path(vault_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match vault_path {
        Some(p) => Ok(p),
        None => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| "Failed to obtain user's home path".to_string())?;

            let mut path = PathBuf::from(base_dirs.home_dir());
            path.push(".safe");
            path.push("vault");
            path.push(SAFE_VAULT_EXECUTABLE);
            Ok(path)
        }
    }
}

fn build_vault_args<'a>(
    mut base_args: Vec<&'a str>,
    vault_dir: &'a str,
    contact_info: Option<&'a str>,
) -> Vec<&'a str> {
    if let Some(contact) = contact_info {
        base_args.push("--hard-coded-contacts");
        base_args.push(contact);
    } else {
        base_args.push("--first");
    }

    base_args.push("--root-dir");
    base_args.push(vault_dir);
    base_args.push("--log-dir");
    base_args.push(vault_dir);

    base_args
}

fn run_vault_cmd(vault_path: &PathBuf, args: &[&str], verbosity: u8) -> Result<(), String> {
    let path_str = vault_path.display().to_string();
    let msg = format!("Running '{}' with args {:?} ...", path_str, args);
    if verbosity > 1 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let _child = Command::new(&path_str)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| {
            format!(
                "Failed to run '{}' with args '{:?}': {}",
                path_str, args, err
            )
        })?;

    Ok(())
}

fn grep_connection_info(log_path: &PathBuf) -> Result<String, String> {
    let regex_query = Regex::new(r".+Vault connection info:\s(.+)$").map_err(|err| {
        format!(
            "Failed to obtain the contact info of the genesis vault: {}",
            err
        )
    })?;
    let file_content = fs::read_to_string(log_path).map_err(|err| {
        format!(
            "Failed to obtain the contact info of the genesis vault: {}",
            err
        )
    })?;

    for (_, line) in file_content.lines().enumerate() {
        if let Some(contact_info) = &regex_query.captures(&line) {
            return Ok(format!("[{}]", contact_info[1].to_string()));
        }
    }

    Err("Failed to find the contact info of the genesis vault".to_string())
}
