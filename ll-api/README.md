# LL API
The low-level API is an abstraction level over the hardware in the Hive testrack using embedded-hal.

## Crate Modules
A brief overview on what each module in this crate is supposed to do:
| Module | Description |
| --- | --- |
| [expander_gpio](./src/expander_gpio/) | Contains abstractions over the IO-Expander GPIO pins which are present on each target-stack-shield |
| [rpi_gpio](./src/rpi_gpio/) | Contains abstractions over the Raspberry Pi GPIO pins which are connected to the Hive shields as testchannels |