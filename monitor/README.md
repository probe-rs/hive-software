# Monitor 
The monitor runs the Hive testrack and acts as testserver. It is the heart of Hive as it does all the management of the testrack and handles the communication with the outside world.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [database](./src/database/) | Functions related to the handling of the Hive DB |
| [mode](./src/mode/) | Contains all monitor execution modes. Currently only standalone mode is implemented |
| [tasks](./src/tasks/) | The monitor task manager and runner. Handles incoming tasks such as a test task, executes those tasks and returns the execution status and results |
| [testprogram](./src/testprogram/) | Handles all testprograms. Does building and linking of those binaries, determines address ranges which need to be linked based on the connected targets |
| [webserver](./src/webserver/) | Runs the axum webserver. Handles various things such as a static fileserver as well as graphql API for the backend and REST API for the test endpoint |
| [flash](./src/flash.rs) | Handles the flashing of the testprograms onto the targets |
| [init](./src/init.rs) | Monitor initilization routines on startup |