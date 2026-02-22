# Replace TCP IPC with Windows Named Pipe

**Date:** 2026-02-23
**Status:** Implementing

## Summary

Replace the TCP-based IPC between the Settings subprocess and the main tray app
(`stt_server.rs` / `stt_client.rs`) with a Windows named pipe (`\\.\pipe\voxctrl-stt`)
using the `interprocess` crate.

## Motivation

- TCP on port 5201 causes confusion with port 5200 (external llama-server)
- Firewall issues: Windows Firewall may block TCP connections
- Port conflicts: other apps may bind to 5201
- Named pipes: no ports, no firewall, automatic cleanup on process exit

## Changes

| File | Change |
|------|--------|
| `Cargo.toml` | Add `interprocess = "2"` |
| `src/stt_server.rs` | Named pipe listener via `interprocess::local_socket` |
| `src/stt_client.rs` | Named pipe client, no port parameter |
| `src/config.rs` | Remove `stt_server_port` field + default fn |
| `src/main.rs` | Drop port arg from `stt_server::start()` |
| `src/ui/model_table.rs` | Drop port arg from `transcribe_via_server()` |

## Wire Protocol (unchanged)

```
Request:  [4 bytes: WAV len u32 BE] [N bytes: WAV data]
Response: [1 byte: status 0=ok 1=err] [4 bytes: text len u32 BE] [N bytes: UTF-8 text]
```

## Named Pipe Details

- Pipe name: `voxctrl-stt` (interprocess prefixes with `\\.\pipe\` on Windows)
- Server: `ListenerOptions::new().name("voxctrl-stt").create_sync()`
- Client: `ConnectOptions::new().name("voxctrl-stt").connect_sync()`
- No timeouts (not supported on local sockets; not needed for local IPC)
- Generic `impl Read + Write` instead of `TcpStream` in handler functions
