# hivecross dockerfile
#
# This is a custom dockerfile used for cross-compiling the Hive executables using cross-rs.
# This is needed to install certain libraries which are required to build probe-rs, for more information, see probe-rs readme

FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes libusb-1.0-0-dev:arm64 libftdi1-dev:arm64 libudev-dev:arm64 && \