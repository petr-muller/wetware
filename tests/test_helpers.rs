/// Test helpers for contract and integration tests
use std::process::Command;
use tempfile::TempDir;

/// Result of running a CLI command
#[derive(Debug)]
pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run a wet CLI command with arguments
pub fn run_wet_command(args: &[&str], db_dir: Option<&TempDir>) -> CommandResult {
    // Build the binary first (in a separate command to avoid stderr pollution)
    let build_result = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .output()
        .expect("Failed to build wetware");

    if !build_result.status.success() {
        panic!(
            "Failed to build wetware: {}",
            String::from_utf8_lossy(&build_result.stderr)
        );
    }

    // Run the built binary directly
    let mut cmd = Command::new("target/debug/wetware");

    // Add database path if temp dir provided
    if let Some(dir) = db_dir {
        let db_path = dir.path().join("test.db");
        cmd.env("WETWARE_DB", db_path.to_str().unwrap());
    }

    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to execute command");

    CommandResult {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Create a temporary database for testing
pub fn setup_temp_db() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}
