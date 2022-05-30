FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes libusb-1.0-0-dev:arm64 libftdi1-dev:arm64 libudev-dev:arm64 && \