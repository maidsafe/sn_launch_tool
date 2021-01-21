// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use directories::{BaseDirs, UserDirs};
use log::debug;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use structopt::StructOpt;

#[cfg(not(target_os = "windows"))]
const SN_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
const SN_NODE_EXECUTABLE: &str = "sn_node.exe";

/// Tool to launch Safe nodes to form a local single-section network
///
/// Currently, this tool runs nodes on localhost (since that's the default if no IP address is given to the nodes)
#[derive(StructOpt, Debug)]
#[structopt(name = "sn_launch_tool")]
struct CmdArgs {
    /// Verbosity level for this tool
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbosity: u8,

    /// Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SN_NODE_PATH")]
    node_path: Option<PathBuf>,

    /// Interval in seconds between launching each of the nodes
    #[structopt(short = "i", long, default_value = "1")]
    interval: u64,

    /// Interval in seconds before deeming a peer to have timed out
    #[structopt(long = "idle-timeout-msec", default_value = "5500")]
    idle_timeout_msec: u64,

    /// Interval in seconds between qp2p keep alive messages
    #[structopt(long = "keep-alive-interval-msec", default_value = "5")]
    keep_alive_interval_msec: u64,

    /// Path where the output directories for all the notes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Number of nodes to spawn with the first one being the genesis. This number should be greater than 0.
    #[structopt(short = "n", long, default_value = "8")]
    num_nodes: u8,

    /// Verbosity level for nodes logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    nodes_verbosity: u8,

    /// IP used to launch the nodes with.
    #[structopt(long = "ip")]
    ip: Option<String>,

    /// Run the section locally.
    #[structopt(long = "local")]
    is_local: bool,
}

/// Run a Safe node to join a network
#[derive(StructOpt, Debug)]
#[structopt(name = "sn_launch_tool-join")]
struct JoinCmdArgs {
    /// Verbosity level for this tool
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbosity: u8,

    /// Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SN_NODE_PATH")]
    node_path: Option<PathBuf>,

    /// Path where the output directories for all the nodes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Verbosity level for nodes logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    nodes_verbosity: u8,

    /// IP used to launch the nodes with.
    #[structopt(short = "h", long)]
    hard_coded_contacts: Option<String>,
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

    let node_bin_path = get_node_bin_path(args.node_path)?;
    let msg = format!(
        "Launching with node executable from: {}",
        node_bin_path.display()
    );
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let mut common_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for nodes verbosity,
    // since the genesis node logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.nodes_verbosity as usize));
    common_args.push(&verbosity);

    if let Some(ref hccs) = args.hard_coded_contacts {
        let mut hard_coded_contacts: Vec<String> = Vec::new();
        for hcc in hccs.split(',') {
            hard_coded_contacts.push(format!("\"{}\"", hcc));
        }
        let genesis_contact_info = format!("[{}]", hard_coded_contacts.join(","));
        let msg = format!(
            "Node started with hardcoded contact(s): {}",
            genesis_contact_info
        );
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);

        // Construct current node's command arguments
        let node_dir = &args.nodes_dir.display().to_string();

        let current_node_args =
            build_node_args(common_args.clone(), &node_dir, Some(&genesis_contact_info));

        let msg = "Launching node...";
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);
        run_node_cmd(&node_bin_path, &current_node_args, args.verbosity)?;

        let msg = format!("Node logs are being stored at: {}/sn_node.log", node_dir);
        if args.verbosity > 0 {
            println!("{}", msg);
        }
    } else {
        let msg = "Failed to start a node. No hardcoded contacts provided.";
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);
    }
    Ok(())
}

