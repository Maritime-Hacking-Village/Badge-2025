#!/bin/bash

cargo build && sudo /home/$(whoami)/.cargo/bin/probe-rs run --chip RP235x target/thumbv8m.main-none-eabihf/debug/mhv_dc33_fw

