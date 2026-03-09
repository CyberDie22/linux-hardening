use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::{anyhow, Context};
use inquire::Select;
use owo_colors::OwoColorize;
use crate::busybox::{busybox, busybox_with_stdin, usermod};

#[derive(Debug)]
struct User {
    name: String,
    uid: u32,
    home_directory: String,
    shell: String,
    is_locked: bool,
    groups: Vec<String>,
    deleted: bool,
}

fn get_users() -> anyhow::Result<Vec<User>> {
    let passwd = File::open("/etc/passwd").context("Failed to open /etc/passwd")?;
    let mut users = Vec::new();
    for line in BufReader::new(passwd).lines() {
        let line = line.context("Failed to read line from /etc/passwd")?;
        let parts: Vec<&str> = line.split(':').collect();
        let user = User {
            name: parts.get(0).ok_or(anyhow!("Name field missing from line: {}", line))?.to_string(),
            uid: parts.get(2).ok_or(anyhow!("UID field missing from line: {}", line))?.parse().context("Failed to parse UID")?,
            home_directory: parts.get(5).ok_or(anyhow!("Home directory field missing from line: {}", line))?.to_string(),
            shell: parts.get(6).ok_or(anyhow!("Shell field missing from line: {}", line))?.to_string(),
            is_locked: false,
            groups: Vec::new(),
            deleted: false,
        };
        if user.name == "root" { continue };
        users.push(user);
    }

    let shadow = File::open("/etc/shadow").context("Failed to open /etc/shadow")?;
    for line in BufReader::new(shadow).lines() {
        let line = line.context("Failed to read line from /etc/shadow")?;
        let parts: Vec<&str> = line.split(':').collect();
        if *parts.get(0).ok_or(anyhow!("Name field missing from line: {}", line))? == "root" { continue };
        if let Some(user) = users.iter_mut().find(|user| user.name == parts[0]) {
            let password = parts.get(1).ok_or(anyhow!("Password field missing from line: {}", line))?;
            user.is_locked = password.starts_with('*') || password.starts_with('!');
        }
    }

    let groups = File::open("/etc/group").context("Failed to open /etc/group")?;
    for line in BufReader::new(groups).lines() {
        let line = line.context("Failed to read line from /etc/group")?;
        let parts: Vec<&str> = line.split(':').collect();
        for username in parts.get(3).ok_or(anyhow!("Group members field missing from line: {}", line))?.split(',').filter(|s| !s.is_empty()) {
            if let Some(user) = users.iter_mut().find(|user| user.name == username) {
                user.groups.push(parts[0].to_string());
            }
        }
    }

    Ok(users)
}

// const NONINTERACTIVE_SHELLS: [&str; 4] = ["/sbin/nologin", "/usr/sbin/nologin", "/bin/false", "/usr/bin/false"];

fn make_noninteractive(user: &mut User) -> anyhow::Result<()> {
    print!("Making non-interactive");
    user.is_locked = true;
    busybox("passwd", &["-l", &*user.name]).context("Failed to lock user")?;
    usermod(&["-s", "/sbin/nologin", &*user.name]).context("Failed to set shell to /sbin/nologin")?;
    Ok(())
}

pub fn users() -> anyhow::Result<()> {
    let mut users = get_users()?;

    let users_txt = fs::read_to_string("users.txt")?;
    let allowed_users = users_txt.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let admins_txt = fs::read_to_string("admins.txt")?;
    let allowed_admins = admins_txt.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();

    for user in &mut users {
        print!("{} {}", "User:".yellow(), user.name);

        if user.uid < 1000 {
            make_noninteractive(user)?;
        }

        if (user.groups.contains(&"sudo".to_string()) || user.groups.contains(&"wheel".to_string())) && !allowed_admins.contains(&user.name.as_str()) {
            user.groups.retain(|group| group != &"sudo".to_string());
            busybox("delgroup", &[&*user.name, "sudo"]).context("Failed to remove user from sudo group")?;
            busybox("delgroup", &[&*user.name, "wheel"]).context("Failed to remove user from wheel group")?;
        }

        if !allowed_users.contains(&user.name.as_str()) {
            print!(" - {}", "Not allowed".red());
            continue;
        }

        let home_directory = Path::new(&user.home_directory);
        if home_directory.exists() {
            crate::files::print_directory(&home_directory.join(".ssh"), vec!["authorized_keys", "environment", "rc"])?;
        }
        println!();
    }
    Ok(())
}