// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use eyre::{eyre, Result, WrapErr};
use std::fs;
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{self, BufReader},
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use structopt::StructOpt;
use tracing::{debug, info, trace};

#[cfg(not(target_os = "windows"))]
const SN_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
const SN_NODE_EXECUTABLE: &str = "sn_node.exe";

// Relative path from $HOME where to read the genesis node connection information from
const GENESIS_CONN_INFO_FILEPATH: &str = ".safe/node/node_connection_info.config";

const DEFAULT_RUST_LOG: &str = "safe_network=debug";
const NODE_LIVENESS_TIMEOUT: Duration = Duration::from_secs(2);

/// Tool to launch Safe nodes to form a local single-section network
///
/// Currently, this tool runs nodes on localhost (since that's the default if no IP address is given to the nodes)
#[derive(Debug, StructOpt)]
pub struct Launch {
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
    #[structopt(long = "keep-alive-interval-msec", default_value = "4000")]
    keep_alive_interval_msec: u64,

    /// Path where the output directories for all the nodes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Number of nodes to spawn with the first one being the genesis. This number should be greater than 0.
    #[structopt(short = "n", long, default_value = "11", env = "NODE_COUNT")]
    num_nodes: usize,

    /// Verbosity level for nodes logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    nodes_verbosity: u8,

    /// IP used to launch the nodes with.
    #[structopt(long = "ip")]
    ip: Option<String>,

    /// IP used to launch the nodes with.
    #[structopt(long = "add")]
    add_nodes_to_existing_network: bool,

    /// Run the section locally.
    #[structopt(long = "local")]
    is_local: bool,

    /// RUST_LOG env var value to launch the nodes with.
    #[structopt(short = "l", long)]
    rust_log: Option<String>,
}

impl Launch {
    /// Launch a network with these arguments.
    pub fn run(&self) -> Result<()> {
        launch(self)
    }
}

/// Run a Safe node to join a network
#[derive(Debug, StructOpt)]
pub struct Join {
    /// Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SN_NODE_PATH")]
    node_path: Option<PathBuf>,

    /// Path where the output directories for all the nodes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Verbosity level for nodes logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    nodes_verbosity: u8,

    /// Max storage to use while running the node
    #[structopt(short, long)]
    max_capacity: Option<u64>,

    /// List of node addresses to bootstrap to for joining
    #[structopt(short = "h", long)]
    hard_coded_contacts: Vec<SocketAddr>,

    /// Local network address for the node, eg 192.168.1.100:12000
    #[structopt(long)]
    local_addr: Option<SocketAddr>,

    /// Public address for the node
    #[structopt(long)]
    public_addr: Option<SocketAddr>,

    /// RUST_LOG env var value to launch the nodes with.
    #[structopt(short = "l", long)]
    rust_log: Option<String>,

    /// Clear data directory created by a previous node run
    #[structopt(long = "clear-data")]
    clear_data: bool,
}

impl Join {
    /// Join a network with these arguments.
    pub fn run(&self) -> Result<()> {
        join(self)
    }
}

