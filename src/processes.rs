use anyhow::Context;
use owo_colors::OwoColorize;
use procfs::process::{MMapPath, Process};

fn check_scripts() -> anyhow::Result<()> {
    println!();
    let mut potential_scripts = Vec::new();

    for process in procfs::process::all_processes().context("Failed to enumerate processes")? {
        let process = process?;
        let process_exe = process.exe();
        if let Ok(process_exe) = process_exe {
            if let Some(name) = process_exe.file_name().and_then(|n| n.to_str()) {
                if name.contains("python") {
                    potential_scripts.push(process);
                    continue;
                } else if name.contains("bash") {
                    potential_scripts.push(process);
                    continue;
                }
            }
        }

        if let Ok(process_maps) = process.maps() {
            if process_maps.iter().any(|map| {
                if let MMapPath::Path(path) = &map.pathname {
                    path.to_str().unwrap_or("").contains("python")
                } else {
                    false
                }
            }) {
                potential_scripts.push(process);
                continue;
            }
        }
    }

    for process in potential_scripts {
        println!("{}", "Found potential script process:".red());
        println!("  PID: {}", process.pid);
        if let Ok(cmdline) = process.cmdline() {
            println!("  Command: {}", cmdline.join(" "));
        }

        if let Ok(stat) = process.stat() {
            let mut possible_parent = Process::new(stat.ppid);
            loop {
                if let Ok(parent) = possible_parent {
                    if let Ok(parent_stat) = parent.stat() {
                        possible_parent = Process::new(parent_stat.ppid);
                        let cmd = parent.cmdline().unwrap_or_default().join(" ");
                        println!("  Parent: {} - {}", parent.pid, cmd);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }
    println!();
    Ok(())
}

pub fn check() -> anyhow::Result<()> {
    check_scripts()
}