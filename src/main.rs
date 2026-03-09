#![feature(once_cell_try)]

use std::collections::HashMap;
use crate::files::PrintName;
use crate::packages::{verify_package, Package};
use owo_colors::OwoColorize;

pub mod files;
mod users;
mod processes;
mod networking;
mod packages;
mod memorybinary;
mod busybox;
mod shellscript;
// TODO: https://www.cisecurity.org/cis-benchmarks/

// const CRACKLIB_PACKAGE: &str = include_str!("../cracklib-runtime.txt");
// fn main() {
//     let package_text = CRACKLIB_PACKAGE.trim();
//     let package_parts = package_text.splitn(3, ':').collect::<Vec<&str>>();
//     let (package_name, hashes, download_url) = (package_parts[0], package_parts[1], package_parts[2]);
//     let package_hashes = hashes.split('|').collect::<Vec<&str>>();
//     let package_file_list = package_hashes.iter().map(|hash| hash.split(';').map(|part| part.to_string()).collect::<Vec<String>>()).collect::<Vec<Vec<String>>>();
//     let package_file_map = package_file_list.iter().map(|file_list| (file_list[0].clone(), file_list[1].clone())).collect::<HashMap<_, _>>();
//     let package = Package {
//         name: package_name.to_string(),
//         files: package_file_map,
//         download_url: Some(download_url.to_string()),
//     };
//
//     let checked = verify_package(&package);
// }

const LINPEAS_SH: &'static str = include_str!("../tools/linpeas.sh");
const LYNIS_TAR: &'static [u8] = include_bytes!("../tools/lynis-3.1.6.tar.gz");
const LOKIRS_TAR: &'static [u8] = include_bytes!("../tools/Loki-RS/build/loki-rs.tar.gz");

macro_rules! print_error {
    ($attempt:expr, $message:expr) => {
        if let Err(e) = $attempt {
            eprintln!("{}: {:#}", $message, e);
        }
    }
}

fn main() -> anyhow::Result<()> {
    // TODO: output to log file
    // TODO: Disable networking

    print_error!(users::users(), "Failed to get users");

    print_error!(files::print_file("/etc/sudoers", PrintName::Full, 0, true), "Failed to print /etc/sudoers");
    print_error!(files::print_directory("/etc/sudoers.d", vec![]), "Failed to print /etc/sudoers.d");

    print_error!(files::print_directory("/var/spool/cron/crontabs", vec![]), "Failed to print /var/spool/cron/crontabs");
    print_error!(files::print_file("/etc/crontab", PrintName::Full, 0, true), "Failed to print /etc/crontab");
    print_error!(files::print_directory("/etc/cron.hourly", vec![]), "Failed to print /etc/cron.hourly");
    print_error!(files::print_directory("/etc/cron.daily", vec![]), "Failed to print /etc/cron.daily");
    print_error!(files::print_directory("/etc/cron.weekly", vec![]), "Failed to print /etc/cron.weekly");
    print_error!(files::print_directory("/etc/cron.monthly", vec![]), "Failed to print /etc/cron.monthly");
    print_error!(files::print_directory("/etc/cron.d", vec![]), "Failed to print /etc/cron.d");
    print_error!(files::print_file("/etc/anacrontab", PrintName::Full, 0, true), "Failed to print /etc/anacrontab");

    print_error!(files::print_directory("/etc/init.d", vec![]), "Failed to print /etc/init.d");

    println!("\nPackages:");
    match packages::get_packages() {
        Ok(packages) => {
            for package in packages { // TODO: write this to a file
                let hashes = package.files.iter().map(|(file, hash)| format!("{};{}", file, hash)).collect::<Vec<String>>().join("|");
                let (failed_files, missed_files) = verify_package(&package)?.unwrap_or((vec![], vec![]));
                let message = format!(" {} failed, {} missed", failed_files.len(), missed_files.len());
                if failed_files.is_empty() {
                    println!("{}", message.green());
                } else {
                    println!("{}", message.red());
                }
                for file in failed_files {
                    println!("  {}: FAILED", file);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get packages: {:#}", e);
        }
    }

    // TODO: verify packages
    // TODO: support rpms

    // TODO: important file permissions

    print_error!(processes::check(), "Failed to check processes");

    print_error!(networking::network_connections(), "Failed to get network connections");

    // TODO: check installed packages

    // TODO: configure firewall

    print_error!(shellscript::run_script("linpeas.sh", LINPEAS_SH, &[]), "Failed to run linpeas.sh");
    print_error!(shellscript::run_script_tar("lynis", "lynis/lynis", LYNIS_TAR, &["audit", "system"], true), "Failed to run lynis");

    // print_error!(shellscript::run_script_tar("loki", "loki", LOKIRS_TAR, &[], false), "Failed to run loki"); // TODO: not sure about this

    // TODO: STIG scripts https://www.cyber.mil/stigs/downloads/
    // TODO: Maldetect + ClamAV https://docs.clamav.net/ https://www.rfxn.com/projects/linux-malware-detect/
    // TODO: unix-privesc-check https://pentestmonkey.net/tools/audit/unix-privesc-check
    // TODO: rkhunter

    // TODO: enable networking

    Ok(())
}

