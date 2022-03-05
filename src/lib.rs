// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod cmd;

use eyre::{eyre, Result, WrapErr};
use std::{
    borrow::Cow,
    collections::HashSet,
    env,
    fs::{self, File},
    io::BufReader,
    net::SocketAddr,
    ops::RangeInclusive,
    path::PathBuf,
    thread,
    time::Duration,
};
use structopt::StructOpt;
use tracing::{debug, info};

use cmd::NodeCmd;

#[cfg(not(target_os = "windows"))]
const SN_NODE_EXECUTABLE: &str = "sn_node";

#[cfg(target_os = "windows")]
const SN_NODE_EXECUTABLE: &str = "sn_node.exe";

// Relative path from $HOME where to read the genesis node connection information from
const GENESIS_CONN_INFO_FILEPATH: &str = ".safe/node/node_connection_info.config";

const DEFAULT_RUST_LOG: &str = "safe_network=debug";

/// Tool to launch Safe nodes to form a local single-section network
///
/// Currently, this tool runs nodes on localhost (since that's the default if no IP address is given to the nodes)
#[derive(Debug, StructOpt)]
pub struct Launch {
    #[structopt(flatten)]
    common: CommonArgs,

    /// Interval in milliseconds between launching each of the nodes
    #[structopt(short = "i", long, default_value = "100")]
    interval: u64,

    /// Interval in seconds before deeming a peer to have timed out
    #[structopt(long = "idle-timeout-msec")]
    idle_timeout_msec: Option<u64>,

    /// Interval in seconds between qp2p keep alive messages
    #[structopt(long = "keep-alive-interval-msec")]
    keep_alive_interval_msec: Option<u64>,

    /// Path where the output directories for all the nodes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Number of nodes to spawn with the first one being the genesis. This number should be greater than 0.
    #[structopt(short = "n", long, default_value = "15", env = "NODE_COUNT")]
    num_nodes: usize,

    /// IP used to launch the nodes with.
    #[structopt(long = "ip")]
    ip: Option<String>,

    /// IP used to launch the nodes with.
    #[structopt(long = "add")]
    add_nodes_to_existing_network: bool,
}

impl Launch {
    /// Launch a network with these arguments.
    pub fn run(&self) -> Result<()> {
        let mut node_cmd = self.common.node_cmd()?;

        if let Some(idle) = self.idle_timeout_msec {
            node_cmd.push_arg("--idle-timeout-msec");
            node_cmd.push_arg(idle.to_string());
        }

        if let Some(keep_alive_interval_msec) = self.keep_alive_interval_msec {
            node_cmd.push_arg("--keep-alive-interval-msec");
            node_cmd.push_arg(keep_alive_interval_msec.to_string());
        }

        if self.common.is_local {
            node_cmd.push_arg("--skip-auto-port-forwarding");
        }

        if let Some(ip) = &self.ip {
            node_cmd.push_arg("--local-addr");
            node_cmd.push_arg(format!("{}:0", ip));
        } else if self.common.is_local {
            node_cmd.push_arg("--local-addr");
            node_cmd.push_arg("127.0.0.1:0");
        }

        debug!("Network size: {} nodes", self.num_nodes);

        let interval = Duration::from_millis(self.interval);

        if !self.add_nodes_to_existing_network {
            self.run_genesis(&node_cmd)?;
            thread::sleep(interval);

            debug!("Genesis wait over...");
        }

        // Fetch node_conn_info from $HOME/.safe/node/node_connection_info.config.
        let (genesis_contact_info, genesis_key) = read_genesis_conn_info()?;

        debug!(
            "Common node args for launching the network: {:?}",
            node_cmd.args()
        );

        let node_ids = self.node_ids()?;
        if !node_ids.is_empty() {
            info!("Launching nodes {:?}", node_ids);

            for i in node_ids {
                self.run_node(&node_cmd, i, &genesis_contact_info, genesis_key.as_ref())?;
                thread::sleep(interval);
            }
        }

        info!("Done!");
        Ok(())
    }

