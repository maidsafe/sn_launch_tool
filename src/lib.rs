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
    borrow::Cow,
    collections::HashSet,
    env,
    ffi::{OsStr, OsString},
    fmt,
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
    #[structopt(flatten)]
    common: CommonArgs,

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

    /// IP used to launch the nodes with.
    #[structopt(long = "ip")]
    ip: Option<String>,

    /// IP used to launch the nodes with.
    #[structopt(long = "add")]
    add_nodes_to_existing_network: bool,

    /// Run the section locally.
    #[structopt(long = "local")]
    is_local: bool,
}

impl Launch {
    /// Launch a network with these arguments.
    pub fn run(&self) -> Result<()> {
        launch(self)
    }

    fn node_cmd(&self) -> Result<NodeCmd<'_>> {
        let mut cmd = self.common.node_cmd()?;

        cmd.push_arg("--idle-timeout-msec");
        cmd.push_arg(self.idle_timeout_msec.to_string());
        cmd.push_arg("--keep-alive-interval-msec");
        cmd.push_arg(self.keep_alive_interval_msec.to_string());

        Ok(cmd)
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

    /// Local network address for the node, eg 192.168.1.100:12000
    #[structopt(long)]
    local_addr: Option<SocketAddr>,

    /// Public address for the node
    #[structopt(long)]
    public_addr: Option<SocketAddr>,

    /// Clear data directory created by a previous node run
    #[structopt(long = "clear-data")]
    clear_data: bool,
}

impl Join {
    /// Join a network with these arguments.
    pub fn run(&self) -> Result<()> {
        join(self)
    }

    fn node_cmd(&self) -> Result<NodeCmd<'_>> {
        let mut cmd = self.common.node_cmd()?;

        if let Some(max_capacity) = self.max_capacity {
            cmd.push_arg("--max-capacity");
            cmd.push_arg(max_capacity.to_string());
        }

        if let Some(local_addr) = self.local_addr {
            cmd.push_arg("--local-addr");
            cmd.push_arg(local_addr.to_string());
        }

        if let Some(public_addr) = self.public_addr {
            cmd.push_arg("--public-addr");
            cmd.push_arg(public_addr.to_string());
        }

        if self.clear_data {
            cmd.push_arg("--clear-data");
        }

        Ok(cmd)
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
}

impl CommonArgs {
    fn node_cmd(&self) -> Result<NodeCmd> {
        let mut cmd = NodeCmd::new(self.node_path()?);

        cmd.push_env("RUST_LOG", self.rust_log());
        cmd.push_arg(
            // We need a minimum of INFO level for nodes verbosity,
            // since the genesis node logs the contact info at INFO level
            format!("-{}", "v".repeat(2 + self.nodes_verbosity as usize)),
        );

        Ok(cmd)
    }

    fn node_path(&self) -> Result<Cow<'_, Path>> {
        match self.node_path.as_deref() {
            Some(p) => Ok(p.into()),
            None => {
                let mut path =
                    dirs_next::home_dir().ok_or_else(|| eyre!("Home directory not found"))?;

                path.push(".safe/node");
                path.push(SN_NODE_EXECUTABLE);
                Ok(path.into())
            }
        }
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

fn launch(args: &Launch) -> Result<()> {
    let node_cmd = args.node_cmd()?;
    node_cmd.print_version()?;

    debug!("Network size: {} nodes", args.num_nodes);

    let adding_nodes: bool = args.add_nodes_to_existing_network;

    let addr = if let Some(ref ip) = args.ip {
        format!("{}:0", ip)
    } else {
        "127.0.0.1:0".to_string()
    };

    let rust_log = args.common.rust_log();
    info!("Using RUST_LOG '{}'", rust_log);
    // Get port number of genesis node to pass it as hard-coded contact to the other nodes
    let interval_duration = Duration::from_secs(args.interval);

    if !adding_nodes {
        // Construct genesis node's command arguments
        let genesis_node_dir = &args.nodes_dir.join("sn-node-genesis");
        let genesis_node_dir_str = genesis_node_dir.display().to_string();
        let mut genesis_node_args = build_node_args(&genesis_node_dir_str, None);
        genesis_node_args.push("--first");
        genesis_node_args.push(&addr);

        // Let's launch genesis node now
        debug!("Launching genesis node (#1)...");
        node_cmd.run(&genesis_node_args)?;

        thread::sleep(interval_duration);
    }

    // Fetch node_conn_info from $HOME/.safe/node/node_connection_info.config.
    let genesis_contact_info = read_genesis_conn_info()?;

    debug!(
        "Common node args for launching the network: {:?}",
        node_cmd.args
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

        let current_node_args = build_node_args(&node_dir, Some(&genesis_contact_info));

        if adding_nodes {
            debug!("Adding node #{}...", this_node)
        } else {
            debug!("Launching node #{}...", this_node)
        };
        node_cmd.run(&current_node_args)?;

        // We wait for a few secs before launching each new node
        thread::sleep(interval_duration);
    }

    info!("Done!");
    Ok(())
}

fn join(args: &Join) -> Result<()> {
    let node_cmd = args.node_cmd()?;
    node_cmd.print_version()?;

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

    let rust_log = args.common.rust_log();
    info!("Using RUST_LOG '{}'", rust_log);

    debug!("Node to be started with contact(s): {}", conn_info_str);

    // Construct current node's command arguments
    let node_dir = args.nodes_dir.display().to_string();

    let current_node_args = build_node_args(&node_dir, Some(&conn_info_str));

    debug!("Launching node...");
    node_cmd.run(&current_node_args)?;

    debug!(
        "Node logs are being stored at: {}/sn_node.log<DATETIME>",
        node_dir
    );
    debug!("(Note that log files are rotated hourly, and subsequent files will be named sn_node.log<NEW DATE TINE>.");

    Ok(())
}

struct NodeCmd<'a> {
    path: Cow<'a, OsStr>,
    envs: Vec<(Cow<'a, OsStr>, Cow<'a, OsStr>)>,
    args: NodeArgs<'a>,
}

