#!/bin/sh
# Capture the IOKit + system_profiler data plugcheck's macOS probe parses.
# Run with a USB-C charger AND a data cable/dock/SSD connected for a useful
# "occupied" fixture. Safe to run repeatedly.
set -e
DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/tests/fixtures"
mkdir -p "$DIR"
# AppleHPMDevice subtree holds the Type-C port controllers (AppleTCControllerType1x)
# as children, with cable / orientation / transport / power state decoded.
ioreg -a -r -l -c AppleHPMDevice > "$DIR/ioreg_ports.plist"
system_profiler -json SPUSBDataType SPThunderboltDataType > "$DIR/system_profiler.json"
echo "wrote $DIR/ioreg_ports.plist ($(wc -c < "$DIR/ioreg_ports.plist") bytes)"
echo "wrote $DIR/system_profiler.json ($(wc -c < "$DIR/system_profiler.json") bytes)"
