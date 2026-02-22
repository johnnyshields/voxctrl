#!/usr/bin/env python3
"""List all audio INPUT devices so you can find the right device_pattern for config.json."""

import sounddevice as sd

devices = sd.query_devices()
print(f"\n{'Idx':>4}  {'Ch':>3}  {'Sample Rate':>12}  Name")
print("-" * 72)
for i, dev in enumerate(devices):
    if dev["max_input_channels"] > 0:
        marker = "  <-- DJI mic" if "dji" in dev["name"].lower() else ""
        print(f"{i:>4}  {dev['max_input_channels']:>3}  "
              f"{int(dev['default_samplerate']):>12}  {dev['name']}{marker}")

print()
print('Set "device_pattern" in config.json to a substring of your mic name.')
print('Example: "DJI", "Wireless", or the full device name.')
