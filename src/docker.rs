use std::process::{self, Command};

pub fn execute(path: &str, args: &str) -> String {
    let output = Command::new("docker")
        .arg("run")
        .args(["--entrypoint", "", "--pull", "always"])
        .arg(path)
        .args(args.split(' '))
        .output()
        .unwrap_or_else(|err| {
            eprintln!("Unable to run docker command: {err}");
            process::exit(1);
        });

    String::from_utf8(output.stdout).unwrap()
}
