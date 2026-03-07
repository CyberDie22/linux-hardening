#![feature(once_cell_try)]

use std::collections::HashMap;
use crate::files::PrintName;
use crate::packages::{verify_package, Package};

pub mod files;
mod users;
mod processes;
mod networking;
mod packages;
mod memorybinary;
mod busybox;
mod shellscript;
// TODO: https://www.cisecurity.org/cis-benchmarks/

const CRACKLIB_PACKAGE: &str = include_str!("../cracklib-runtime.txt");
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

fn main() {
    // TODO: output to log file
    // TODO: Disable networking
    // TODO: Backup entire system

    // TODO: automate based on good user list
    // users::users();

    // files::print_file("/etc/sudoers", PrintName::Full, 0, true);
    // files::print_directory("/etc/sudoers.d", vec![]);
    //
    // files::print_directory("/var/spool/cron/crontabs", vec![]);
    // files::print_file("/etc/crontab", PrintName::Full, 0, true);
    // files::print_directory("/etc/cron.hourly", vec![]);
    // files::print_directory("/etc/cron.daily", vec![]);
    // files::print_directory("/etc/cron.weekly", vec![]);
    // files::print_directory("/etc/cron.monthly", vec![]);
    // files::print_directory("/etc/cron.d", vec![]);
    // files::print_file("/etc/anacrontab", PrintName::Full, 0, true);
    //
    // files::print_directory("/etc/init.d", vec![]);

    // println!("\nPackages:");
    // let packages = packages::get_packages();
    // for package in packages { // TODO: write this to a file
    //     let hashes = package.files.iter().map(|(file, hash)| format!("{};{}", file, hash)).collect::<Vec<String>>().join("|");
    //     let (failed_files, missed_files) = verify_package(&package).unwrap_or((vec![], vec![]));
    //     println!("{}: {} failed, {} missed", package.name, failed_files.len(), missed_files.len());
    //     for file in failed_files {
    //         println!("  {}: FAILED", file);
    //     }
    // }

    // TODO: verify packages
    // TODO: support rpms

    // TODO: important file permissions

    // processes::check();

    // networking::network_connections();

    // TODO: check installed packages

    // TODO: configure firewall

    // shellscript::run_script("linpeas.sh", LINPEAS_SH, &[]).unwrap();
    // shellscript::run_script_tar("lynis", "lynis/lynis", LYNIS_TAR, &["audit", "system"], true).unwrap();

    // shellscript::run_script_tar("loki", "loki", LOKIRS_TAR, &[], false).unwrap();  // TODO: not sure about this

    // TODO: STIG scripts https://www.cyber.mil/stigs/downloads/
    // TODO: Maldetect + ClamAV https://docs.clamav.net/ https://www.rfxn.com/projects/linux-malware-detect/
    // TODO: unix-privesc-check https://pentestmonkey.net/tools/audit/unix-privesc-check
    // TODO: rkhunter

    // TODO: enable networking
}