    fn run_genesis(&self, node_cmd: &NodeCmd) -> Result<()> {
        // Set genesis node's command arguments
        let mut genesis_cmd = node_cmd.clone();
        genesis_cmd.push_arg("--first");

        // Let's launch genesis node now
        debug!("Launching genesis node (#1)...");
        genesis_cmd.run("sn-node-genesis", &self.nodes_dir, &[], None)?;

        Ok(())
    }

    fn run_node(
        &self,
        node_cmd: &NodeCmd,
        node_idx: usize,
        contacts: &[SocketAddr],
        genesis_key_str: &str,
    ) -> Result<()> {
        if self.add_nodes_to_existing_network {
            debug!("Adding node #{}...", node_idx)
        } else {
            debug!("Launching node #{}...", node_idx)
        };
        node_cmd.run(
            &format!("sn-node-{}", node_idx),
            &self.nodes_dir,
            contacts,
            Some(genesis_key_str),
        )?;

        Ok(())
    }

    fn node_ids(&self) -> Result<RangeInclusive<usize>> {
        let paths =
            fs::read_dir(&self.nodes_dir).wrap_err("Could not read existing testnet log dir")?;

        let count = paths
            .collect::<Result<Vec<_>, _>>()
            .wrap_err("Error collecting testnet log dir")?
            .len();

        if count == 0 {
            return Err(eyre!("A genesis node could not be found."));
        }

        let last_idx: usize = if self.add_nodes_to_existing_network {
            count + self.num_nodes
        } else {
            self.num_nodes
        };

        Ok(count + 1..=last_idx)
    }
}

/// Run a Safe node to join a network
#[derive(Debug, StructOpt)]
pub struct Join {
    #[structopt(flatten)]
    common: CommonArgs,

    /// Path where the output directories for all the nodes are written
    #[structopt(short = "d", long, default_value = "./nodes")]
    nodes_dir: PathBuf,

    /// Max storage to use while running the node
    #[structopt(short, long)]
    max_capacity: Option<u64>,

    /// List of node addresses to bootstrap to for joining
    #[structopt(short = "h", long)]
    hard_coded_contacts: Vec<SocketAddr>,

    /// Genesis key of the network to join
    #[structopt(short = "g", long)]
    genesis_key: String,

    /// Local network address for the node, eg 192.168.1.100:12000
    #[structopt(long)]
    local_addr: Option<SocketAddr>,

    /// Public address for the node
    #[structopt(long)]
    public_addr: Option<SocketAddr>,

    /// Use this flag to skip automated port forwarding if you are having trouble joining a remote
    /// network. You can then setup 'manual' port forwarding on your router.
    #[structopt(long)]
    skip_auto_port_forwarding: bool,

    /// Clear data directory created by a previous node run
    #[structopt(long = "clear-data")]
    clear_data: bool,
}

