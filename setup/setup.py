from re import sub
import requests
from yaspin import yaspin
import subprocess
import configparser
import os
import tarfile

import update


def setup_group(groupname: str, create: bool):
    """Setup the group used to run the Hive testserver"""
    if create:
        try:
            res = subprocess.run(["groupadd", groupname],
                                 check=True, capture_output=True)
            print(f"Successfully created Hive group '{groupname}'")
        except subprocess.CalledProcessError:
            reason = res.stderr.decode("utf-8", "ignore")
            print(f"Failed to create Hive group: {reason}")
            exit(1)
        except Exception as e:
            print(f"Failed to create Hive group: {e}")
            exit(1)


def setup_user(username: str, groupname: str, create: bool):
    """Setup the user used to run the Hive testserver"""
    if create:
        try:
            res = subprocess.run(
                ["adduser",  "--system", username], check=True, capture_output=True)
            print(f"Successfully created Hive user '{username}'")
        except subprocess.CalledProcessError:
            reason = res.stderr.decode("utf-8", "ignore")
            print(f"Failed to create Hive user: {reason}")
            exit(1)
        except Exception as e:
            print(f"Failed to create Hive user:  {e}")
            exit(1)

    try:
        res = subprocess.run(
            ["usermod", "-G", f"{groupname},plugdev,i2c", username], check=True, capture_output=True)
        print(f"Added groups to Hive user '{username}'")
    except subprocess.CalledProcessError as e:
        reason = res.stderr.decode("utf-8", "ignore")
        print(f"Failed to add groups to Hive user '{username}': {reason}")
        exit(1)
    except Exception as e:
        print(f"Failed to add groups to Hive user '{username}': {e}")
        exit(1)


HIVE_CONFIG_TOP_SEPARATOR = "# ==============Hive Configuration=============="
HIVE_CONFIG_BOTTOM_SEPARATOR = "# ==============End of Hive Configuration=============="


def setup_hardware():
    """Configures the hardware of the Raspberry Pi 4B controller for usage as Hive testrack. Overwrites existing Hive configurations."""
    # Disable login shell on serial port and enable serial port
    try:
        res = subprocess.run(
            ["raspi-config", "nonint", "do_serial", "2"], check=True, capture_output=True)
        print("Disabled Login shell via serial port, enabled serial port")
    except subprocess.CalledProcessError:
        reason = res.stderr.decode("utf-8", "ignore")
        print(
            f"Failed to disable Login shell via serial port and enable serial port: {reason}")
        exit(1)
    except Exception as e:
        print(
            f"Failed to disable Login shell via serial port and enable serial port: {e}")
        exit(1)

    # Enable I2C bus
    try:
        res = subprocess.run(
            ["raspi-config", "nonint", "do_i2c", "0"], check=True, capture_output=True)
        print("Enabled I2C bus")
    except subprocess.CalledProcessError:
        reason = res.stderr.decode("utf-8", "ignore")
        print(f"Failed to enable I2C bus: {reason}")
        exit(1)
    except Exception as e:
        print(f"Failed to enable I2C bus: {e}")
        exit(1)

    # Set I2C bus speed, disable bluetooth and enable UART interfaces
    try:
        config_file = open("/boot/config.txt", "Wrt")

        file_lines = config_file.readlines()

        try:
            hive_config_top_separator_idx = file_lines.index(
                HIVE_CONFIG_TOP_SEPARATOR)
            hive_config_bottom_separator_idx = file_lines.index(
                HIVE_CONFIG_BOTTOM_SEPARATOR)
            print(
                "Found existing Hive configuration in '/boot/config.txt'. Overwriting...")

            file_lines = file_lines[:hive_config_top_separator_idx].extend(
                file_lines[hive_config_bottom_separator_idx:])
        except:
            pass

        # Add actual configuration file lines
        file_lines.extend([
            HIVE_CONFIG_TOP_SEPARATOR,
            "[all]",
            "dtparam=i2c_baudrate=400000",
            "dtparam=disable-bt",
            "dtoverlay=uart3",
            "dtoverlay=uart0",
            "dtoverlay=uart4",
            "dtoverlay=uart5",
            HIVE_CONFIG_BOTTOM_SEPARATOR
        ])

        if len(file_lines) > 98:
            print("WARNING: Config file '/boot/config.txt' contains more than 98 lines.\nIt is very likely that some hardware configuration beyond the 98th line is entirely ignored.\nPlease manually clean up your config file to avoid any misconfiguration.\nConsult https://www.raspberrypi.com/documentation/computers/config_txt.html#file-format for more info.")

        config_file.write(file_lines)

        print("Disabled Bluetooth, enabled all required UART interfaces and set I2C speed to 400kHz")
    except Exception as e:
        print(
            f"Failed to edit Raspberry Pi hardware configuration in '/boot/config.txt': {e}")
        exit(1)


HIVE_LOGS = "./data/logs"
HIVE_RUNNER_BINARY = "./data/runner"
ASSEMBLER_WORKSPACE = "./data/assembler_workspace"


