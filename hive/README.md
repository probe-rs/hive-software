# Hive CLI
The CLI used to test probe-rs using the Hive testserver. It allows you to connect to a testserver, view its capabilities (Connected probes and targets) and run tests on the testserver.

## Installation
In order to successfully run, the Hive CLI needs access to the Cross binary. If Cross is not yet installed on your system, please install it and make sure it is accessible to the Hive CLI. Instructions on how to install Cross can be found [here](https://github.com/cross-rs/cross).

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [config](./src/config.rs) | The cli uses a config file to keep configuration persistent. The logic for that is implemented in this module |
| [models](./src/models.rs) | Contains data models used in the CLI that also implement De- Serialize to be able to be stored in the config |
| [validate](./src/validate.rs) | Performs custom input validation for various commands |
| [client](./src/client.rs) | Builds the clients for the http and websocket connections used by various commands |
| [workspace](./src/workspace.rs) | Manages the workspace of the cli which is used to build the runner binary based on the user defined test functions |
| [subcommands](./src/subcommands/) | Contains each CLI subcommand and its logic |