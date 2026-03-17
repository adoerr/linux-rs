# AI Agent Guide for linux-rs

This guide outlines the architecture, patterns, and workflows for working productively in the `linux-rs` codebase, a collection of low-level Linux systems programming crates.

## Project Architecture

The workspace consists of several crates, each focusing on a specific Linux kernel subsystem or functionality:

*   **`syscall-rs`**: The core library. Provides safe wrappers around `libc` system calls, custom `Result` types, and essential types (`FileDesc`, `Stdio`, `Signal`). Use this as a reference or dependency for wrapping other syscalls.
*   **`poll`**: Implements an `epoll`-based event loop (reactor pattern). Defines `Registry` and `Poll` types for async I/O.
*   **`netlink`**: Implements the Netlink protocol for kernel communication (e.g., WiFi management). Handles raw socket creation and message framing.
*   **`debugger`**: Wraps `ptrace` for process inspection and control.
*   **`vpn`**: A functional VPN implementation using `tun-tap`, `socket2`, and `etherparse`.
*   **`bluetooth`**, **`wayland`**: Domain-specific implementations (e.g., raw HCI sockets).

**Key Philosophy**: Minimal abstraction over Linux kernel interfaces. Direct usage of `libc` types and constants is preferred over high-level Rust standard library abstractions when precise control is needed.

## Developer Workflows

The project uses `just` (command runner) for all standard tasks. **Do not use `cargo` directly unless necessary.**

*   **Build & Check**: `just check` (formats, lints with clippy, and checks).
*   **Clean**: `just clean`.
*   **Update Methods**: `just update` (deps), `just outdated`.
*   **Run Examples**: `just example <name>` (e.g., `just example btlist`).
*   **Debug/Release Builds**: `just debug`, `just release`.

**Note**: The project relies on **nightly** features (e.g., for formatting options). ensure your environment supports this.

## Coding Patterns & Conventions

### 1. System Call Wrapping
Use the `syscall!` macro pattern (found in `syscall-rs/src/lib.rs` and `poll/src/poll.rs`) to wrap unsafe `libc` calls. This macro handles `errno` checking automatically.

```rust
// Example pattern
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) }; // or poll_sys::$fn
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}
```

### 2. Binary Compatibility
When defining structs for kernel communication (e.g., Netlink headers, HCI packets), use `#[repr(C, packed)]` or explicit alignment to match C ABI.

```rust
#[repr(C, packed)]
struct MsgHeader {
    opcode: u16,
    index: u16,
    len: u16,
}
```

### 3. Error Handling
*   **Libraries**: Use `thiserror` to define custom `Error` enums.
*   **Wrappers**: Convert `errno` to `std::io::Error`.
*   **Generic**: `anyhow` is used in application code/examples.

### 4. FFI & Unsafe
*   Explicitly type `libc` constants and types (e.g., `libc::c_int`, `libc::sockaddr`).
*   Isolate `unsafe` blocks as much as possible, preferably within wrapper functions or the `syscall!` macro.

## Key Integration Points

*   **File Descriptors**: Everything represents a file descriptor. Use `std::os::fd::AsRawFd` and `test::RawFd` for passing resources between components.
*   **Networking**:
    *   `socket2` is used for creating sockets with specific domains/protocols not exposed by `std`.
    *   `etherparse` is used for packet inspection.
*   **Ptrace**: See `debugger/src/ptrace.rs` for interaction with child processes.

## Examples to Follow
*   **Netlink Communication**: `netlink/src/cmd.rs` (Socket setup, binding, message sending).
*   **Event Loop**: `poll/src/poll.rs` (`epoll_ctl`, `epoll_wait` wrapping).
*   **VPN Tunneling**: `vpn/src/lib.rs` (TUN/TAP interface management).

