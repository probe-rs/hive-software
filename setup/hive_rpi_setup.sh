#!/bin/bash
# This script does the automatic setup of the raspberry pi
# TODO Error handling

####################################################################
##                           Constants                            ##
####################################################################

CONFIG=/boot/config.txt
FSTAB=/etc/fstab
JOURNAL=/etc/systemd/journald.conf
HIVE_LOGS=./data/logs
HIVE_WORKSPACE=./data/workspace
HIVE_RUNNER_BINARY=./data/runner
ASSEMBLER_WORKSPACE=./data/assembler_workspace

HIVE_GROUP=hive

####################################################################
##                           Functions                            ##
####################################################################

configure_groups() {
    groupadd $HIVE_GROUP
    printf "\tCreated group for Hive with name: $HIVE_GROUP"
}

configure_hardware() {
    # Add [all] tag to CONFIG end
    if ! tail -1 $CONFIG | grep -q "\\[all\\]"; then
        sed -i -e '$a[all]' $CONFIG
    fi
    sed -i -e '$a# ==============Hive Configuration==============' $CONFIG

    # Disable Login shell via Serial, Enable Serial
    raspi-config nonint do_serial 2
    printf "\tDisabled Login-Shell via serial port, enabled serial port\n"

    # Enable I2C and set speed to 400kHz
    raspi-config nonint do_i2c 0
    sed -i -e '$adtparam=i2c_baudrate=400000' $CONFIG
    printf "\tEnabled I2C bus and set bus speed to 400kHz\n"

    # Enable all required UART interfaces
    # Enable UART0
    sed -i -e '$adtoverlay=uart3' $CONFIG
    # Enable UART1
    sed -i -e '$adtparam=disable-bt' $CONFIG
    sed -i -e '$adtoverlay=uart0' $CONFIG
    # Enable UART2
    sed -i -e '$adtoverlay=uart4' $CONFIG
    # Enable UART3
    sed -i -e '$adtoverlay=uart5' $CONFIG
    printf "\tDisabled bluetooth, enabled all required UART interfaces\n"

    sed -i -e '$a# ==============End of Hive Configuration==============' $CONFIG
}

configure_os() {
    # Disable logging of os logs to SD-Card, log to tempfs instead
    sed -i -e '/Storage=/c Storage=volatile' $JOURNAL
    printf "\tDisabled OS logging to SD-Card. Logs will be stored on tempfs /run/log"
}

configure_storage() {
    hive_gid=getent group $HIVE_GROUP | cut -d: -f3

    sed -i -e '$a# ==============Hive Configuration==============' $FSTAB

    # Create tempfs for runner and monitor logs
    mkdir -p $HIVE_LOGS
    abs_path=$(readlink -f $HIVE_LOGS)
    sed -i -e "\$atmpfs $abs_path tmpfs nodev,nouser,gid=$hive_gid,mode=775,noexec,noatime,rw,size=100M 0 0" $FSTAB
    printf "\tCreated $abs_path tempfs to store Hive logs\n"

    # Create tempfs for building the runner
    mkdir -p $HIVE_WORKSPACE
    abs_path=$(readlink -f $HIVE_WORKSPACE)
    sed -i -e "\$atmpfs $abs_path tmpfs nodev,nouser,gid=$hive_gid,mode=774,noexec,noatime,rw,size=1500M 0 0" $FSTAB
    printf "\tCreated $abs_path tempfs to use as workspace for building the runner\n"

    # Create tempfs for storing the runner binary
    mkdir -p $HIVE_RUNNER_BINARY
    abs_path=$(readlink -f $HIVE_RUNNER_BINARY)
    sed -i -e "\$atmpfs $HIVE_RUNNER_BINARY tmpfs nodev,nouser,gid=$hive_gid,mode=774,exec,noatime,rw,size=150M 0 0" $FSTAB
    printf "\tCreated $abs_path tempfs to use as tmp folder to store the runner binary\n"

    # Create tempfs for assembler
    mkdir -p $ASSEMBLER_WORKSPACE
    abs_path=$(readlink -f $ASSEMBLER_WORKSPACE)
    sed -i -e "\$atmpfs $abs_path tmpfs nodev,nouser,gid=$hive_gid,mode=774,noexec,noatime,rw,size=10M 0 0" $FSTAB
    printf "\tCreated $abs_path tempfs to use as workspace for the assembler\n"

    sed -i -e '$a# ==============End of Hive Configuration==============' $FSTAB
}

configure_user() {
    # Add user to dialout group
    usermod -a -G dialout $USER
    printf "\tAdded user to dialout group\n"

    # Add user to hive group
    usermod -a -G $HIVE_GROUP $USER
    printf "\tAdded user to hive group\n"
}

####################################################################
##                             Main                               ##
####################################################################

if [ $EUID -ne 0 ]; then
    printf "Please run this script as a superuser\n"
    exit 1
fi

printf "\033[1mSetting up Rpi for usage as Hive-Testrack\033[0m\n"

printf "Configuring groups\n"

configure_groups

printf "Configuring hardware\n"

configure_hardware

printf "\nConfiguring OS\n"

configure_os

printf "\nConfiguring storage\n"

configure_storage

printf "\nConfiguring user\n"

configure_user

printf "Finished setup for Hive. Please reboot your raspberry pi to apply the changes.\n\n"

exit 0
