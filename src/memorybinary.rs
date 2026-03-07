use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::process::{Command, Stdio};

pub struct MemoryBinary {
    binary: &'static [u8],
    file: File,
    fd_path: String,
}

impl MemoryBinary {
    pub fn new(name: &str, binary: &'static [u8]) -> std::io::Result<Self> {
        let name = CString::new(name).unwrap();
        let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error())
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        file.write_all(binary)?;
        let fd_path = format!("/proc/self/fd/{}", fd);
        Ok(Self { binary, file, fd_path })
    }

    pub fn run(&self, app: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new(&self.fd_path)
            .arg(app)
            .args(args)
            .output()
    }

    pub fn run_with_stdin(&self, app: &str, args: &[&str], stdin: &[u8]) -> std::io::Result<std::process::Output> {
        let mut child = Command::new(&self.fd_path)
            .arg(app)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        child.stdin.take().unwrap().write_all(stdin)?;

        child.wait_with_output()
    }
}