use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn run_script(name: &str, script: &str, args: &[&str]) -> anyhow::Result<()> {
    let file_path = format!("/tmp/{}", name);
    let mut file = File::create(&file_path)?;
    file.write_all(script.as_bytes())?;

    let output = Command::new("bash")
        .arg(&file_path)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    std::fs::remove_file(&file_path)?;

    Ok(())
}

pub fn run_script_tar(tar_name: &str, script_name: &str, tar_data: &[u8], args: &[&str], is_bash: bool) -> anyhow::Result<()> {
    let tar_directory_path = format!("/tmp/{}", tar_name);
    let tar_file_path = format!("{}/{}.tar", tar_directory_path, tar_name);
    std::fs::create_dir_all(format!("{}", &tar_directory_path))?;
    let mut tar_file = File::create(&tar_file_path)?;
    tar_file.write_all(tar_data)?;
    tar_file.flush()?;

    Command::new("tar")
        .args(["-C", &tar_directory_path, "-xzf", &tar_file_path])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    std::fs::remove_file(&tar_file_path)?;

    let mut script_dir = script_name.split('/').collect::<Vec<&str>>();
    script_dir.pop();
    let script_dir = script_dir.join("/");

    let script_path = format!("{}/{}", tar_directory_path, script_name);
    let mut command = if is_bash {
        Command::new("bash")
    } else {
        Command::new(&script_path)
    };
    let output = command
        .current_dir(format!("{}/{}", tar_directory_path, script_dir))
        .args(if is_bash { vec![&script_path] } else { vec![] })
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    std::fs::remove_dir_all(format!("/tmp/{}", tar_name))?;

    Ok(())
}