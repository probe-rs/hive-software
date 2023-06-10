#! /bin/sh

# Workaround to include the probe-rs testcandidate in the cross docker container without having to mount it manually and mess with cargo.toml dependency paths
cd ../

cross build --manifest-path ./hive-software/Cargo.toml --target aarch64-unknown-linux-gnu --release -p monitor
