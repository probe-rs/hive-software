# Runner 
The runner is a binary which runs the Hive tests with which it was built with. The runner is usually built automatically by using the Hive CLI and then sent to the monitor which runs it and reports the testresults back.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [comm](./src/comm/) | Handles the IPC communication with the monitor |
| [hive](./src/hive/) | Contains the Hive tests which are defined in probe-rs. The files in there just represent a dummy implementation for testing/building. In normal operation those files are replaced by the tests in probe-rs before building |
| [init](./src/init.rs) | Runner initialization routines |
| [test](./src/test.rs) | Runs the Hive tests on the hardware and collects/reports the results |