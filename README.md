# safe-network-launch-tool
A cross platform tool to easily launch a SAFE Network test section from home

## Installing the SAFE Vault

A local SAFE network is bootstrapped by running several [SAFE vaults](https://github.com/maidsafe/safe_vault) which automatically interconnect forming a network.

In order to run your own local network you'd need to follow these steps:
- Download latest release from [safe_vault releases](https://github.com/maidsafe/safe_vault/releases/latest/)
- Untar/unzip the downloaded file into a directory of your choice
- Execute this tool specifying the path of the `safe_vault` executable

The following is an example of how to perform this on Linux or Mac:
```shell
$ mkdir ~/my-local-network
$ cd ~/my-local-network
$ curl -O https://github.com/maidsafe/safe_vault/releases/download/0.21.0/safe_vault-0.21.0-x86_64-unknown-linux-musl.tar.gz
$ tar -xzvf safe_vault-0.21.0-x86_64-unknown-linux-musl.tar.gz
```

## Run a local network

At current state of the [SAFE project](), a single-section SAFE network can be launched locally in our system. If the SAFE vault binary was downloaded and extracted at `~/my-local-network/` as described above, we can now launch the network using this tool following these steps:
```shell
$ git clone https://github.com/maidsafe/safe-network-launch-tool
$ cd safe-network-launch-tool
$ cargo run -- -p ~/my-local-network/safe_vault -v
Launching with vault executable from: ~/my-local-network/safe_vault
Network size: 8 vaults
Launching genesis vault (#1)...
Genesis vault contact info: ["127.0.0.1:59303"]
Launching vault #2...
Launching vault #3...
Launching vault #4...
Launching vault #5...
Launching vault #6...
Launching vault #7...
Launching vault #8...
Done!
```

Once the local network is running, the connection configuration file will be already in the correct place for your applications to connect to this network, so you can simply run any application from this moment on to connect to your local network. Note that depending on the application, you may need to restart it so it uses the new connection information for your local network.

In order to shutdown a running local network, all processes instances of safe_vault must be killed, e.g. on Linux or Mac you can use the `killall` command:
```shell
$ killall safe_vault
```

This tool allows you to change default values to customise part of the process, you can use the `--help` flag to get a complete list of the flags and options it supports:
```shell
safe-nlt 0.0.1
Tool to launch SAFE vaults to form a local single-section network

Currently, this tool runs vaults on localhost (since that's the default if no IP address is given to the vaults)

USAGE:
    safe-nlt [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                
            Prints help information

    -y, --vaults-verbosity    
            Verbosity level for vaults logs (default: INFO)

    -V, --version             
            Prints version information

    -v, --verbosity           
            Verbosity level for this tool


OPTIONS:
    -i, --interval <interval>        
            Interval in seconds between launching each of the vaults [default: 5]

        --ip <ip>
            IP used to launch the vaults with

    -n, --num-vaults <num-vaults>    
            Number of vaults to spawn with the first one being the genesis. This number should be greater than 0
            [default: 8]

    -p, --vault-path <vault-path>    
            Path where to locate safe_vault/safe_vault.exe binary. The SAFE_VAULT_PATH env var can be also used to set
            the path [env: SAFE_VAULT_PATH=]

    -d, --vaults-dir <vaults-dir>    
            Path where the output directories for all the vaults are written [default: ./vaults]
```

## License

This SAFE Network tool is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