pub fn run_with(cmd_args: Option<&[&str]>) -> Result<(), String> {
    // Let's first get all the arguments passed in, either as function's args, or CLI args
    let args = match cmd_args {
        None => CmdArgs::from_args(),
        Some(cmd_args) => CmdArgs::from_iter_safe(cmd_args).map_err(|err| err.to_string())?,
    };

    let node_bin_path = get_node_bin_path(args.node_path)?;
    let msg = format!(
        "Launching with node executable from: {}",
        node_bin_path.display()
    );
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let msg = format!("Network size: {} nodes", args.num_nodes);
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let mut common_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for nodes verbosity,
    // since the genesis node logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.nodes_verbosity as usize));
    common_args.push(&verbosity);

    let idle = args.idle_timeout_msec.to_string();
    let keep_alive = args.keep_alive_interval_msec.to_string();

    common_args.push("--idle-timeout-msec");
    common_args.push(&idle);
    common_args.push("--keep-alive-interval-msec");
    common_args.push(&keep_alive);

    if let Some(ref ip) = args.ip {
        common_args.push("--ip");
        common_args.push(ip);
    }

    if args.is_local {
        common_args.push("--local");
    }

    // Construct genesis node's command arguments
    let genesis_node_dir = &args.nodes_dir.join("sn-node-genesis");
    let genesis_node_dir_str = genesis_node_dir.display().to_string();
    let genesis_node_args = build_node_args(
        common_args.clone(),
        &genesis_node_dir_str,
        None, /* genesis */
    );

    // Let's launch genesis node now
    let msg = "Launching genesis node (#1)...";
    if args.verbosity > 0 {
        println!("{}", msg);
    }
    debug!("{}", msg);
    run_node_cmd(&node_bin_path, &genesis_node_args, args.verbosity)?;

    // Get port number of genesis node to pass it as hard-coded contact to the other nodes
    let interval_duration = Duration::from_secs(args.interval);
    thread::sleep(interval_duration);

    // Fetch node_conn_info from $HOME/.safe/node/node_connection_info.config.
    let user_dir = UserDirs::new().ok_or_else(|| "Could not fetch home directory".to_string())?;
    let node_conn_info = user_dir
        .home_dir()
        .join(".safe/node/node_connection_info.config");

    let raw = fs::read_to_string(&node_conn_info).map_err(|e| e.to_string())?;
    let genesis_contact_info = format!("[{}]", raw);
    let msg = format!("Genesis node contact info: {}", genesis_contact_info);
    if args.verbosity > 0 {
        println!("Connection info directory: {:?}", node_conn_info);
        println!("{}", msg);
    }
    debug!("{}", msg);
    debug!("Connection info directory: {:?}", node_conn_info);

    if args.verbosity > 0 {
        println!(
            "Common node args for launching the network: {:?}",
            common_args
        );
    }

    // We can now run the rest of the nodes
    for i in 2..args.num_nodes + 1 {
        // Construct current node's command arguments
        let node_dir = &args
            .nodes_dir
            .join(&format!("sn-node-{}", i))
            .display()
            .to_string();

        let current_node_args =
            build_node_args(common_args.clone(), &node_dir, Some(&genesis_contact_info));

        let msg = format!("Launching node #{}...", i);
        if args.verbosity > 0 {
            println!("{}", msg);
        }
        debug!("{}", msg);
        run_node_cmd(&node_bin_path, &current_node_args, args.verbosity)?;

        // We wait for a few secs before launching each new node
        thread::sleep(interval_duration);
    }

    println!("Done!");
    Ok(())
}

#[inline]
fn get_node_bin_path(node_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match node_path {
        Some(p) => Ok(p),
        None => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| "Failed to obtain user's home path".to_string())?;

            let mut path = PathBuf::from(base_dirs.home_dir());
            path.push(".safe");
            path.push("node");
            path.push(SN_NODE_EXECUTABLE);
            Ok(path)
        }
    }
}

fn build_node_args<'a>(
    mut base_args: Vec<&'a str>,
    node_dir: &'a str,
    contact_info: Option<&'a str>,
) -> Vec<&'a str> {
    if let Some(contact) = contact_info {
        base_args.push("--hard-coded-contacts");
        base_args.push(contact);
    } else {
        base_args.push("--first");
    }

    base_args.push("--root-dir");
    base_args.push(node_dir);
    base_args.push("--log-dir");
    base_args.push(node_dir);

    base_args
}

fn run_node_cmd(node_path: &PathBuf, args: &[&str], verbosity: u8) -> Result<(), String> {
    let path_str = node_path.display().to_string();
    let msg = format!("Running '{}' with args {:?} ...", path_str, args);
    if verbosity > 1 {
        println!("{}", msg);
    }
    debug!("{}", msg);

    let _child = Command::new(&path_str)
        .args(args)
        .env("RUST_LOG", "sn_node=debug")
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
