use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
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

fn get_users() -> Vec<User> {
    let passwd = File::open("/etc/passwd").unwrap();
    let user_lines = BufReader::new(passwd).lines().map(|line| line.unwrap());
    let mut users = Vec::new();
    for line in user_lines {
        let parts: Vec<&str> = line.split(':').collect();
        let user = User {
            name: parts[0].to_string(),
            uid: parts[2].parse().unwrap(),
            home_directory: parts[5].to_string(),
            shell: parts[6].to_string(),
            is_locked: false,
            groups: Vec::new(),
            deleted: false,
        };
        if user.name == "root" { continue };
        users.push(user);
    }

    let shadow = File::open("/etc/shadow").unwrap();
    let shadow_lines = BufReader::new(shadow).lines().map(|line| line.unwrap());
    for line in shadow_lines {
        let parts: Vec<&str> = line.split(':').collect();
        if parts[0] == "root" { continue };
        let user = users.iter_mut().find(|user| user.name == parts[0]).unwrap();
        user.is_locked = parts[1].starts_with('*') || parts[1].starts_with('!');
    }

    let groups = File::open("/etc/group").unwrap();
    let group_lines = BufReader::new(groups).lines().map(|line| line.unwrap());

    for line in group_lines {
        let parts: Vec<&str> = line.split(':').collect();
        for username in parts[3].split(',').filter(|s| !s.is_empty()) {
            users.iter_mut().find(|user| user.name == username.to_string()).unwrap().groups.push(parts[0].to_string());
        }
    }

    users
}

const NONINTERACTIVE_SHELLS: [&str; 4] = ["/sbin/nologin", "/usr/sbin/nologin", "/bin/false", "/usr/bin/false"];

const MANAGE_USER_OPTIONS: [&str; 3] = ["Make non-interactive", "Delete", "Ignore"];
const MANAGE_SUDO_OPTIONS: [&str; 2] = ["Leave in sudo", "Remove from sudo"];

fn handle_interactive_user(user: &mut User) {
    println!("{}", format!("User `{}` is possibly interactive ({})", user.name, user.shell).red());
    let answer = Select::new("Choose an option: ", MANAGE_USER_OPTIONS.to_vec()).prompt().unwrap();
    match answer {
        "Make non-interactive" => {
            user.shell = "/sbin/nologin".to_string();
            println!("Run usermod --shell /sbin/nologin {}", user.name); // TODO
        },
        "Delete" => {
            user.deleted = true;
            busybox("deluser", &["--remove-home", &*user.name]).expect("Failed to delete user");
        },
        "Ignore" => {
            busybox_with_stdin("chpasswd", &["-c", "sha512"], format!("{}:Password", user.name).as_bytes()).expect("Failed to set password for user");
        }
        _ => ()
    }
}

fn handle_noninteractive_user(user: &mut User) {
    user.is_locked = true;
    busybox("passwd", &["-l", &*user.name]).expect("Failed to lock user");
}

fn handle_admin_user(user: &mut User) {
    println!("{}", format!("User `{}` is in sudo group", user.name).red());
    let answer = Select::new("Choose an option: ", MANAGE_SUDO_OPTIONS.to_vec()).prompt().unwrap();
    match answer {
        "Leave in sudo" => (),
        "Remove from sudo" => {
            user.groups.retain(|group| group != &"sudo".to_string());
            busybox("delgroup", &[&*user.name, "sudo"]).expect("Failed to remove user from sudo group");
        },
        _ => ()
    }
}

pub fn users() {
    let mut users = get_users();

    for user in &mut users {
        println!("\n{} {}", "User:".yellow(), user.name);
        if !NONINTERACTIVE_SHELLS.contains(&user.shell.as_str()) {
            handle_interactive_user(user);
            if user.deleted { continue };
        }

        if NONINTERACTIVE_SHELLS.contains(&user.shell.as_str()) && !user.is_locked {
            handle_noninteractive_user(user);
        }

        if user.groups.contains(&"sudo".to_string()) {
            handle_admin_user(user);
        }

        let home_directory = Path::new(&user.home_directory);
        if home_directory.exists() {
            crate::files::print_directory(&home_directory.join(".ssh"), vec!["authorized_keys", "environment", "rc"]);
        }
    }
}