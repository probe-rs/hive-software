# TODO

## Monitor
- [ ] Add a way to add testprograms as objectfiles, which would give some more flexibility in terms of the language used to create the testprogram. Linking still needs to be done in the Monitor. Monitor needs to be able to link all relevant sections (currently only .data and .text are implemented)
- [X] Flashing the testbinaries onto certain testprograms still fails on targets where it should not fail. This might be related to using connect-under-reset. Certain STM appear to require it, while nRF should not have connect-under-reset. Either automatically determine what is required, let the user define it or even just retry with different reset option on failure
- [ ] Determine why flashing still fails and is not stable at all
- [ ] Add the ability to run multiple testbinaries in a single testrun. The currently most promising idea is to create a testprogram with multiple entrypoints, which then run the individual testprograms. This would not require reflashing of testbinaries during a testrun (which might be a quite expensive operation if the runner runs in a virtualised environment). In this approach the runner would set the PC to the appropriate entrypoint based on which testprogram is defined in the testfunction.
- [ ] It needs to be determined how the different operating modes are implemented with having as little redundant code as possible. Most operating modes just require changes in the external communication which would only affect the webserver.
- [X] Fix IPC server unit tests (static DB)
- [ ] ~~Check JWT-SECRET env variable strength to enforce strong secrets~~
- [ ] ~~Websocket pass auth jwt as query param instead of auth header, as browser ws libs usually don't support setting such headers. Caching is not really a concern as the token is very short lived, it needs to be determined whether axum logs urls with params to avoid logging jwt's in query params~~
- [ ] ~~Websocket verify origin, autoclose socket after certain duration~~
- [X] Add auth to backend graphql server
- [X] std::process::exit does not call drop which is a problem for the DB as dropping the db flushes the cache and makes changes persistent on the drive. A clean way to shutdown the program is required due to that.
- [X] Implement hive auth to supply jwt in http only cookie
- [X] authenticate_user Function in webserver auth is very slow, fix performance
- [ ] ~~consider switching to base64 ct encoding instead of base64 for tokens etc.~~
- [X] Implement proper initialization which blocks the monitor in case no initialization has been done, also add reinitialization which can be triggered automatically or by the user
- [X] Move all blocking functions in async context into blocking tokio tasks
- [X] Check if axum layers are properly implemented. Scope them correctly to avoid having unnecessary layers on certain routes. Check if request limiting is required on public routes.
- [X] Return data type on /test/run endpoint which can be either error or ok with testresults, the current implementation fails with a bad request status code in case an error occurred in the testmanager, which is not an actual http error but an application error.
- [X] Current DB implementation is prone to race conditions as there is no guarantee that the previously read out value is not changed by another function before the previously read value is modified and written back to the DB
- [ ] ~~get_and_init_target_address_ranges() it might be better to do this based on the DB data as the testmanager will ultimately reinitialize the hardware based on DB data and not runtime data~~
- [X] Deadlock when reloading the testrack hardware due to user request via graphql. Probably inside Testmanager
- [ ] The debugprobe info on the JLINK probe is incomplete (missing S/N and wrong identifier) when `probe::list()` is called while the probe is in use by the program. This leads to the software suggesting a third available probe to the user which does not actually exist. Might be a problem in probe-rs, needs further investigation
- [ ] Add possibility to filter targets and probes on a test request which would allow for only testing the requested targets
- [ ] Implementation of testprogram status is not sound, there should be a better solution for checking user input before handling it as actual testprogram
- [ ] Determine wheter it is necessary to sandbox the build process of the runner, as build scripts of dependencies might contain malicious code
- [ ] Execute runner in a VM, determine best ways to make VM access hardware securely
- [X] Something is not quite correct in the way WS tickets for test tasks are handled, as some test task WS connection requests yield 401 HTTP status codes with tickets which should be correct
- [ ] Add tests for new testprogram mutations
- [X] Move IPC socket file into data folder of the program
- [ ] Determine how to handle cargo workspace target folder size to not reach memory limit of the tempfs but also to avoid excessive wait times if building off clean workspace on every test request
- [X] Replace task manager busy loop with more efficient implementation
- [X] Fix test endpoint tests
- [X] Shutdown might hang if hive cli is stopped while the websocket connection is established. Probably something wrong with detecting a broken/closed websocket which should lead to abortion of a test task.

## Runner
- [X] Add a proper shutdown procedure 
- [ ] ~~Runner assumes data desync if not all connected probes have been assigned to a testchannel. This should be considered fine and not cause a data desync~~ -> Related to JLINK not showing S/N if listed during use

## Hive Backend UI
- [X] Current appollo retrylink is useless as it does not call fetch function to change headers. Write own retry function which tries to append csrf header on each retry
- [ ] Enhance log view, currently it is pretty basic and only shows the 100 latest entries it does not allow any kind of cursor which would make it possible to lazy load log entries
- [X] The backend UI is only shown on the default route / any other route returns a 404 (for example /testprograms) Figure out a way to make the vue router compatible with axum webserver to avoid such situations in the backend
- [X] Finally export the graphql schema and generate proper Typescript Types to stop all the current any etc. mess
- [ ] Add actual functionality to the testprogram view, currently it only looks nice
- [X] User store does not stay persistent between page reloads, leading to wrongly denied routes
- [X] Fix appearance of Hive Testrack. Currently the active animation will not reset. Probably due to reassigning the Konva Tween object (It then looses info on its state)
- [X] Display unhandled errors to user as snackbar
- [ ] Fix tooltips, most of them don't show at all. Might be related to using alpha/beta version of vuetify
- [X] Add status to target which matches the actual status in runtime

## Hive CLI 
- [X] Implement pretty print for test results

## Hive Macro
- [X] Make it possible to use submodules inside top-level tests module on hive macro. The HiveTestFunction struct should be passed automatically from the top-level module to the sub level modules. Ensure that the used dependencies are in the allowed list in each module
- [ ] Make module level macros able to be used inline once this is stable https://github.com/rust-lang/rust/issues/54726 this will remove the need to declare hive test modules with bodies inside the same document and allow those modules to be stored in separate files

## Hive Test
- [X] The usefulness of this crate is highly questionnable right now as it only reexports functionality from other crates. 

## Hive Setup
- [ ] Do proper error handling in shell setup script