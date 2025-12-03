// src/clipboard/platform.rs
//! Platform-specific clipboard operations.

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform_impl;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform_impl;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform_impl;

pub use platform_impl::{copy_file_handle, perform_copy, perform_read};
