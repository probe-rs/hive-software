# Hive DB
The database used by the monitor. 

The DB is based on [sled](https://github.com/spacejam/sled) which is an embedded database. This crate implements various wrappers for the sled DB which introduce abstractions (Typed keys for example) and implement CBOR encoding for the data storage and any associated functions.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [db](./src/db.rs) | Implements CBOR De-/Serialization for sled database functions |
| [keys](./src/keys.rs) | Makes it possible to use typed keys with the CBOR DB functions, which avoid mistakes such as trying to read out a wrong data type from a DB key |