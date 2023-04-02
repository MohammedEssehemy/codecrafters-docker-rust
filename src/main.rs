use anyhow::{Context, Result};
use std::process::{self, Command};
use std::str;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let std_out = str::from_utf8(&output.stdout)?;
    if output.status.success() {
        println!("{}", std_out);
    } else {
        eprintln!("{std_out}");
        process::exit(1);
    }

    Ok(())
}
