# MSI Windows Installer for voxctrl (via wixl)

**Date**: 2026-02-22
**Branch**: master

## Summary

Replaced the NSIS `.exe` installer with an MSI built by `wixl` (from the `msitools`
Debian package). This is a Linux-native WiX-compatible compiler — no Wine or Windows
toolchain needed. The MSI format is preferred for enterprise/GPO deployment and
silent installs (`msiexec /i ... /quiet`).

Previously used WiX (replaced due to licensing), then NSIS (replaced here for MSI
format benefits).

## Design Decisions

- **Per-user install** to `%LOCALAPPDATA%\Voxctrl\` — `InstallScope="perUser"`, no UAC
- **HKCU Run key** for auto-start (registry value in the main component)
- **No config.json bundled** — app creates defaults when no config found
- **No .ico file** — app generates tray icons programmatically
- **Start Menu shortcut** with a dummy registry key as KeyPath (WiX requirement)
- **MajorUpgrade** element handles upgrades by UpgradeCode GUID
- **Forward-slash source paths** in `<File Source="...">` for Linux cross-build

## Files

- `wix/main.wxs` — WiX manifest (replaces `installer.nsi`)
- `Dockerfile` — uses `msitools` package, runs `wixl` to produce MSI

## Build

```bash
# In Docker (or locally with msitools installed):
wixl -o target/voxctrl-0.2.0-x86_64.msi wix/main.wxs

# Validate XML without the actual .exe present:
wixl -o /dev/null wix/main.wxs  # errors on missing source file, but validates XML
```
