//! Sprint 1: Project Structure Tests
//!
//! These tests verify the workspace structure exists before
//! any implementation code is written.

use std::path::Path;

/// Get the workspace root from the core crate's perspective
fn workspace_root() -> &'static Path {
    // From crates/pii_redacta_core/tests/, go up 2 levels to workspace root
    Path::new("../../")
}

#[test]
fn test_workspace_cargo_toml_exists() {
    assert!(
        workspace_root().join("Cargo.toml").exists(),
        "Workspace Cargo.toml must exist"
    );
}

#[test]
fn test_core_crate_exists() {
    assert!(
        workspace_root().join("crates/pii_redacta_core").exists(),
        "Core crate directory must exist"
    );
    assert!(
        workspace_root()
            .join("crates/pii_redacta_core/Cargo.toml")
            .exists(),
        "Core crate Cargo.toml must exist"
    );
}

#[test]
fn test_api_crate_exists() {
    assert!(
        workspace_root().join("crates/pii_redacta_api").exists(),
        "API crate directory must exist"
    );
    assert!(
        workspace_root()
            .join("crates/pii_redacta_api/Cargo.toml")
            .exists(),
        "API crate Cargo.toml must exist"
    );
}

#[test]
fn test_core_src_directory_exists() {
    assert!(
        workspace_root()
            .join("crates/pii_redacta_core/src")
            .exists(),
        "Core src directory must exist"
    );
}

#[test]
fn test_ci_workflow_exists() {
    assert!(
        workspace_root().join(".github/workflows/ci.yml").exists(),
        "CI workflow must exist"
    );
}

#[test]
fn test_gitignore_exists() {
    assert!(
        workspace_root().join(".gitignore").exists(),
        ".gitignore must exist"
    );
}
