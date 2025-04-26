//! For non-Windows hosts, zig-bootstrap expects:
//!
//! - Recent GCC or Clang C++ compiler
//! - Static C++ standard library on some systems
//! - Recent CMake
//! - Make or Ninja
//! - POSIX shell & CLI tools
//! - Recent Python 3
//!
//! For Windows hosts, zig-bootstrap expects:
//!
//! - C++ CMake tools for Windows
//! - Developer Command Prompt for VS 2019 shell environment
//!
//! _Unsure if the Windows host requirements also include CMake and Python 3._
//!
//! There's a `./build` or `./build.bat` script in the `zig-bootstrap` directory
//! that runs the whole build suite (LLVM, zlib, zstd, etc.) and then builds Zig
//! itself.
//!
//! ```sh
//! ./build <arch>-<os>-<abi> <mcpu>
//! ```
//!
//! Note that these are Zig's names for arch, os, abi, and mcpu. These don't
//! necessarily line up with the ones that rustc uses.
//!
//! Output is placed in `./out/zig-<triple>-<cpu>/` relative to the
//! `zig-bootstrap` directory. The `zig`/`zig.exe` binary is placed directly in
//! that directory and the `lib/` folder is right next to it.

use std::{
    env::{self, var},
    error::Error,
    fs,
    io::{self, Seek, SeekFrom},
    path::Path,
    process::{Command, Stdio},
};

use build::{cargo_pkg_version_major, cargo_pkg_version_minor, cargo_pkg_version_patch};
use reqwest::blocking::get;
use zip::{ZipArchive, read::root_dir_common_filter};

/// If `./zig-bootstrap/` is not present we need to clone it. If we're building
/// documentation for docs.rs or similar we don't want to do that. Instead of
/// `git clone` we can skip depending on Git and just download & extract a
/// `.zip` or `tar.gz` archive of the tag that we want.
fn main() -> Result<(), Box<dyn Error>> {
    build::rerun_if_env_changed("DO_IT");

    // Dev shortcircuit
    if !env::var("DO_IT").is_ok() {
        return Ok(());
    }

    if !docs_rs() && !fs::exists("zig-bootstrap")? {
        let major = build::cargo_pkg_version_major();
        let minor = build::cargo_pkg_version_minor();
        let patch = build::cargo_pkg_version_patch();

        {
            let response = reqwest::blocking::get(format!(
                "https://github.com/ziglang/zig-bootstrap/archive/refs/tags/{major}.{minor}.{patch}.zip"
            ))?;
            let mut response = response.error_for_status()?;
            let mut file = fs_err::File::create("zig-bootstrap.zip")?;
            response.copy_to(&mut file)?;
        }

        {
            let file = fs_err::File::open("zig-bootstrap.zip")?;
            let mut zip_archive = ZipArchive::new(file)?;
            zip_archive.extract_unwrapped_root_dir("zig-bootstrap", root_dir_common_filter)?;
        }

        fs_err::remove_file("zig-bootstrap.zip")?;
    }

    if docs_rs() {
        fs_err::write(
            build::out_dir().join(if build::cargo_cfg_windows() {
                "zig.exe"
            } else {
                "zig"
            }),
            [],
        )?;
        fs_err::create_dir_all(build::out_dir().join("lib"))?;
    } else {
        let (zig_target, zig_mcpu) = zig_target_mcpu_for_build_target()
            .ok_or_else(|| format!("unmapped target: {}", build::target()))?;
        let mut cmd = Command::new(if cfg!(windows) {
            "./build.bat"
        } else {
            "./build"
        });
        cmd.current_dir("zig-bootstrap")
            .arg(&zig_target)
            .arg(&zig_mcpu);
        cmd.stdin(Stdio::null())
            .stdout(io::stderr())
            .stderr(io::stderr());
        let status = cmd.status()?;
        if !status.success() {
            return Err(format!("zig-bootstrap {:?} failed: {}", &cmd, status).into());
        }
        let zig_out_dir = Path::new("zig-bootstrap")
            .join("out")
            .join(format!("zig-{}-{}", &zig_target, &zig_mcpu));
        fs_err::rename(
            zig_out_dir.join(if build::cargo_cfg_windows() {
                "zig.exe"
            } else {
                "zig"
            }),
            build::out_dir().join(if build::cargo_cfg_windows() {
                "zig.exe"
            } else {
                "zig"
            }),
        )?;
        fs_err::rename(zig_out_dir.join("lib"), build::out_dir().join("lib"))?;
    }

    Ok(())
}

fn docs_rs() -> bool {
    env::var("DOCS_RS").is_ok()
}

/// Returns a `(zig_target, zig_mcpu)` tuple for the Rust target triple & CPU
/// features specified by the environment variables provided to `build.rs`.
fn zig_target_mcpu_for_build_target() -> Option<(String, String)> {
    // Just basic target mapping for now.
    Some(match build::target().as_str() {
        "aarch64-apple-darwin" => ("aarch64-macos-none".into(), "baseline".into()),
        "x86_64-unknown-linux-gnu" => ("x86_64-linux-gnu".into(), "baseline".into()),
        "x86_64-pc-windows-gnu" => ("x86_64-windows-gnu".into(), "baseline".into()),
        _ => return None,
    })
}
