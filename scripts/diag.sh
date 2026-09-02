#!/bin/sh
# Run WITH the problem device (hub, SSD, dock) connected.
# Dumps everything plugcheck's probe looks at, plus what it currently misses.
set -e
OUT="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/tests/fixtures"
mkdir -p "$OUT"

# 1. USB tree on its own (no SPThunderboltDataType — that arg can suppress USB)
system_profiler -json SPUSBDataType > "$OUT/diag_usb_only.json"
# 2. USB + TB together (what the probe currently runs)
system_profiler -json SPUSBDataType SPThunderboltDataType > "$OUT/system_profiler.json"
# 3. Port controllers with cable/transport state
ioreg -a -r -l -c AppleHPMDevice > "$OUT/ioreg_ports.plist"
# 4. Raw IOKit USB device tree (where chained devices actually live on AS Macs)
ioreg -a -l -p IOUSB > "$OUT/ioreg_iousb.plist" 2>/dev/null || \
  ioreg -a -l -c IOUSBHostDevice > "$OUT/ioreg_iousb.plist"

echo "wrote:"
for f in diag_usb_only.json system_profiler.json ioreg_ports.plist ioreg_iousb.plist; do
  echo "  $OUT/$f ($(wc -c < "$OUT/$f") bytes)"
done
echo
echo "quick look — USB-only device names:"
python3 -c "
import json,sys
d=json.load(open('$OUT/diag_usb_only.json'))
def w(x,i=0):
  for it in x or []:
    print('  '*i+'- '+repr(it.get('_name'))+' speed='+repr(it.get('speed')))
    w(it.get('_items'),i+1)
w(d.get('SPUSBDataType'))
"
