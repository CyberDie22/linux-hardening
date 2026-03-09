use std::sync::OnceLock;
use anyhow::Context;
use crate::memorybinary::MemoryBinary;

static BUSYBOX_BINARY_DATA: &'static [u8] = include_bytes!("../tools/busybox");
static BUSYBOX_BIN: OnceLock<MemoryBinary> = OnceLock::new();

pub fn busybox(app: &str, args: &[&str]) -> anyhow::Result<std::process::Output> {
    BUSYBOX_BIN.get_or_try_init(|| MemoryBinary::new("busybox", BUSYBOX_BINARY_DATA)).context("Failed to initialize busybox")?
        .run(app, args)
}

pub fn busybox_with_stdin(app: &str, args: &[&str], stdin: &[u8]) -> anyhow::Result<std::process::Output> {
    BUSYBOX_BIN.get_or_try_init(|| MemoryBinary::new("busybox", BUSYBOX_BINARY_DATA))?
        .run_with_stdin(app, args, stdin)
}

static USERMOD_BINARY_DATA: &'static [u8] = include_bytes!("../tools/shadow/src/usermod");
static USERMOD_BIN: OnceLock<MemoryBinary> = OnceLock::new();

pub fn usermod(args: &[&str]) -> anyhow::Result<std::process::Output> {
    USERMOD_BIN.get_or_try_init(|| MemoryBinary::new("usermod", USERMOD_BINARY_DATA)).context("Failed to initialize usermod")?
        .run("usermod", args)
}
