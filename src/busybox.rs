use std::sync::OnceLock;
use crate::memorybinary::MemoryBinary;

static BUSYBOX_BINARY_DATA: &'static [u8] = include_bytes!("../tools/busybox");
static BUSYBOX_BIN: OnceLock<MemoryBinary> = OnceLock::new();

pub fn busybox(app: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    BUSYBOX_BIN.get_or_try_init(|| MemoryBinary::new("busybox", BUSYBOX_BINARY_DATA))?
        .run(app, args)
}

pub fn busybox_with_stdin(app: &str, args: &[&str], stdin: &[u8]) -> std::io::Result<std::process::Output> {
    BUSYBOX_BIN.get_or_try_init(|| MemoryBinary::new("busybox", BUSYBOX_BINARY_DATA))?
        .run_with_stdin(app, args, stdin)
}