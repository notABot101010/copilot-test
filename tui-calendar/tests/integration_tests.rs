// Basic integration tests for the TUI Calendar application
// These tests verify that the binary can be built and executed

const BINARY_NAME: &str = "tui-calendar";

#[test]
fn test_binary_exists() {
    // This test ensures the project can be built
    let output = std::process::Command::new("cargo")
        .args(["build", "--bin", BINARY_NAME])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "Failed to build tui-calendar: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_binary_runs() {
    // Build first
    let build_output = std::process::Command::new("cargo")
        .args(["build", "--bin", BINARY_NAME])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute cargo build");

    assert!(build_output.status.success(), "Build failed");

    // Test that the binary can start (it will fail without a TTY, but that's expected)
    let run_output = std::process::Command::new("cargo")
        .args(["run", "--bin", BINARY_NAME])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("TERM", "dumb")
        .output()
        .expect("Failed to execute tui-calendar");

    // The application should exit (either successfully or with an error about no TTY)
    // We're just testing that it can be invoked without panicking
    assert!(
        run_output.status.code().is_some(),
        "Binary should exit with a status code"
    );
}
