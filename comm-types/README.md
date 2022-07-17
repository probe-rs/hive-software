# Comm types
This crate contains various types that are used accross multiple crates in this workspace mainly for IPC and communication with the outside world. Most (probably all) types in this crate implement De-/Serialize. The crate is heavily feature-gated so each crate can define what it needs and what not.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [defines](./src/defines/) | Contains the Hive Defines. Defines are a way to inject variables into a testprogram and test against that variable in Hive tests |
| [auth](./src/auth.rs) | Types used for webserver authorization and authentication |
| [cbor](./src/cbor.rs) | Implements functionality to use the axum framework with CBOR |
| [hardware](./src/hardware.rs) | Types related to the Hive hardware which need to be De-/Serializable for use in communication or the DB |
| [ipc](./src/ipc.rs) | Types used for IPC between the monitor and the runner |
| [test](./src/test.rs) | Types related to Hive tests such as test results, status etc. |