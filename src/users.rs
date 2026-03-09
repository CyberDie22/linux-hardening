use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::Context;
use inquire::Select;
use owo_colors::OwoColorize;
use crate::busybox::{busybox, busybox_with_stdin};

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
            name: parts[0].to_string(),
            uid: parts[2].parse().context("Failed to parse UID")?,
            home_directory: parts[5].to_string(),
            shell: parts[6].to_string(),
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
        if parts[0] == "root" { continue };
        if let Some(user) = users.iter_mut().find(|user| user.name == parts[0]) {
            user.is_locked = parts[1].starts_with('*') || parts[1].starts_with('!');
        }
    }

    let groups = File::open("/etc/group").context("Failed to open /etc/group")?;
    for line in BufReader::new(groups).lines() {
        let line = line.context("Failed to read line from /etc/group")?;
        let parts: Vec<&str> = line.split(':').collect();
        for username in parts[3].split(',').filter(|s| !s.is_empty()) {
            if let Some(user) = users.iter_mut().find(|user| user.name == username) {
                user.groups.push(parts[0].to_string());
            }
        }
    }

    Ok(users)
}

const NONINTERACTIVE_SHELLS: [&str; 4] = ["/sbin/nologin", "/usr/sbin/nologin", "/bin/false", "/usr/bin/false"];

const MANAGE_USER_OPTIONS: [&str; 3] = ["Make non-interactive", "Delete", "Ignore"];
const MANAGE_SUDO_OPTIONS: [&str; 2] = ["Leave in sudo", "Remove from sudo"];

fn handle_interactive_user(user: &mut User) -> anyhow::Result<()> {
    println!("{}", format!("User `{}` is possibly interactive ({})", user.name, user.shell).red());
    let answer = Select::new("Choose an option: ", MANAGE_USER_OPTIONS.to_vec()).prompt().context("Failed to get user selection")?;
    match answer {
        "Make non-interactive" => {
            user.shell = "/sbin/nologin".to_string();
            println!("Run usermod --shell /sbin/nologin {}", user.name); // TODO
        },
        "Delete" => {
            user.deleted = true;
            busybox("deluser", &["--remove-home", &*user.name]).context("Failed to delete user")?;
        },
        "Ignore" => {
            busybox_with_stdin("chpasswd", &["-c", "sha512"], format!("{}:Password", user.name).as_bytes()).context("Failed to set password for user")?;
        }
        _ => ()
    }
    Ok(())
}

fn handle_noninteractive_user(user: &mut User) -> anyhow::Result<()> {
    user.is_locked = true;
    busybox("passwd", &["-l", &*user.name]).context("Failed to lock user")?;
    Ok(())
}

fn handle_admin_user(user: &mut User) -> anyhow::Result<()> {
    println!("{}", format!("User `{}` is in sudo group", user.name).red());
    let answer = Select::new("Choose an option: ", MANAGE_SUDO_OPTIONS.to_vec()).prompt().context("Failed to get user selection")?;
    match answer {
        "Leave in sudo" => (),
        "Remove from sudo" => {
            user.groups.retain(|group| group != &"sudo".to_string());
            busybox("delgroup", &[&*user.name, "sudo"]).context("Failed to remove user from sudo group")?;
        },
        _ => ()
    }
    Ok(())
}


fn make_noninteractive(user: &mut User) -> anyhow::Result<()> {
    print!("Making non-interactive");

}

pub fn users() -> anyhow::Result<()> {
    let mut users = get_users()?;

    let allowed_users = fs::read_to_string("users.txt")?.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();
    let allowed_admins = fs::read_to_string("admins.txt")?.split('\n').filter(|s| !s.is_empty()).collect::<Vec<_>>();

    for user in &mut users {
        print!("\n{} {}", "User:".yellow(), user.name);

        if (user.uid < 1000) {
            handle_
        }

        if !NONINTERACTIVE_SHELLS.contains(&user.shell.as_str()) {
            handle_interactive_user(user)?;
            if user.deleted { continue };
        }

        if NONINTERACTIVE_SHELLS.contains(&user.shell.as_str()) && !user.is_locked {
            handle_noninteractive_user(user)?;
        }

        if user.groups.contains(&"sudo".to_string()) {
            handle_admin_user(user)?;
        }

        let home_directory = Path::new(&user.home_directory);
        if home_directory.exists() {
            crate::files::print_directory(&home_directory.join(".ssh"), vec!["authorized_keys", "environment", "rc"])?;
        }
    }
    Ok(())
}