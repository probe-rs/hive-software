# TODO

## Monitor
- [ ] Add a way to add testprograms as objectfiles, which would give some more flexibility in terms of the language used to create the testprogram. Linking still needs to be done in the Monitor. Monitor needs to be able to link all relevant sections (currently only .data and .text are implemented)
- [X] Flashing the testbinaries onto certain testprograms still fails on targets where it should not fail. This might be related to using connect-under-reset. Certain STM appear to require it, while nRF should not have connect-under-reset. Either automatically determine what is required, let the user define it or even just retry with different reset option on failure
- [ ] Determine why flashing still fails and is not stable at all
- [ ] Add the ability to run multiple testbinaries in a single testrun. The currently most promising idea is to create a testprogram with multiple entrypoints, which then run the individual testprograms. This would not require reflashing of testbinaries during a testrun (which might be a quite expensive operation if the runner runs in a virtualised environment). In this approach the runner would set the PC to the appropriate entrypoint based on which testprogram is defined in the testfunction.
- [ ] It needs to be determined how the different operating modes are implemented with having as little redundant code as possible. Most operating modes just require changes in the external communication which would only affect the webserver.
- [ ] Fix IPC server unit tests (static DB)
- [ ] Check JWT-SECRET env variable strength to enforce strong secrets
- [ ] ~~Websocket pass auth jwt as query param instead of auth header, as browser ws libs usually don't support setting such headers. Caching is not really a concern as the token is very short lived, it needs to be determined whether axum logs urls with params to avoid logging jwt's in query params~~
- [ ] ~~Websocket verify origin, autoclose socket after certain duration~~
- [X] Add auth to backend graphql server