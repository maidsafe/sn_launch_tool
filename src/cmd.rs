use eyre::{eyre, Result, WrapErr};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt,
    net::SocketAddr,
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
        }
    }

    fn path(&self) -> &Path {
        Path::new(&self.path)
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

    pub(crate) fn print_version(&self) -> Result<()> {
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

    pub(crate) fn run(&self, node_dir: impl AsRef<Path>, contacts: &[SocketAddr]) -> Result<()> {
        let path_str = self.path().display().to_string();
        trace!("Running '{}' with args {:?} ...", path_str, self.args);

        let mut extra_args = NodeArgs::default();
        extra_args.push("--root-dir");
        extra_args.push(node_dir.as_ref());
        extra_args.push("--log-dir");
        extra_args.push(node_dir.as_ref());

        if !contacts.is_empty() {
            extra_args.push("--hard-coded-contacts");
            extra_args.push(
                serde_json::to_string(
                    &contacts
                        .iter()
                        .map(|contact| contact.to_string())
                        .collect::<Vec<_>>(),
                )
                .wrap_err("Failed to generate genesis contacts list parameter")?,
            );
        }

        Command::new(&path_str)
            .args(&self.args)
            .args(&extra_args)
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
            .wrap_err_with(|| {
                format!("Failed to start '{}' with args '{:?}'", path_str, self.args)
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