fn launch(args: &Launch) -> Result<()> {
    let node_bin_path = get_node_bin_path(args.node_path.as_deref())?;

    debug!("Network size: {} nodes", args.num_nodes);

    let mut common_node_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for nodes verbosity,
    // since the genesis node logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.nodes_verbosity as usize));
    common_node_args.push(&verbosity);

    let idle = args.idle_timeout_msec.to_string();
    let keep_alive = args.keep_alive_interval_msec.to_string();
    let adding_nodes: bool = args.add_nodes_to_existing_network;

    common_node_args.push("--idle-timeout-msec");
    common_node_args.push(&idle);
    common_node_args.push("--keep-alive-interval-msec");
    common_node_args.push(&keep_alive);

    let addr = if let Some(ref ip) = args.ip {
        format!("{}:0", ip)
    } else {
        "127.0.0.1:0".to_string()
    };

    let rust_log = get_rust_log(args.rust_log.as_deref());
    // Get port number of genesis node to pass it as hard-coded contact to the other nodes
    let interval_duration = Duration::from_secs(args.interval);

    if !adding_nodes {
        // Construct genesis node's command arguments
        let genesis_node_dir = &args.nodes_dir.join("sn-node-genesis");
        let genesis_node_dir_str = genesis_node_dir.display().to_string();
        let mut genesis_args = common_node_args.clone();
        genesis_args.push("--first");
        genesis_args.push(&addr);
        let genesis_node_args =
            build_node_args(genesis_args, &genesis_node_dir_str, None /* genesis */);

        // Let's launch genesis node now
        debug!("Launching genesis node (#1)...");
        run_node_cmd(&node_bin_path, &genesis_node_args, rust_log.clone())?;

        thread::sleep(interval_duration);
    }

    // Fetch node_conn_info from $HOME/.safe/node/node_connection_info.config.
    let genesis_contact_info = read_genesis_conn_info()?;

    debug!(
        "Common node args for launching the network: {:?}",
        common_node_args
    );

    let paths =
        fs::read_dir(&args.nodes_dir).wrap_err("Could not read existing testnet log dir")?;

    let existing_nodes_count = paths
        .collect::<Result<Vec<_>, io::Error>>()
        .wrap_err("Error collecting testnet log dir")?
        .len();

    info!("{:?} existing nodes found", existing_nodes_count);

    if existing_nodes_count == 0 {
        return Err(eyre!("A genesis node could not be found."));
    }

    let end: usize = if adding_nodes {
        existing_nodes_count + args.num_nodes
    } else {
        args.num_nodes
    };

    // We can now run the rest of the nodes
    for i in existing_nodes_count..end {
        let this_node = i + 1;
        // Construct current node's command arguments
        let node_dir = args
            .nodes_dir
            .join(&format!("sn-node-{}", this_node))
            .display()
            .to_string();

        let current_node_args = build_node_args(
            common_node_args.clone(),
            &node_dir,
            Some(&genesis_contact_info),
        );

        if adding_nodes {
            debug!("Adding node #{}...", this_node)
        } else {
            debug!("Launching node #{}...", this_node)
        };
        run_node_cmd(&node_bin_path, &current_node_args, rust_log.clone())?;

        // We wait for a few secs before launching each new node
        thread::sleep(interval_duration);
    }

    info!("Done!");
    Ok(())
}

fn join(args: &Join) -> Result<()> {
    let node_bin_path = get_node_bin_path(args.node_path.as_deref())?;

    let mut common_args: Vec<&str> = vec![];

    // We need a minimum of INFO level for nodes verbosity,
    // since the genesis node logs the contact info at INFO level
    let verbosity = format!("-{}", "v".repeat(2 + args.nodes_verbosity as usize));
    common_args.push(&verbosity);

    let max_capacity_string;
    if let Some(max_capacity) = args.max_capacity {
        common_args.push("--max-capacity");
        max_capacity_string = max_capacity.to_string();
        common_args.push(&max_capacity_string);
    }

    let local_addr_string;
    if let Some(local_addr) = args.local_addr {
        common_args.push("--local-addr");
        local_addr_string = local_addr.to_string();
        common_args.push(&local_addr_string);
    }

    let public_addr_string;
    if let Some(public_addr) = args.public_addr {
        common_args.push("--public-addr");
        public_addr_string = public_addr.to_string();
        common_args.push(&public_addr_string);
    }

    if args.clear_data {
        common_args.push("--clear-data");
    }

    if args.hard_coded_contacts.is_empty() {
        debug!("Failed to start a node. No contacts nodes provided.");
        return Ok(());
    }

    let contacts: Vec<String> = args
        .hard_coded_contacts
        .iter()
        .map(|c| c.to_string())
        .collect();

    let conn_info_str = serde_json::to_string(&contacts)
        .wrap_err("Failed to generate genesis contacts list parameter")?;

    let rust_log = get_rust_log(args.rust_log.as_deref());

    debug!("Node to be started with contact(s): {}", conn_info_str);

    // Construct current node's command arguments
    let node_dir = args.nodes_dir.display().to_string();

    let current_node_args = build_node_args(common_args.clone(), &node_dir, Some(&conn_info_str));

    debug!("Launching node...");
    run_node_cmd(&node_bin_path, &current_node_args, rust_log)?;

    debug!(
        "Node logs are being stored at: {}/sn_node.log<DATETIME>",
        node_dir
    );
    debug!("(Note that log files are rotated hourly, and subsequent files will be named sn_node.log<NEW DATE TINE>.");

    Ok(())
}

