import click
import os
from pathlib import Path

# Submodules
import update as update_routines
import setup as setup_routines

MONITOR_BINARY_DOWNLOAD = "https://github.com/probe-rs/hive-software/releases/latest"
LATEST_MONITOR_VERSION = "0.1.0"

HIVE_GROUP = "hive"
HIVE_USER = "hive"


@click.group(invoke_without_command=True)
def cli():
    """
    Utility to install the Hive testserver on the Raspberry Pi

    Please run this program as superuser
    """

    if os.geteuid() != 0:
        print("Please run this program as superuser")
        exit(1)


@cli.command()
def install():
    """Install the Hive testserver"""
    print("Hive Testserver installer\n")

    print("In order to run the testserver a new group and user with the appropriate permissions are created")
    hive_user = click.prompt(
        "Please specify the new user name", default=HIVE_USER)
    hive_group = click.prompt(
        "Please specify the new group name", default=HIVE_GROUP)

    setup_hive(hive_user=hive_user, hive_group=hive_group, create=True)


@cli.command()
def autostart():
    """Setup autostart of the Testserver on each boot"""
    hive_user = click.prompt(
        "Please specify the Hive user name", default=HIVE_USER)

    print("In order to configure autostart the locations of the folders containing the toolchain binaries need to be known.")
    arm_path = click.prompt(text="ARM-Toolchain binary folder path")
    arm_path = os.path.abspath(arm_path)
    if not os.path.exists(arm_path):
        print(f"Provided path '{arm_path}' does not exist.")
        exit(1)

    riscv_path = click.prompt(text="RISCV-Toolchain binary folder path")
    riscv_path = os.path.abspath(riscv_path)
    if not os.path.exists(riscv_path):
        print(f"Provided path '{riscv_path}' does not exist.")
        exit(1)

    setup_routines.setup_autostart(
        username=hive_user, arm_toolchain_path=arm_path, riscv_toolchain_path=riscv_path)


@cli.command()
def update():
    """Updates an existing installation with the newest Hive testserver version"""

    update_routines.check_version()

    print("In order to run the update process please specify the used user and group names for the Hive testserver")
    hive_user = click.prompt(
        "Please specify the Hive user name", default=HIVE_USER)
    hive_group = click.prompt(
        "Please specify the Hive group name", default=HIVE_GROUP)

    setup_hive(hive_user=hive_user, hive_group=hive_group, create=False)


def setup_hive(hive_user: str, hive_group: str, create: bool):
    """Run the whole setup process. If create is True attempts to create a new Hive install. If False attempts to update an existing install."""
    setup_routines.setup_monitor(create=False)

    setup_routines.setup_group(groupname=hive_group, create=False)

    setup_routines.setup_user(
        username=hive_user, groupname=hive_group, create=False)

    setup_routines.setup_hardware()

    setup_routines.setup_os()

    setup_routines.setup_storage()

    reboot_promt()


def reboot_promt():
    """Notifies the user that a reboot is necessary to apply changes and reboots the system if the user wishes to do so."""
    print("In order to apply the changes the system needs to be restarted.")
    restart = click.prompt("Restart now?", default=True)

    if restart:
        os.system("reboot")


if __name__ == '__main__':
    cli()