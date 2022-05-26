use eyre::{eyre, Result, WrapErr};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use tracing::{debug, trace};

const NODE_LIVENESS_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub(crate) struct NodeCmd<'a> {
    path: Cow<'a, OsStr>,
    envs: Vec<(Cow<'a, OsStr>, Cow<'a, OsStr>)>,
    args: NodeArgs<'a>,
    // run w/ flamegraph
    flame: bool,
}

impl<'a> NodeCmd<'a> {
    pub(crate) fn new<P, Pb>(path: P) -> Self
    where
        P: Into<Cow<'a, Pb>>,
        Pb: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        Pb::Owned: Into<OsString>,
    {
        Self {
            path: into_cow_os_str(path),
            envs: Default::default(),
            args: Default::default(),
            flame: false,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        Path::new(&self.path)
    }

    pub(crate) fn set_flame(&mut self, flame: bool) {
        self.flame = flame
    }

    pub(crate) fn gen_flamegraph(&self) -> bool {
        self.flame
    }

    pub(crate) fn args(&self) -> &NodeArgs {
        &self.args
    }

    pub(crate) fn push_env<K, Kb, V, Vb>(&mut self, key: K, value: V)
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

    pub(crate) fn push_arg<A, B>(&mut self, arg: A)
    where
        A: Into<Cow<'a, B>>,
        B: AsRef<OsStr> + ToOwned + ?Sized + 'a,
        B::Owned: Into<OsString>,
    {
        self.args.push(arg);
    }

    pub(crate) fn version(&self) -> Result<String> {
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

        Ok(String::from_utf8_lossy(&version).trim().to_string())
    }

    pub(crate) fn run(&self, node_name: &str, node_dir: &Path) -> Result<()> {
        let node_dir = node_dir.join(node_name);

        let mut cmd = self.path().display().to_string();

        let flame_on = self.gen_flamegraph();
        let graph_output = format!("-o {}-flame.svg", node_name);

        if flame_on {
            cmd = "cargo".to_string();
            // make a dir per node
            std::fs::create_dir_all(node_name)?;
            debug!("Flame graph will be stored: {:?}", graph_output);
        }

        trace!("Running '{cmd}' with args {:?} ...", self.args);

        let mut extra_args = NodeArgs::default();
        extra_args.push("--root-dir");
        extra_args.push(node_dir.clone());
        extra_args.push("--log-dir");
        extra_args.push(node_dir);

        let mut the_cmd = Command::new(cmd.clone());
        let additonal_flame_args = vec![
            "flamegraph",
            &graph_output,
            "--root",
            "--bin",
            "sn_node",
            "--",
        ];
        if flame_on {
            debug!("Launching nodes via `cargo flamegraph`");
            // we set the command ro run in each individal node dir (as each flamegraph uses a file `cargo-flamegraph.stacks` which cannot be renamed per per node)
            // we set flamegraph to root as that's necesasry on mac
            the_cmd
                .current_dir(node_name)
                .args(additonal_flame_args.clone());
        }
        the_cmd
            .args(&self.args)
            .args(&extra_args)
            .envs(self.envs.iter().map(
                // this looks like a no-op but really converts `&(_, _)` into `(_, _)`
                |(key, value)| (key, value),
            ))
            .stdout(Stdio::inherit())
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
            .wrap_err_with(|| {
                let mut all_args = vec![];
                if flame_on {
                    // all_args.extend(additonal_flame_args);

                    for arg in additonal_flame_args {
                        let c = into_cow_os_str(arg);
                        all_args.push(c);
                    }
                }

                for arg in self.args.into_iter() {
                    all_args.push(arg.clone());
                }
                for arg in extra_args.into_iter() {
                    all_args.push(arg.clone());
                }

                format!("Failed to start '{}' with args '{:?}'", cmd, all_args)
            })?;

        Ok(())
    }
}

#[derive(Clone, Default)]
pub(crate) struct NodeArgs<'a>(Vec<Cow<'a, OsStr>>);

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
