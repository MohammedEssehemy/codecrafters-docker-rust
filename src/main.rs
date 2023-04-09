use anyhow::{Context, Result};
use std::env::{args, set_current_dir};
use std::fs::{
    self, copy, create_dir, create_dir_all, read_dir, set_permissions, File, Permissions,
};
use std::os::unix::fs::{chroot, PermissionsExt};
use std::path::Path;
use std::process::{self, Command, Stdio};
use tempfile::tempdir;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let exit_code = run_child_process(command, command_args)
        .with_context(|| "run_child_process: failed to run child process")?;
    process::exit(exit_code);
}

fn _log_dir_contents(path: &Path) -> Result<()> {
    println!("{path:?} => {:?}", read_dir(path)?.collect::<Vec<_>>());
    Ok(())
}

fn run_child_process(command: &String, command_args: &[String]) -> Result<i32> {
    let sandbox = tempdir()?;

    copy_command(command, sandbox.path()).with_context(|| "copy_command failed")?;

    create_dev_null(sandbox.path()).with_context(|| "create_dev_null failed")?;

    chroot_process(sandbox.path()).with_context(|| "chroot_process failed")?;

    let mut child = Command::new(command)
        .args(command_args)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let exit_status = child.wait()?;
    Ok(exit_status.code().unwrap_or(1))
}

fn copy_command(command: &String, path: &Path) -> Result<()> {
    let command_relative = command.trim_start_matches("/");
    let target_path = path.join(command_relative);
    create_dir_all(target_path.parent().unwrap())?;
    copy(command, target_path)?;
    Ok(())
}

fn _copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(&dst)?;
    println!("src: {src:?} dst: {dst:?}");
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            _copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else if ty.is_file() {
            fs::copy(entry.path(), &dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn create_dev_null(path: &Path) -> Result<()> {
    let dev_path = path.join("dev");
    let dev_null_path = path.join("dev/null");

    create_dir(&dev_path)?;
    File::create(&dev_null_path)?;
    set_permissions(&dev_path, Permissions::from_mode(0o555))?;
    set_permissions(&dev_null_path, Permissions::from_mode(0o555))?;

    Ok(())
}

fn chroot_process(path: &Path) -> Result<()> {
    chroot(path)?;
    set_current_dir("/")?;
    Ok(())
}