impl<'a> NodeCmd<'a> {
    fn new<P, Pb>(path: P) -> Self
    where
        P: Into<Cow<'a, Pb>>,
        Pb: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        Pb::Owned: Into<OsString>,
    {
        Self {
            path: into_cow_os_str(path),
            envs: Default::default(),
            args: Default::default(),
        }
    }

    fn path(&self) -> &Path {
        Path::new(&self.path)
    }

    fn push_env<K, Kb, V, Vb>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'a, Kb>>,
        Kb: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        Kb::Owned: Into<OsString>,
        V: Into<Cow<'a, Vb>>,
        Vb: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        Vb::Owned: Into<OsString>,
    {
        self.envs
            .push((into_cow_os_str(key), into_cow_os_str(value)));
    }

    fn push_arg<A, B>(&mut self, arg: A)
    where
        A: Into<Cow<'a, B>>,
        B: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        B::Owned: Into<OsString>,
    {
        self.args.push(arg);
    }

    fn print_version(&self) -> Result<()> {
        let version = Command::new(&self.path)
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
                    self.path().display(),
                    &["-V"]
                )
            })?;

        debug!(
            "Using sn_node @ {} from {}",
            String::from_utf8_lossy(&version).trim(),
            self.path().display()
        );

        Ok(())
    }

    fn run(&self, args: &[&str]) -> Result<()> {
        let path_str = self.path().display().to_string();
        trace!("Running '{}' with args {:?} ...", path_str, args);

        Command::new(&path_str)
            .args(&self.args)
            .args(args)
            .envs(self.envs.iter().map(
                // this looks like a no-op but really converts `&(_, _)` into `(_, _)`
                |(key, value)| (key, value),
            ))
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
}

#[derive(Default)]
struct NodeArgs<'a>(Vec<Cow<'a, OsStr>>);

impl<'a> NodeArgs<'a> {
    fn push<A, B>(&mut self, arg: A)
    where
        A: Into<Cow<'a, B>>,
        B: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        B::Owned: Into<OsString>,
    {
        self.0.push(into_cow_os_str(arg));
    }
}

impl<'a> IntoIterator for &'a NodeArgs<'a> {
    type Item = &'a Cow<'a, OsStr>;

    type IntoIter = std::slice::Iter<'a, Cow<'a, OsStr>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> fmt::Debug for NodeArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|arg| -> &OsStr { arg.as_ref() }))
            .finish()
    }
}

fn build_node_args<'a>(node_dir: &'a str, contact_info: Option<&'a str>) -> Vec<&'a str> {
    let mut base_args = Vec::new();

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

fn into_cow_os_str<'a, V, Vb>(val: V) -> Cow<'a, OsStr>
where
    V: Into<Cow<'a, Vb>>,
    Vb: AsRef<OsStr> + ToOwned + ?Sized + 'a,
    Vb::Owned: Into<OsString>,
{
    match val.into() {
        Cow::Borrowed(val) => val.as_ref().into(),
        Cow::Owned(val) => val.into().into(),
    }
}
