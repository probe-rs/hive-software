# TODO

## Monitor

- [ ] Add a way to add testprograms as objectfiles, which would give some more flexibility in terms of the language used to create the testprogram. Linking still needs to be done in the Monitor. Monitor needs to be able to link all relevant sections (currently only .data and .text are implemented)
- [x] Flashing the testbinaries onto certain testprograms still fails on targets where it should not fail. This might be related to using connect-under-reset. Certain STM appear to require it, while nRF should not have connect-under-reset. Either automatically determine what is required, let the user define it or even just retry with different reset option on failure
- [x] Determine why flashing still fails and is not stable at all
- [ ] Add the ability to run multiple testbinaries in a single testrun. The currently most promising idea is to create a testprogram with multiple entrypoints, which then run the individual testprograms. This would not require reflashing of testbinaries during a testrun (which might be a quite expensive operation if the runner runs in a virtualised environment). In this approach the runner would set the PC to the appropriate entrypoint based on which testprogram is defined in the testfunction.
- [ ] It needs to be determined how the different operating modes are implemented with having as little redundant code as possible. Most operating modes just require changes in the external communication which would only affect the webserver.
- [x] Fix IPC server unit tests (static DB)
- [ ] ~~Check JWT-SECRET env variable strength to enforce strong secrets~~
- [ ] ~~Websocket pass auth jwt as query param instead of auth header, as browser ws libs usually don't support setting such headers. Caching is not really a concern as the token is very short lived, it needs to be determined whether axum logs urls with params to avoid logging jwt's in query params~~
- [ ] ~~Websocket verify origin, autoclose socket after certain duration~~
- [x] Add auth to backend graphql server
- [x] std::process::exit does not call drop which is a problem for the DB as dropping the db flushes the cache and makes changes persistent on the drive. A clean way to shutdown the program is required due to that.
- [x] Implement hive auth to supply jwt in http only cookie
- [x] authenticate_user Function in webserver auth is very slow, fix performance
- [ ] ~~consider switching to base64 ct encoding instead of base64 for tokens etc.~~
- [x] Implement proper initialization which blocks the monitor in case no initialization has been done, also add reinitialization which can be triggered automatically or by the user
- [x] Move all blocking functions in async context into blocking tokio tasks
- [x] Check if axum layers are properly implemented. Scope them correctly to avoid having unnecessary layers on certain routes. Check if request limiting is required on public routes.
- [x] Return data type on /test/run endpoint which can be either error or ok with testresults, the current implementation fails with a bad request status code in case an error occurred in the testmanager, which is not an actual http error but an application error.
- [x] Current DB implementation is prone to race conditions as there is no guarantee that the previously read out value is not changed by another function before the previously read value is modified and written back to the DB
- [ ] ~~get_and_init_target_address_ranges() it might be better to do this based on the DB data as the testmanager will ultimately reinitialize the hardware based on DB data and not runtime data~~
- [x] Deadlock when reloading the testrack hardware due to user request via graphql. Probably inside Testmanager
- [ ] The debugprobe info on the JLINK probe is incomplete (missing S/N and wrong identifier) when `probe::list()` is called while the probe is in use by the program. This leads to the software suggesting a third available probe to the user which does not actually exist. Might be a problem in probe-rs, needs further investigation
- [x] Add possibility to filter targets and probes on a test request which would allow for only testing the requested targets
- [ ] Implementation of testprogram status is not sound, there should be a better solution for checking user input before handling it as actual testprogram
- [ ] ~~Determine wheter it is necessary to sandbox the build process of the runner, as build scripts of dependencies might contain malicious code~~
- [x] Execute runner in a ~~VM~~ Sandbox, determine best ways to make ~~VM~~ Sandbox access hardware securely
- [x] Something is not quite correct in the way WS tickets for test tasks are handled, as some test task WS connection requests yield 401 HTTP status codes with tickets which should be correct
- [ ] Add tests for new testprogram mutations
- [x] Move IPC socket file into data folder of the program
- [ ] ~~Determine how to handle cargo workspace target folder size to not reach memory limit of the tempfs but also to avoid excessive wait times if building off clean workspace on every test request~~
- [x] Replace task manager busy loop with more efficient implementation
- [x] Fix test endpoint tests
- [x] Shutdown might hang if hive cli is stopped while the websocket connection is established. Probably something wrong with detecting a broken/closed websocket which should lead to abortion of a test task.
- [ ] Link binaries of the active Testprogram into a ramdisk to avoid killing the SD-Card
- [x] Currently the Address range determining functions compare based on the address range but should compare only the start of the range
- [x] `TaskRunner` contains duplicate logic for hardware reinitialization + hardware reinit function resyncs testprogram binaries for no reason
- [ ] Replace ring dependency with other hmac lib

## Runner

- [x] Add a proper shutdown procedure
- [ ] ~~Runner assumes data desync if not all connected probes have been assigned to a testchannel. This should be considered fine and not cause a data desync~~ -> Related to JLINK not showing S/N if listed during use
- [x] Modify backtrace to actually show correct backtrace of the test function
- [x] Current Implementation of caching Test results does not make much sense. The global results vector can be removed and a vector can be directly created from the mpsc receiver once all test threads have completed.
- [x] Add Test-timeout to avoid having testfunctions running forever

## Hive Backend UI

- [x] Current appollo retrylink is useless as it does not call fetch function to change headers. Write own retry function which tries to append csrf header on each retry
- [ ] Enhance log view, currently it is pretty basic and only shows the 100 latest entries it does not allow any kind of cursor which would make it possible to lazy load log entries
- [x] The backend UI is only shown on the default route / any other route returns a 404 (for example /testprograms) Figure out a way to make the vue router compatible with axum webserver to avoid such situations in the backend
- [x] Finally export the graphql schema and generate proper Typescript Types to stop all the current any etc. mess
- [ ] Add actual functionality to the testprogram view, currently it only looks nice
- [x] User store does not stay persistent between page reloads, leading to wrongly denied routes
- [x] Fix appearance of Hive Testrack. Currently the active animation will not reset. Probably due to reassigning the Konva Tween object (It then looses info on its state)
- [x] Display unhandled errors to user as snackbar
- [x] Fix tooltips, most of them don't show at all. Might be related to using alpha/beta version of vuetify
- [x] Add status to target which matches the actual status in runtime

## Hive CLI

- [x] Implement pretty print for test results
- [x] Implement proper checks if a git repo already exists in the cached workspace and if it is up to date with the requested version

## Hive Macro

- [x] Make it possible to use submodules inside top-level tests module on hive macro. The HiveTestFunction struct should be passed automatically from the top-level module to the sub level modules. Ensure that the used dependencies are in the allowed list in each module
- [ ] Make module level macros able to be used inline once this is stable https://github.com/rust-lang/rust/issues/54726 this will remove the need to declare hive test modules with bodies inside the same document and allow those modules to be stored in separate files
- [x] Allow super dependencies from submodules to top-level test module
- [x] Import Inventory crate with macro
- [ ] Update Darling & Syn deps

## Hive Test

- [x] The usefulness of this crate is highly questionnable right now as it only reexports functionality from other crates.

## Comm Types

- [ ] Make Hive Defines more ergonomic by allowing anything that implements HiveDefine to become registered automatically in the registry instead of doing this in the monitor by hand
- [ ] Hive Defines make more sense to be placed in Hive Test or Monitor crate. Currently they are placed in this crate to avoid circular dependencies (Because of usage in IPC Message enum)

## Hive Setup

- [x] Do proper error handling in shell setup script
