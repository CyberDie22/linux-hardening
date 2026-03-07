use owo_colors::OwoColorize;
use procfs::process::{MMapPath, Process};

fn check_scripts() {
    println!();
    let mut potential_scripts = Vec::new();

    for process in procfs::process::all_processes().unwrap() {
        let process = process.unwrap();
        let process_exe = process.exe();
        if let Ok(process_exe) = process_exe {
            let name = process_exe.file_name().unwrap().to_str().unwrap();
            if name.contains("python") {
                potential_scripts.push(process);
                continue;
            } else if name.contains("bash") {
                potential_scripts.push(process);
                continue;
            }
        }

        let process_maps = process.maps().unwrap();
        if process_maps.iter().any(|map| {
            if let MMapPath::Path(path) = map.pathname.clone() {
                path.to_str().unwrap().contains("python")
            } else {
                false
            }
        }) {
            potential_scripts.push(process);
            continue;
        }
    }

    for process in potential_scripts {
        println!("{}", "Found potential script process:".red());
        println!("  PID: {}", process.pid);
        println!("  Command: {}", process.cmdline().unwrap().join(" "));

        let parent_id = process.stat().unwrap().ppid;
        let mut possible_parent = Process::new(parent_id);
        loop {
            if let Ok(parent) = possible_parent {
                let parent_id = parent.stat().unwrap().ppid;
                possible_parent = Process::new(parent_id);
                println!("  Parent: {} - {}", parent.pid, parent.cmdline().unwrap().join(" "));
            } else {
                break
            }
        }
    }
    println!();
}

pub fn check() {
    check_scripts();
}