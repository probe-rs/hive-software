# Hive Macro
Contains all macros used by Hive. Currently contains the macros used in Hive test modules and functions.

The macros are tested by using trybuild which ensures that it builds or fails with a meaningful error message.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [hive](./src/hive.rs) | The hive macro used to annotate the top-level Hive test module. It injects code that is required by the testfunctions. |
| [hive_test](./src/hive_test.rs) | The hive_test macro to annotate a testfunction. Also injects code that is required by Hive to collect the testfunctions and ensures that the testfunction syntax stays consistent |