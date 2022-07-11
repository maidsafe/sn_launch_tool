# Safe Network Launch Tool
A cross-platform tool to easily launch a Safe Network test section from home

## Installing the Safe Network Node

A local Safe network is bootstrapped by running several [Safe nodes](https://github.com/maidsafe/safe_network/tree/main/sn_node) which automatically interconnect forming a network.

In order to run your own local network you'd need to follow these steps:
- Download the latest sn_node [release](https://github.com/maidsafe/safe_network/releases)
- Untar/unzip the downloaded file into a directory of your choice
- Execute this tool specifying the path of the `sn_node` executable

The following is an example of how to perform this on Linux or Mac:
```shell
$ mkdir ~/my-local-network
$ cd ~/my-local-network
$ curl -LO https://github.com/maidsafe/safe_network/releases/download/0.8.2-0.7.1-0.68.2-0.64.2-0.66.3-0.59.3/sn_node-0.64.2-x86_64-unknown-linux-musl.tar.gz
$ tar -xzvf sn_node-0.64.2-x86_64-unknown-linux-musl.tar.gz
```

## Run a local network

At current state of the [Safe project](), a single-section Safe network can be launched locally in our system. If the Safe node binary was downloaded and extracted at `~/my-local-network/` as described above, we can now launch the network using this tool following these steps:
```shell
$ git clone https://github.com/maidsafe/sn_launch_tool
$ cd sn_launch_tool
$ cargo run -- --local --num-nodes 15 --node-path ~/my-local-network/sn_node --nodes-dir ~/my-local-network/nodes 
2022-07-11T10:24:47.007035Z  INFO sn_launch_tool: Using RUST_LOG 'safe_network=debug'
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-genesis"
Node PID: 74681, prefix: Prefix(), name: 031758(00000011).., age: 255, connection info:
"127.0.0.1:46641"
2022-07-11T10:24:49.107884Z  INFO sn_launch_tool: Launching nodes 2..=15
Starting logging to directory: "/home/me/my-local-network/nodessn-node-2"
Node PID: 74718, prefix: Prefix(), name: 8d3072(10001101).., age: 98, connection info:
"127.0.0.1:57299"
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-3"
Node PID: 74755, prefix: Prefix(), name: 0f31a0(00001111).., age: 96, connection info:
"127.0.0.1:36822"
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-4"
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-5"
Node PID: 74819, prefix: Prefix(), name: f6bda3(11110110).., age: 94, connection info:
"127.0.0.1:50413"
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-6"
Node PID: 74856, prefix: Prefix(), name: 757076(01110101).., age: 92, connection info:
"127.0.0.1:46653"
Starting logging to directory: "/home/me/my-local-network/nodes/sn-node-7"
Node PID: 74899, prefix: Prefix(), name: b493f4(10110100).., age: 90, connection info:
"127.0.0.1:60569"
...
2022-07-11T10:26:49.537676Z  INFO sn_launch_tool: Done
```

Once the local network is running, the connection configuration file will be already in the correct place for your applications to connect to this network, so you can simply run any application from this moment on to connect to your local network. Note that depending on the application, you may need to restart it so it uses the new connection information for your local network.

In order to shutdown a running local network, all processes instances of sn_node must be killed, e.g. on Linux or Mac you can use the `killall` command:
```shell
$ killall sn_node
```

This tool allows you to change default values to customise part of the process, you can use the `--help` flag to get a complete list of the flags and options it supports:
```shell
sn_launch_tool 0.10.0
Tool to launch Safe nodes to form a local single-section network

Currently, this tool runs nodes on localhost (since that's the default if no IP address is given to
the nodes)

USAGE:
    sn_launch_tool [OPTIONS]

OPTIONS:
    --add
        IP used to launch the nodes with

    -d, --nodes-dir <NODES_DIR>
            Path where the output directories for all the nodes are written
            
            [default: ./nodes]

        --flame
            Run the nodes using `cargo flamegraph` (which needs to be preinstalled.) It is
            recommended to manually run `cargo flamegraph --root --bin=sn_node -- --first` to ensure
            everything is built. (This command will fail dur to insufficient args, but that's okay,
            carry testnetting w/ --flame thereafter)

    -h, --help
            Print help information

    -i, --interval <INTERVAL>
            Interval in milliseconds between launching each of the nodes
            
            [default: 100]

        --idle-timeout-msec <IDLE_TIMEOUT_MSEC>
            Interval in seconds before deeming a peer to have timed out

        --ip <IP>
            IP used to launch the nodes with

        --json-logs
            Output logs in json format for easier processing

        --keep-alive-interval-msec <KEEP_ALIVE_INTERVAL_MSEC>
            Interval in seconds between qp2p keep alive messages

    -l, --rust-log <RUST_LOG>
            RUST_LOG env var value to launch the nodes with

        --local
            Run the section locally

    -n, --num-nodes <NUM_NODES>
            Number of nodes to spawn with the first one being the genesis. This number should be
            greater than 0
            
            [env: NODE_COUNT=]
            [default: 15]

    -p, --node-path <NODE_PATH>
            Path where to locate sn_node/sn_node.exe binary. The SN_NODE_PATH env var can be also
            used to set the path
            
            [env: SN_NODE_PATH=]

    -V, --version
            Print version information

    -y, --nodes-verbosity
            Verbosity level for nodes logs (default: INFO)

```

## License

This Safe Network tool is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
