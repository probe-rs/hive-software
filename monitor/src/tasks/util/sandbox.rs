//! This module provides a way to sandbox the runner binary using [bubblewrap](https://github.com/containers/bubblewrap)
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::process::{Child, Command, Stdio};

use command_fds::{CommandFdExt, FdMapping};

const SECCOMP_BPF_FD: i32 = 25;

pub struct Sandbox {
    seccomp_file: File,
}

/// Creates and runs the provided runner binary in a secure bubblewrap sandbox which limits access to files and syscalls
impl Sandbox {
    pub fn new(seccomp_path: &str) -> Self {
        // Get runner seccomp FD to use bubblewrap sandbox with seccomp
        let seccomp_file = OpenOptions::new().read(true).write(false).open(seccomp_path).expect("Failed to open runner seccomp rule file. This is likely caused by a configuration issue or a corrupted installation.");

        Self { seccomp_file }
    }

    /// Runs the runner binary in the sandbox and returns the resulting child process
    pub fn run(
        &self,
        runner_binary_path: &str,
        restricted_uid: &str,
        restricted_gid: &str,
    ) -> Child {
        Command::new("bwrap").args([
        "--die-with-parent", "--new-session",
        // Add runner seccomp filter
        "--seccomp", &SECCOMP_BPF_FD.to_string(),
        // Unshare all namespaces and run under restricted user/group id
        "--unshare-all", "--uid", restricted_uid, "--gid", restricted_gid,
        // Bind library folder for usage of shared objects used by runner binary
        "--ro-bind", "/lib/", "/lib/",
        "--ro-bind", "/usr/lib/debug/", "/usr/lib/debug/",
        // Bind required ressources in /etc
        "--ro-bind", "/etc/localtime", "/etc/localtime",
        "--ro-bind", "/etc/ld.so.cache", "/etc/ld.so.cache",
        "--ro-bind-try", "/etc/ld.so.preload", "/etc/ld.so.preload",
        // Bind required ressources in /proc
        "--proc", "/proc",
        "--ro-bind", "/proc/cpuinfo", "/proc/cpuinfo",
        // Bind required devices
        "--dev-bind", "/dev/i2c-1", "/dev/i2c-1",
        "--dev-bind", "/dev/bus/usb/001/", "/dev/bus/usb/001/",
        "--dev-bind", "/dev/bus/usb/002/", "/dev/bus/usb/002/",
        "--ro-bind", "/sys/bus/usb/devices/", "/sys/bus/usb/devices/",
        "--ro-bind", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb1/", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb1/",
        "--ro-bind", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb2/", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb2/",
        "--ro-bind", "/run/udev/control", "/run/udev/control",
        "--ro-bind", "/run/udev/data/", "/run/udev/data/",
        "--ro-bind", "/sys/class/hidraw", "/sys/class/hidraw",
        // Bind log as rw so runner can save logs
        "--bind", "./data/logs/", "./data/logs/",
        // Bind testprograms as r so runner can use them to flash
        "--ro-bind", "./data/testprograms/", "./data/testprograms/",
        // Bind runner dir to get access to ipc and runner executable
        "--ro-bind", "./data/runner/", "./data/runner/",
        runner_binary_path
        ]).fd_mappings(vec![
            FdMapping { parent_fd: self.seccomp_file.as_raw_fd(), child_fd: SECCOMP_BPF_FD },
            ]).unwrap()
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("Failed to run bubblewrap sandbox with runner. Is the bwrap command accessible to the application?")
    }
}
