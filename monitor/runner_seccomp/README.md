# Runner Seccomp Filter
This code generates the seccomp BPF filter used by the bubblewrap binary to sandbox the runner.

The code is using libseccomp to generate the filters in a user friendly way. In order to be able to compile it, please install libseccomp first (https://github.com/seccomp/libseccomp).

The resulting binary generates bpf files with the defined filter as default. If a human readable form is desired, instead of bpf, run the binary with the 'human' argument.