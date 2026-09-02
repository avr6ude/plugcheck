#!/bin/sh
# Capture the IOKit data plugcheck's macOS probe parses. Run with a charger +
# data cable/dock connected for a useful "occupied" fixture; the probe reads:
#   ioreg -a -r -l -c AppleHPMDevice   -> Type-C port controllers
#   ioreg -a -l -p IOUSB               -> USB device tree
#   system_profiler SPThunderboltDataType -> TB topology
set -e
DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/tests/fixtures"
mkdir -p "$DIR"
SUFFIX="${1:-occupied}"
ioreg -a -r -l -c AppleHPMDevice          > "$DIR/ioreg_ports_${SUFFIX}.plist"
ioreg -a -l -p IOUSB                      > "$DIR/ioreg_iousb_${SUFFIX}.plist"
system_profiler -json SPThunderboltDataType > "$DIR/system_profiler.json"
for f in "ioreg_ports_${SUFFIX}.plist" "ioreg_iousb_${SUFFIX}.plist" system_profiler.json; do
  echo "wrote $DIR/$f ($(wc -c < "$DIR/$f") bytes)"
done
