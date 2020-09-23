# Safe Network Launch Tool
A cross platform tool to easily launch a Safe Network test section from home

## Installing the Safe Network Node

A local Safe network is bootstrapped by running several [Safe nodes](https://github.com/maidsafe/sn_node) which automatically interconnect forming a network.

In order to run your own local network you'd need to follow these steps:
- Download latest release from [sn_node releases](https://github.com/maidsafe/sn_node/releases/latest/)
- Untar/unzip the downloaded file into a directory of your choice
- Execute this tool specifying the path of the `sn_node` executable

The following is an example of how to perform this on Linux or Mac:
```shell
$ mkdir ~/my-local-network
$ cd ~/my-local-network
$ curl -O https://github.com/maidsafe/sn_node/releases/download/0.21.0/sn_node-0.21.0-x86_64-unknown-linux-musl.tar.gz
$ tar -xzvf sn_node-0.21.0-x86_64-unknown-linux-musl.tar.gz
```

## Run a local network

At current state of the [Safe project](), a single-section Safe network can be launched locally in our system. If the Safe node binary was downloaded and extracted at `~/my-local-network/` as described above, we can now launch the network using this tool following these steps:
```shell
$ git clone https://github.com/maidsafe/sn_launch_tool
$ cd sn_launch_tool
$ cargo run -- -p ~/my-local-network/sn_node -v
Launching with node executable from: ~/my-local-network/sn_node
Network size: 8 nodes
Launching genesis node (#1)...
Genesis node contact info: ["127.0.0.1:59303"]
Launching node #2...
Launching node #3...
Launching node #4...
Launching node #5...
Launching node #6...
Launching node #7...
Launching node #8...
Done!
```

Once the local network is running, the connection configuration file will be already in the correct place for your applications to connect to this network, so you can simply run any application from this moment on to connect to your local network. Note that depending on the application, you may need to restart it so it uses the new connection information for your local network.

In order to shutdown a running local network, all processes instances of sn_node must be killed, e.g. on Linux or Mac you can use the `killall` command:
```shell
$ killall sn_node
```

This tool allows you to change default values to customise part of the process, you can use the `--help` flag to get a complete list of the flags and options it supports:
```shell
sn_launch_tool 0.0.1
Tool to launch Safe nodes to form a local single-section network

Currently, this tool runs nodes on localhost (since that's the default if no IP address is given to the nodes)

USAGE:
    sn_launch_tool [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                
            Prints help information

    -y, --nodes-verbosity    
            Verbosity level for nodes logs (default: INFO)

    -V, --version             
            Prints version information

    -v, --verbosity           
            Verbosity level for this tool


OPTIONS:
    -i, --interval <interval>        
            Interval in seconds between launching each of the nodes [default: 5]

        --ip <ip>
            IP used to launch the nodes with

    -n, --num-nodes <num-nodes>    
            Number of nodes to spawn with the first one being the genesis. This number should be greater than 0
            [default: 8]

    -p, --node-path <node-path>    
            Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also used to set
            the path [env: SN_NODE_PATH=]

    -d, --nodes-dir <nodes-dir>    
            Path where the output directories for all the nodes are written [default: ./nodes]
```

## License

This Safe Network tool is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
