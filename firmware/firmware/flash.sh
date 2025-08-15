#!/bin/bash
set -eux

cargo build --target thumbv8m.main-none-eabihf
sudo ~/.cargo/bin/probe-rs run --chip RP235x target/thumbv8m.main-non-eabihf/debug/mhv_dc33_fw