impl Join {
    /// Join a network with these arguments.
    pub fn run(&self) -> Result<()> {
        let mut node_cmd = self.common.node_cmd()?;

        if let Some(max_capacity) = self.max_capacity {
            node_cmd.push_arg("--max-capacity");
            node_cmd.push_arg(max_capacity.to_string());
        }

        if self.common.is_local || self.skip_auto_port_forwarding {
            node_cmd.push_arg("--skip-auto-port-forwarding");
        }

        if let Some(local_addr) = self.local_addr {
            node_cmd.push_arg("--local-addr");
            node_cmd.push_arg(local_addr.to_string());
        } else if self.common.is_local {
            node_cmd.push_arg("--local-addr");
            node_cmd.push_arg("127.0.0.1:0");
        }

        if let Some(public_addr) = self.public_addr {
            node_cmd.push_arg("--public-addr");
            node_cmd.push_arg(public_addr.to_string());
        }

        if self.clear_data {
            node_cmd.push_arg("--clear-data");
        }

        if self.hard_coded_contacts.is_empty() {
            debug!("Failed to start a node. No contacts nodes provided.");
            return Ok(());
        }

        debug!(
            "Node to be started with contact(s): {:?}",
            self.hard_coded_contacts
        );

        debug!("Launching node...");
        node_cmd.run(
            "", // no name passed
            &self.nodes_dir,
            &self.hard_coded_contacts,
            Some(&self.genesis_key),
        )?;

        debug!(
            "Node logs are being stored at: {}/sn_node.log<DATETIME>",
            self.nodes_dir.display()
        );
        debug!("(Note that log files are rotated hourly, and subsequent files will be named sn_node.log<NEW DATE TINE>.");

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct CommonArgs {
    /// Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also used to set the path
    #[structopt(short = "p", long, env = "SN_NODE_PATH")]
    node_path: Option<PathBuf>,

    /// Verbosity level for nodes logs (default: INFO)
    #[structopt(short = "y", long, parse(from_occurrences))]
    nodes_verbosity: u8,

    /// RUST_LOG env var value to launch the nodes with.
    #[structopt(short = "l", long)]
    rust_log: Option<String>,

    /// Output logs in json format for easier processing.
    #[structopt(long)]
    json_logs: bool,

    /// Run the section locally.
    #[structopt(long = "local")]
    is_local: bool,

    /// Run the nodes using `cargo flamegraph` (which needs to be preinstalled.)
    /// It is recommended to manually run `cargo flamegraph --root --bin=sn_node -- --first` to ensure
    /// everything is built. (This command will fail dur to insufficient args, but that's okay, carry
    /// testnetting w/ --flame thereafter)
    #[structopt(long = "flame")]
    flame: bool,
}

impl CommonArgs {
    fn node_cmd(&self) -> Result<NodeCmd> {
        let mut cmd = match self.node_path.as_deref() {
            Some(p) => NodeCmd::new(p),
            None => {
                let mut path =
                    dirs_next::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;

                path.push(".safe/node");
                path.push(SN_NODE_EXECUTABLE);
                NodeCmd::new(path)
            }
        };

        let rust_log = self.rust_log();
        info!("Using RUST_LOG '{}'", rust_log);

        cmd.push_env("RUST_LOG", rust_log);
        cmd.push_arg(
            // We need a minimum of INFO level for nodes verbosity,
            // since the genesis node logs the contact info at INFO level
            format!("-{}", "v".repeat(2 + self.nodes_verbosity as usize)),
        );

        if self.json_logs {
            cmd.push_arg("--json-logs");
        }

        if self.flame {
            cmd.set_flame(self.flame);
        }

        debug!(
            "Using sn_node @ {} from {}",
            cmd.version()?,
            cmd.path().display()
        );

        Ok(cmd)
    }

    fn rust_log(&self) -> Cow<'_, str> {
        match self.rust_log.as_deref() {
            Some(rust_log_flag) => rust_log_flag.into(),
            None => match env::var("RUST_LOG") {
                Ok(rust_log_env) => rust_log_env.into(),
                Err(_) => DEFAULT_RUST_LOG.into(),
            },
        }
    }
}

fn read_genesis_conn_info() -> Result<(Vec<SocketAddr>, String)> {
    let home_dir = dirs_next::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;
    let conn_info_path = home_dir.join(GENESIS_CONN_INFO_FILEPATH);

    let file = File::open(&conn_info_path).wrap_err_with(|| {
        format!(
            "Failed to open node connection information file at '{}'",
            conn_info_path.display()
        )
    })?;
    let reader = BufReader::new(file);
    let (genesis_key_str, hard_coded_contacts): (String, HashSet<SocketAddr>) =
        serde_json::from_reader(reader).wrap_err_with(|| {
            format!(
                "Failed to parse content of node connection information file at '{}'",
                conn_info_path.display()
            )
        })?;

    let contacts: Vec<SocketAddr> = hard_coded_contacts.into_iter().collect();

    debug!("Connection info directory: {}", conn_info_path.display());
    debug!("Genesis node contact info: {:?}", contacts);
    debug!("Network's genesis key: {}", genesis_key_str);

    Ok((contacts, genesis_key_str))
}
