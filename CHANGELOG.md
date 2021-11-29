# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.9.4](https://github.com/maidsafe/sn_launch_tool/compare/v0.9.3...v0.9.4) (2021-11-29)


### Features

* introduce skip auto port forwarding flag ([e52b711](https://github.com/maidsafe/sn_launch_tool/commit/e52b71138eb7dd9a76e938094ae0c231b245e047))

### [0.9.3](https://github.com/maidsafe/sn_launch_tool/compare/v0.9.2...v0.9.3) (2021-11-15)

### [0.9.2](https://github.com/maidsafe/sn_launch_tool/compare/v0.9.1...v0.9.2) (2021-11-04)


### Features

* provide a `--local` flag for joining network ([d48edfc](https://github.com/maidsafe/sn_launch_tool/commit/d48edfc138554905279480acca03d9fc309a26ce))

### [0.9.1](https://github.com/maidsafe/sn_launch_tool/compare/v0.9.0...v0.9.1) (2021-11-04)


### Features

* add a flag to enable JSON logs on `sn_node` ([024d1e3](https://github.com/maidsafe/sn_launch_tool/commit/024d1e385b39227ad6a4fea8ee7552dca878caca))

## [0.9.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.8.1...v0.9.0) (2021-10-27)


### ⚠ BREAKING CHANGES

* The default value for `--keep-alive-interval-msec` if
unspecified has changed to `None`, which will not pass any flag to
`sn_node`, meaning the upstream default will apply.

* don't specify keep-alive-interval by default ([aad3826](https://github.com/maidsafe/sn_launch_tool/commit/aad3826459da234e43199c90f90edeabdcb04e59))

### [0.8.1](https://github.com/maidsafe/sn_launch_tool/compare/v0.8.0...v0.8.1) (2021-10-26)

## [0.8.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.7.0...v0.8.0) (2021-09-22)


### ⚠ BREAKING CHANGES

* launch args interval is now in millis, and default is 100

### Features

* use millis for interval ([6763063](https://github.com/maidsafe/sn_launch_tool/commit/6763063100b1bde7f75e6b8f4818d46ecb9de4e2))

## [0.7.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.6.0...v0.7.0) (2021-09-09)


### ⚠ BREAKING CHANGES

* For genesis nodes, `sn_node` will now be invoked with
`--first` without a value, which is not compatible with older versions
of `sn_node`.

* Update for changes to `sn_node` CLI ([b7d35f6](https://github.com/maidsafe/sn_launch_tool/commit/b7d35f648effe9fe6272b747e4e267c9b0b8423f))

## [0.6.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.5.0...v0.6.0) (2021-09-09)


### ⚠ BREAKING CHANGES

* this now expects the contacts info file to contain both the peer addreses and genesis key in hex format.

### Features

* read genesis key from contacts info file and pass it as arg to joining nodes ([ffa48b3](https://github.com/maidsafe/sn_launch_tool/commit/ffa48b30cf734144398640b3be29e37e15c50261))

## [0.5.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.4.0...v0.5.0) (2021-09-02)


### ⚠ BREAKING CHANGES

* The `run`, `run_with`, `join`, and `join_with`
functions have been removed. Instead use `Launch::run` or `Join::run`.
These structs can be constructed via the `StructOpt` constructors.

### Bug Fixes

* Use `Launch::is_local` to skip IGD ([173fb4b](https://github.com/maidsafe/sn_launch_tool/commit/173fb4bdcabb2b76ee27cd653f9674ba9db73c90))


* Change API to use argument structs as entrypoints ([78627fa](https://github.com/maidsafe/sn_launch_tool/commit/78627fafe7c640482cd9626f087493b571be3e8f))

## [0.4.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.3.2...v0.4.0) (2021-09-01)


### ⚠ BREAKING CHANGES

* The `verbosity` argument has been removed.
* The public functions `run`, `run_with`, `join`, and
`join_with` all now return `eyre::Result<()>`, rather than `Result<(),
String>`.

* Use `eyre` for errors ([b944a92](https://github.com/maidsafe/sn_launch_tool/commit/b944a92783646f1335a6ddf3cec2de5fbc244477))
* Use `tracing` for all messages ([068eedd](https://github.com/maidsafe/sn_launch_tool/commit/068eedd73a51abdb00a411526604167508ee6868))

### [0.3.2](https://github.com/maidsafe/sn_launch_tool/compare/v0.3.1...v0.3.2) (2021-07-19)


### Features

* Adding nodes ([19e85a5](https://github.com/maidsafe/sn_launch_tool/commit/19e85a50ed820fcca3900ce37ee3ae5f073ab693))


### Bug Fixes

* added nodes use new log dirs ([da316a3](https://github.com/maidsafe/sn_launch_tool/commit/da316a3a338a16ebf569b63e9ec7c8e72dacfed7))

### [0.3.1](https://github.com/maidsafe/sn_launch_tool/compare/v0.3.0...v0.3.1) (2021-07-01)


### Features

* **args:** add clear-data arg ([7fef81a](https://github.com/maidsafe/sn_launch_tool/commit/7fef81a99fc4052f8b00f977197b7fbee0d7c571))
* **args:** take local and public ip args ([98b7930](https://github.com/maidsafe/sn_launch_tool/commit/98b793048bf43c403563d96f8d62fd10b46ed4b9))

## [0.3.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.2.3...v0.3.0) (2021-06-23)


### ⚠ BREAKING CHANGES

* safe_network not sn_node for logging

### Features

* update default log for safe_network ([39c179b](https://github.com/maidsafe/sn_launch_tool/commit/39c179b2622ffee12a66e489a8b2e7b41a126cda))

### [0.2.3](https://github.com/maidsafe/sn_launch_tool/compare/v0.2.2...v0.2.3) (2021-06-17)

### [0.2.2](https://github.com/maidsafe/sn_launch_tool/compare/v0.2.1...v0.2.2) (2021-06-08)

### [0.2.1](https://github.com/maidsafe/sn_launch_tool/compare/v0.2.0...v0.2.1) (2021-04-26)


### Features

* **args:** make aware of max capacity argument ([f9e241e](https://github.com/maidsafe/sn_launch_tool/commit/f9e241ed5f7e9331075901dc1ca32af17cfec168))

## [0.2.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.1.0...v0.2.0) (2021-04-05)


### ⚠ BREAKING CHANGES

* **args:** this uses the args for the latest sn_node

* **args:** update using the latest node args ([557c950](https://github.com/maidsafe/sn_launch_tool/commit/557c9507ce32d170aa3820f321fc40127a04fdf3))

## [0.1.0](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.20...v0.1.0) (2021-04-01)


### ⚠ BREAKING CHANGES

* **args:** the new argument is for the latest version of sn_node

### update

* **args:** update args passed for the latest version of sn_node ([1fd88fb](https://github.com/maidsafe/sn_launch_tool/commit/1fd88fb099e0fd533b0a1f48af6c9b1d2341a8f2))

### [0.0.20](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.19...v0.0.20) (2021-03-31)

### [0.0.19](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.18...v0.0.19) (2021-03-03)

### [0.0.18](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.17...v0.0.18) (2021-02-25)

### [0.0.17](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.16...v0.0.17) (2021-02-25)


### Bug Fixes

* **ip:** changes to support latest version of sn_node which renamed --ip arg to --local-ip ([5d7a000](https://github.com/maidsafe/sn_launch_tool/commit/5d7a000193f980e52b765b6c8106bb3436d2ab69))

### [0.0.16](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.15...v0.0.16) (2021-02-18)


### Bug Fixes

* set rust_log correctly ([68f64e0](https://github.com/maidsafe/sn_launch_tool/commit/68f64e097016dc06e3603141d68b14d39470d2fe))

### [0.0.15](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.14...v0.0.15) (2021-02-09)


### Features

* **join:** allow to join to network with a list of addresses rather than a single one ([57ee3b0](https://github.com/maidsafe/sn_launch_tool/commit/57ee3b0979c991e336887709a555d10bb1dd2b96))

### [0.0.14](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.13...v0.0.14) (2021-02-04)


### Features

* print sn_node version when verbosity is set ([10a226b](https://github.com/maidsafe/sn_launch_tool/commit/10a226bae26cab168458a757447f05959e27c525))

### [0.0.13](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.12...v0.0.13) (2021-02-03)

### [0.0.12](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.11...v0.0.12) (2021-02-01)

### [0.0.11](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.10...v0.0.11) (2021-01-27)


### Features

* allow override of RUST_LOG env var for ndoe startup ([99daa5b](https://github.com/maidsafe/sn_launch_tool/commit/99daa5b9b6af1a082da03d4678c76b8ff563d4dd))

### [0.0.10](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.9...v0.0.10) (2021-01-26)

### [0.0.9](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.8...v0.0.9) (2021-01-21)


### Features

* improve network lag w/ node startup options ([6cecb63](https://github.com/maidsafe/sn_launch_tool/commit/6cecb63c24ef91acae8a0f22d4bb4f9b29c19539))
* log common args on network launch ([57322b2](https://github.com/maidsafe/sn_launch_tool/commit/57322b2c4a2e91278722b3df94654b6672a871b0))

### [0.0.8](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.7...v0.0.8) (2021-01-14)

### [0.0.7](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.6...v0.0.7) (2020-12-26)


### Bug Fixes

* **publish:** fix publish command by adding flag ([4b97eca](https://github.com/maidsafe/sn_launch_tool/commit/4b97ecaf09f734fe87b25b8ec29aae2a7355989b))

### [0.0.6](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.5...v0.0.6) (2020-12-24)

### [0.0.5](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.4...v0.0.5) (2020-11-30)

### [0.0.4](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.3...v0.0.4) (2020-11-23)

### [0.0.3](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.2...v0.0.3) (2020-10-08)

### [0.0.2](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.1...v0.0.2) (2020-09-29)


### Features

* **audit:** add scheduled security audit scan ([90cc5e2](https://github.com/maidsafe/sn_launch_tool/commit/90cc5e2df5177a114c638077d3e5f0b0c164ccbc))
* **tool:** add option to join the existing network ([b33f455](https://github.com/maidsafe/sn_launch_tool/commit/b33f4556e1e20f48ceaf9dd55222a415f77fc0df))
* **tool:** support phase-2b vaults ([b1b73f0](https://github.com/maidsafe/sn_launch_tool/commit/b1b73f06f336728316e87abbd7fa71ab2723d391))


### Bug Fixes

* **tool:** use hardcoded contact ip in correct format ([6f5d277](https://github.com/maidsafe/sn_launch_tool/commit/6f5d277502078ab08acbaa1346e14c5b6167cd66))

### [0.0.1](https://github.com/maidsafe/sn_launch_tool/compare/v0.0.1...v0.0.1) (2020-02-24)
* Initial implementation
