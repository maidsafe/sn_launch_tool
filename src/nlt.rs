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
use std::{
    /*io::{self, Write},*/
    path::PathBuf,
    process::{Command, Stdio},
    thread, time,
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
struct Opt {
    /// Path where to locate safe_vault/safe_vault.exe binary. The SAFE_VAULT_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SAFE_VAULT_PATH")]
    vault_path: Option<PathBuf>,

    /// Interval in seconds between launching each of the vaults
    #[structopt(short = "i", long, default_value = "5")]
    interval: u64,

    /// Path where the output directories for all the vaults are written
    #[structopt(short = "d", long, default_value = "./vaults")]
    vaults_dir: PathBuf,

    /// Number of vaults to spawn with the first one being the genesis. This number should be greater than 0.
    #[structopt(short = "n", long, default_value = "8")]
    num_vaults: u8,

    /// Vebosity level for vaults logs
    #[structopt(short = "v", parse(from_occurrences))]
    vault_verbosity: u8,
}

pub fn run() -> Result<(), String> {
    let opt = Opt::from_args();
    let vault_bin_path = get_vault_bin_path(opt.vault_path)?;
    debug!(
        "Launching with safe_vault executable from: {}",
        vault_bin_path.display()
    );

    // TODO: read genesis IP and port number from genesis vault stdout output
    let genesis_port_number = "40000";
    let mut common_args: Vec<&str> = vec![];

    let mut verbosity = String::from("-");
    if opt.vault_verbosity > 0 {
        for _ in 0..opt.vault_verbosity {
            verbosity.push('v');
        }
        common_args.push(&verbosity);
    }

    // Construct genesis vault's command arguments
    let mut genesis_vault_args = common_args.clone();
    genesis_vault_args.push("--first");
    let genesis_vault_dir = &opt
        .vaults_dir
        .join("safe-vault-genesis")
        .display()
        .to_string();
    genesis_vault_args.push("--root-dir");
    genesis_vault_args.push(genesis_vault_dir);
    genesis_vault_args.push("--port");
    genesis_vault_args.push(genesis_port_number);

    // Let's launch genesis vault now
    debug!("Launching genesis vault...");
    run_vault_cmd(&vault_bin_path, &genesis_vault_args)?;

    // We can now run the rest of the vaults
    for i in 1..opt.num_vaults {
        // We wait for a few secs before launching each new vault
        let interval_duration = time::Duration::from_secs(opt.interval);
        thread::sleep(interval_duration);

        // Construct current vault's command arguments
        let mut current_vault_args = common_args.clone();
        let vault_dir = &opt
            .vaults_dir
            .join(&format!("safe-vault-{}", i))
            .display()
            .to_string();

        current_vault_args.push("--root-dir");
        current_vault_args.push(vault_dir);
        current_vault_args.push("--hard-coded-contacts");
        let contacts = format!("[\"127.0.0.1:{}\"]", genesis_port_number);
        current_vault_args.push(&contacts);

        debug!("Launching vault #{}...", i);
        run_vault_cmd(&vault_bin_path, &current_vault_args)?;
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

fn run_vault_cmd(vault_path: &PathBuf, args: &[&str]) -> Result<(), String> {
    let path_str = vault_path.display().to_string();
    debug!("Running '{}' with args {:?} ...", path_str, args);

    let _child = Command::new(&path_str)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| {
            format!(
                "Failed to run '{}' with args '{:?}': {}",
                path_str, args, err
            )
        })?;

    /*let output = child.wait_with_output().map_err(|err| {
        format!(
            "Failed to run '{}' with args '{:?}': {}",
            path_str, args, err
        )
    })?;

    if output.status.success() {
        io::stdout()
            .write_all(&output.stdout)
            .map_err(|err| format!("Failed to output stdout: {}", err))?;
        Ok(())
    } else {
        Err(format!(
            "Failed when running '{}' with args '{:?}':\n{}",
            path_str,
            args,
            String::from_utf8_lossy(&output.stderr)
        ))
    }*/
    Ok(())
}
