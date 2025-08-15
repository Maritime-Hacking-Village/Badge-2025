#!/bin/bash

cargo build && sudo ~/.cargo/bin/probe-rs run --chip RP235x target/thumbv8m.main-non-eabihf/debug/mhv_dc33_fw

