use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    // let status = cc::Build::new()
    //     .get_compiler()
    //     .to_command()
    //     .args([
    //         "-o",
    //         "zig-bootstrap/zig/bootstrap",
    //         "zig-bootstrap/zig/bootstrap.c",
    //     ])
    //     .status()?;
    // if !status.success() {
    //     return Err(format!("Failed to compile bootstrap.c: {}", status).into());
    // }

    // let status = Command::new("./bootstrap")
    //     .current_dir("zig-bootstrap/zig")
    //     .status()?;
    // if !status.success() {
    //     return Err(format!("Failed to run ./bootstrap: {}", status).into());
    // }

    Ok(())
}
