# Controller
The controller contains shared functionality that is used by the runner and monitor binary crates.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [hardware](./src/hardware/) | Contains another abstraction over the ll-api hardware interfaces to further simplify the usage of the testrack hardware |
| [logger](./src/logger/) | Contains the logging functionality |