use std::error::Error;
use std::process::{self, Command};

pub fn run(name: &str, path: &str) -> Result<(), Box<dyn Error>> {
    Command::new("docker")
        .args([
            "run",
            "--entrypoint",
            "",
            "--pull",
            "always",
            "--name",
            name,
            "-dit",
            path,
            "bash",
        ])
        .output()
        .unwrap_or_else(|err| {
            eprintln!("Unable to stand docker container up: {err}");
            process::exit(1);
        });

    Ok(())
}

pub fn stop(name: &str) -> Result<(), Box<dyn Error>> {
    Command::new("docker")
        .args(["rm", "-f", name])
        .output()
        .unwrap();

    Ok(())
}

pub fn execute(name: &str, args: &str) -> String {
    let output = Command::new("docker")
        .args(["exec", name])
        .args(args.split(' '))
        .output()
        .unwrap_or_else(|err| {
            eprintln!("Unable to run docker command: {err}");
            process::exit(1);
        });

    String::from_utf8(output.stdout).unwrap()
}