def setup_storage():
    # Create all required tempfs RAM-Disks
    try:
        fstab_file = open("/etc/fstab", "Wrt")

        file_lines = fstab_file.readlines()

        try:
            hive_config_top_separator_idx = file_lines.index(
                HIVE_CONFIG_TOP_SEPARATOR)
            hive_config_bottom_separator_idx = file_lines.index(
                HIVE_CONFIG_BOTTOM_SEPARATOR)
            print("Found existing Hive configuration in '/etc/fstab'. Overwriting...")

            file_lines = file_lines[:hive_config_top_separator_idx].extend(
                file_lines[hive_config_bottom_separator_idx:])
        except:
            pass

        log_path = os.path.abspath("./data/logs")
        runner_binary_path = os.path.abspath("./data/runner")
        assembler_workspace_path = os.path.abspath(
            "./data/assembler_workspace")

        # Add actual configuration file lines
        file_lines.extend([
            HIVE_CONFIG_TOP_SEPARATOR,
            f"tmpfs {log_path} tmpfs nodev,nouser,gid=$hive_gid,mode=775,noexec,noatime,rw,size=100M 0 0",
            f"tmpfs {runner_binary_path} tmpfs nodev,nouser,gid=$hive_gid,mode=774,exec,noatime,rw,size=400M 0 0",
            f"tmpfs {assembler_workspace_path} tmpfs nodev,nouser,gid=$hive_gid,mode=774,noexec,noatime,rw,size=10M 0 0",
            HIVE_CONFIG_BOTTOM_SEPARATOR
        ])

        fstab_file.write(file_lines)

        print("Created all RAM-Disks used for Hive")
    except Exception as e:
        print(f"Failed to edit fstab configuration in '/etc/fstab': {e}")
        exit(1)


def setup_os():
    """Configures the OS for use as Hive Testrack. Overwrites any existing settings."""
    # Disable journal logging to flash. Instead log to tempfs
    try:
        journal_config = open("/etc/systemd/journald.conf", "Wrt")

        lines = journal_config.readlines()

        for line in lines:
            if line.startswith("#Storage=") or line.startswith("Storage="):
                line = "Storage=volatile"

        journal_config.write(lines)
        print("Disabled journal logging to Flash. The journal is now located at tempfs '/run/log'")
    except Exception as e:
        print(f"Failed to configure journal logging: {e}")
        exit(1)


def setup_monitor(create: bool):
    """Downloads the Testserver data and installs it on the home directory of the Hive user"""
    # TODO: probably download tar archive to have filesystem already in place, need a different function for update runs then

    try:
        latest_version = update.get_latest_version()

        with yaspin(text="Downloading monitor binary...", color="blue"):
            res = requests.get(
                f"https://github.com/probe-rs/hive-software/releases/download/v{latest_version}/monitor")

        if res.ok:
            try:
                tar = open("./data.tar.xz", "wb")
                tar.write(res.content)

                tar = tarfile.open("./data.tar.xz", "r")

                if create:
                    tar.extractall()
                else:
                    monitor = tar.extractfile("./monitor")
                    monitor_file = open("./monitor", "W")
                    monitor_file.write(monitor)

                    tar.extract("./data/webserver/static/")

                os.remove("./data.tar.xz")
            except Exception as e:
                print(f"Failed to extract downloaded tar archive: {e}")
                exit(1)
        else:
            print(
                f"Failed to download the monitor binary with version {latest_version}: {res.status_code} {res.reason}")
            exit(1)
    except Exception as e:
        print(
            f"Failed to download the monitor binary with version {latest_version}: {e}")
        exit(1)

    print("Sucessfully installed Hive files")


def setup_autostart(username: str, arm_toolchain_path: str, riscv_toolchain_path: str):
    """Sets up autostart functionality of Hive using systemd"""
    try:
        hive_service_file = open("/etc/systemd/system/hive.service", "Wrt")

        hive_service_file.write((
            "[Unit]\n"
            "Description=Hive Testserver Service\n\n"
            "[Service]\n"
            "Type=simple\n"
            f"Environment=\"PATH={arm_toolchain_path}:{riscv_toolchain_path}:/usr/bin\"\n"
            f"WorkingDirectory=/home/hive/\nExecStart=runuser -u {username} /home/hive/monitor\n"
            "Restart=on-failure\n"
            "RestartSec=30\n"
            "KillMode=mixed\n\n"
            "[Install]\n"
            "WantedBy=multi-user.target"))

        print("Created systemd unit file 'hive.service'")
    except Exception as e:
        print(f"Failed to edit fstab configuration in '/etc/fstab': {e}")
        exit(1)

    try:
        result = subprocess.run(["systemctl", "daemon-reload"])
        result.check_returncode()
    except subprocess.CalledProcessError:
        reason = result.stderr.decode("utf-8", "ignore")
        print(f"Failed to reload systemctl daemon: {reason}")
        exit(1)
    except Exception as e:
        print(f"Failed to reload systemctl daemon: {e}")
        exit(1)

    try:
        result = subprocess.run(["systemctl", "enable", "hive"])
        result.check_returncode()
    except subprocess.CalledProcessError:
        reason = result.stderr.decode("utf-8", "ignore")
        print(f"Failed to enable hive service for autostart: {reason}")
        exit(1)
    except Exception as e:
        print(f"Failed to enable hive service for autostart: {e}")
        exit(1)

    try:
        result = subprocess.run(["systemctl", "start", "hive"])
        result.check_returncode()
    except subprocess.CalledProcessError:
        reason = result.stderr.decode("utf-8", "ignore")
        print(f"Failed to start hive service: {reason}")
        exit(1)
    except Exception as e:
        print(f"Failed to start hive service: {e}")
        exit(1)

    print("Sucessfully configured autostart for Hive. The new Hive service was automatically started. To check the health of this service please consult the Hive logs or run 'journalctl -xe' or 'systemctl status hive'")