fn get_node_bin_path(node_path: Option<&Path>) -> Result<PathBuf> {
    let node_bin_path = match node_path {
        Some(p) => p.into(),
        None => {
            let mut path =
                dirs_next::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;

            path.push(".safe/node");
            path.push(SN_NODE_EXECUTABLE);
            path
        }
    };

    let version = Command::new(&node_bin_path)
        .args(&["-V"])
        .output()
        .map_or_else(
            |error| Err(eyre!(error)),
            |output| {
                if output.status.success() {
                    Ok(output.stdout)
                } else {
                    Err(eyre!(
                        "Process exited with non-zero status (status: {}, stderr: {})",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            },
        )
        .wrap_err_with(|| {
            format!(
                "Failed to run '{}' with args '{:?}'",
                node_bin_path.display(),
                &["-V"]
            )
        })?;

    debug!(
        "Using sn_node @ {} from {}",
        String::from_utf8_lossy(&version).trim(),
        node_bin_path.display()
    );

    Ok(node_bin_path)
}

fn build_node_args<'a>(
    mut base_args: Vec<&'a str>,
    node_dir: &'a str,
    contact_info: Option<&'a str>,
) -> Vec<&'a str> {
    if let Some(contact) = contact_info {
        base_args.push("--hard-coded-contacts");
        base_args.push(contact);
    }

    base_args.push("--root-dir");
    base_args.push(node_dir);
    base_args.push("--log-dir");
    base_args.push(node_dir);

    base_args
}

fn run_node_cmd(node_path: &Path, args: &[&str], rust_log: String) -> Result<()> {
    let path_str = node_path.display().to_string();
    trace!("Running '{}' with args {:?} ...", path_str, args);

    Command::new(&path_str)
        .args(args)
        .env("RUST_LOG", rust_log)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| eyre!(error))
        .and_then(|mut child| {
            // Wait a couple of seconds to see if the node fails immediately, so we can fail fast
            thread::sleep(NODE_LIVENESS_TIMEOUT);

            if let Some(status) = child.try_wait()? {
                return Err(eyre!("Node exited early (status: {})", status));
            }

            Ok(())
        })
        .wrap_err_with(|| format!("Failed to start '{}' with args '{:?}'", path_str, args))?;

    Ok(())
}

fn read_genesis_conn_info() -> Result<String> {
    let home_dir = dirs_next::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;
    let conn_info_path = home_dir.join(GENESIS_CONN_INFO_FILEPATH);

    let file = File::open(&conn_info_path).wrap_err_with(|| {
        format!(
            "Failed to open node connection information file at '{}'",
            conn_info_path.display()
        )
    })?;
    let reader = BufReader::new(file);
    let hard_coded_contacts: HashSet<SocketAddr> =
        serde_json::from_reader(reader).wrap_err_with(|| {
            format!(
                "Failed to parse content of node connection information file at '{}'",
                conn_info_path.display()
            )
        })?;

    let contacts: Vec<String> = hard_coded_contacts.iter().map(|c| c.to_string()).collect();

    let conn_info_str = serde_json::to_string(&contacts)
        .wrap_err("Failed to generate genesis contacts list parameter")?;

    debug!("Connection info directory: {}", conn_info_path.display());
    debug!("Genesis node contact info: {}", conn_info_str);

    Ok(conn_info_str)
}

fn get_rust_log(rust_log_from_args: Option<&str>) -> String {
    let rust_log = match rust_log_from_args {
        Some(rust_log_flag) => rust_log_flag.to_string(),
        None => match env::var("RUST_LOG") {
            Ok(rust_log_env) => rust_log_env,
            Err(_) => DEFAULT_RUST_LOG.to_string(),
        },
    };
    info!("Using RUST_LOG '{}'", rust_log);
    rust_log
}
