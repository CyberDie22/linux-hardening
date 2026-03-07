use std::collections::HashMap;
use procfs::process::FDTarget;

pub fn network_connections() {
    println!();
    let tcp = procfs::net::tcp().unwrap();
    let tcp6 = procfs::net::tcp6().unwrap();
    let udp = procfs::net::udp().unwrap();
    let udp6 = procfs::net::udp6().unwrap();

    let mut connections: Vec<(String, String, String, String, u64)> = Vec::new();

    for entry in tcp.into_iter().chain(tcp6) {
        let local_address = format!("{}", entry.local_address);
        let remote_address = format!("{}", entry.remote_address);
        let state = format!("{:?}", entry.state);
        let protocol = if entry.local_address.is_ipv6() { "v6" } else { "v4" }.to_string();
        let inode = entry.inode;
        connections.push((local_address, remote_address, state, format!("{}/tcp", protocol), inode))
    }
    for entry in udp.into_iter().chain(udp6) {
        let local_address = format!("{}", entry.local_address);
        let remote_address = format!("{}", entry.remote_address);
        let state = format!("{:?}", entry.state);
        let protocol = if entry.local_address.is_ipv6() { "v6" } else { "v4" }.to_string();
        let inode = entry.inode;
        connections.push((local_address, remote_address, state, format!("{}/udp", protocol), inode))
    }

    let processes = procfs::process::all_processes().unwrap();
    let mut process_map = HashMap::new();
    for process in processes {
        let process = process.unwrap();
        let fds = process.fd().unwrap();
        let stat = process.stat().unwrap();
        for fd in fds {
            if let Ok(fd) = fd {
                if let FDTarget::Socket(inode) = fd.target {
                    process_map.insert(inode, stat.clone());
                }
            }
        }
    }

    println!("{:<26} {:<26} {:<15} {:<9} {:<7} Command/PID", "Local Address", "Remote Address", "State", "Protocol", "Inode");
    for (local_address, remote_address, state, protocol, inode) in connections {
        let process = process_map.get(&inode);

        if let Some(process) = process {
            println!("{:<26} {:<26} {:<15} {:<9} {:<7} {}/{}", local_address, remote_address, state, protocol, inode, process.comm, process.pid);
        } else {
            println!("{:<26} {:<26} {:<15} {:<9} {:<7}", local_address, remote_address, state, protocol, inode);
        }
    }
    println!();
}