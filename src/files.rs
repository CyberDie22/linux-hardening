use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::Context;
use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintName {
    Full,
    End,
    No
}

pub fn print_file<P: AsRef<Path>>(path: P, print_name: PrintName, indent: usize, print_gap: bool) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        if print_gap { println!(); }
        let file = File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
        let text = lines.join("\n");
        let is_empty = text.trim().is_empty();

        if print_name != PrintName::No {
            println!(
                "{}{} {} {}",
                "    ".repeat(indent),
                "File:".yellow(),
                if print_name == PrintName::Full { path.display().to_string() } else { path.file_name().context("missing file name")?.to_str().context("non-UTF-8 file name")?.to_string() },
                if is_empty { "(empty)".yellow() } else { "".yellow() }
            );
        }

        if !is_empty {
            println!("{}", text);
        }
        if print_gap { println!(); }
    }
    Ok(())
}

pub fn print_directory<P: AsRef<Path> + ?Sized>(path: &P, print_files: Vec<&str>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        println!("\n{} {}", "Directory:".yellow(), path.display());
        let entries = path.read_dir()?.collect::<Result<Vec<_>, _>>()?;
        if entries.is_empty() {
            println!("    {}", "Directory is empty".yellow());
            return Ok(());
        }

        for entry in entries {
            let file_name = entry.file_name();
            if print_files.is_empty() || print_files.contains(&file_name.to_str().context("non-UTF-8 file name")?) {
                print_file(&entry.path(), PrintName::End, 1, false)?;
            }
        }
        println!();
    }
    Ok(())
}