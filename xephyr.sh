#!/usr/bin/env bash

set -ex

cargo build --release

XEPHYR_DISPLAY=":2"

Xephyr -br -ac -noreset -screen 1920x1080 "${XEPHYR_DISPLAY}" &
XEPHYR_PID="$!"

export DISPLAY="${XEPHYR_DISPLAY}"

target/release/pop-cosmic &

sleep 1
xterm &

wait "${XEPHYR_PID}"
