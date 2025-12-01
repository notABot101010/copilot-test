//! Landlock-based sandboxing for git operations.
//!
//! This module provides filesystem access restrictions using the Linux Landlock LSM
//! to ensure git operations can only access the intended repository directory.

use std::path::Path;

use landlock::{
    Access, AccessFs, PathBeneath, PathFd, PathFdError, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetError, RulesetStatus, ABI,
};
use tracing::{debug, warn};

/// Error type for sandbox operations
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Landlock error: {0}")]
    Landlock(#[from] RulesetError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path FD error: {0}")]
    PathFd(#[from] PathFdError),
    #[error("Path validation failed: {0}")]
    PathValidation(String),
}

/// Represents a sandbox configuration for restricting filesystem access
#[derive(Debug, Clone)]
pub struct Sandbox {
    /// The allowed paths and their access modes
    allowed_paths: Vec<SandboxPath>,
}

/// A path with its allowed access mode
#[derive(Debug, Clone)]
pub struct SandboxPath {
    /// The path that should be accessible
    pub path: std::path::PathBuf,
    /// The access mode for this path
    pub access: SandboxAccess,
}

/// Access modes for sandboxed paths
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxAccess {
    /// Read-only access (for fetching/cloning)
    ReadOnly,
    /// Read-write access (for pushing/writing)
    ReadWrite,
    /// Execute access (for running binaries like git)
    Execute,
}

/// Check if landlock is supported on the current system
pub fn is_landlock_supported() -> bool {
    // Try to create a basic ruleset to test support
    let abi = ABI::V5;
    let result = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|err| {
            debug!("Landlock not fully supported: {:?}", err);
            err
        })
        .ok();
    result.is_some()
}

/// Get the best available Landlock ABI version
fn get_best_abi() -> ABI {
    // Try ABIs from newest to oldest
    for abi in [ABI::V5, ABI::V4, ABI::V3, ABI::V2, ABI::V1] {
        if Ruleset::default()
            .handle_access(AccessFs::from_all(abi))
            .is_ok()
        {
            return abi;
        }
    }
    ABI::V1
}

impl Sandbox {
    /// Create a new sandbox with no allowed paths
    pub fn new() -> Self {
        Sandbox {
            allowed_paths: Vec::new(),
        }
    }

    /// Add a path with read-only access
    pub fn allow_read<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.allowed_paths.push(SandboxPath {
            path: path.as_ref().to_path_buf(),
            access: SandboxAccess::ReadOnly,
        });
        self
    }

    /// Add a path with read-write access
    pub fn allow_read_write<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.allowed_paths.push(SandboxPath {
            path: path.as_ref().to_path_buf(),
            access: SandboxAccess::ReadWrite,
        });
        self
    }

    /// Add a path with execute access (for binaries)
    pub fn allow_execute<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.allowed_paths.push(SandboxPath {
            path: path.as_ref().to_path_buf(),
            access: SandboxAccess::Execute,
        });
        self
    }

    /// Apply the sandbox restrictions to the current process
    ///
    /// This should be called after fork() but before exec() when spawning
    /// sandboxed subprocesses.
    ///
    /// Returns Ok(true) if landlock was enforced, Ok(false) if not supported,
    /// or an error if something went wrong.
    pub fn apply(&self) -> Result<bool, SandboxError> {
        let abi = get_best_abi();

        let mut ruleset = Ruleset::default()
            .handle_access(AccessFs::from_all(abi))?
            .create()?;

        for sandbox_path in &self.allowed_paths {
            // Canonicalize the path to resolve symlinks
            let canonical_path = match sandbox_path.path.canonicalize() {
                Ok(p) => p,
                Err(err) => {
                    warn!(
                        "Failed to canonicalize path {:?}: {}",
                        sandbox_path.path, err
                    );
                    continue;
                }
            };

            let fd = PathFd::new(&canonical_path)?;
            let access_bits = match sandbox_path.access {
                SandboxAccess::ReadOnly => AccessFs::from_read(abi),
                SandboxAccess::ReadWrite => AccessFs::from_all(abi),
                SandboxAccess::Execute => {
                    AccessFs::Execute | AccessFs::ReadDir | AccessFs::ReadFile
                }
            };

            ruleset = ruleset.add_rule(PathBeneath::new(fd, access_bits))?;
        }

        let status = ruleset.restrict_self()?;

        match status.ruleset {
            RulesetStatus::FullyEnforced => {
                debug!("Landlock sandbox fully enforced");
                Ok(true)
            }
            RulesetStatus::PartiallyEnforced => {
                debug!("Landlock sandbox partially enforced");
                Ok(true)
            }
            RulesetStatus::NotEnforced => {
                debug!("Landlock sandbox not enforced");
                Ok(false)
            }
        }
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a sandbox configuration for a git repository operation
///
/// This provides:
/// - Read-write access to the repository directory
/// - Read-only access to common git system paths
/// - Execute access to /usr/bin (for git binary)
pub fn create_repo_sandbox<P: AsRef<Path>>(repo_path: P) -> Sandbox {
    Sandbox::new()
        // Allow full access to the repository
        .allow_read_write(repo_path)
        // Allow execute access for git binary and other tools
        .allow_execute("/usr/bin")
        .allow_execute("/usr/lib")
        .allow_execute("/bin")
        .allow_execute("/lib")
        .allow_execute("/lib64")
        // Allow read access to common system paths needed by git
        .allow_read("/etc/gitconfig")
        .allow_read("/etc/passwd")
        .allow_read("/etc/group")
        .allow_read("/etc/nsswitch.conf")
        .allow_read("/etc/hosts")
        .allow_read("/etc/resolv.conf")
        .allow_read("/usr/share/git-core")
        // Allow read access to temporary directories
        .allow_read_write("/tmp")
}

/// Validate that a path is within the allowed base directory
///
/// This function performs the following checks:
/// - The path doesn't contain ".." path traversal components
/// - When canonicalized, the path is a subdirectory of the base path
/// - The path doesn't escape via symlinks
pub fn validate_path_within_base(path: &Path, base: &Path) -> Result<std::path::PathBuf, SandboxError> {
    // First, check for obvious path traversal patterns
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(SandboxError::PathValidation(
            "Path contains '..' traversal".to_string(),
        ));
    }

    // Canonicalize both paths to resolve symlinks
    let canonical_base = base.canonicalize().map_err(|err| {
        SandboxError::PathValidation(format!("Failed to canonicalize base path: {}", err))
    })?;

    let canonical_path = path.canonicalize().map_err(|err| {
        SandboxError::PathValidation(format!("Failed to canonicalize path: {}", err))
    })?;

    // Check that the canonical path is within the canonical base
    if !canonical_path.starts_with(&canonical_base) {
        return Err(SandboxError::PathValidation(format!(
            "Path {:?} is not within base {:?}",
            canonical_path, canonical_base
        )));
    }

    Ok(canonical_path)
}

/// Check if a path component is safe (no path traversal or null bytes)
pub fn is_safe_path_component(component: &str) -> bool {
    if component.is_empty() {
        return false;
    }
    if component == "." || component == ".." {
        return false;
    }
    if component.contains('\0') {
        return false;
    }
    if component.contains('/') || component.contains('\\') {
        return false;
    }
    // Reject hidden files/directories (those starting with .)
    // unless they are git directories
    if component.starts_with('.') && !component.starts_with(".git") {
        // Allow .git but reject other hidden paths
        if component != ".git" && !component.starts_with(".git/") {
            return true; // Actually allow other dotfiles like .gitignore
        }
    }
    true
}

/// Validate a repository path (org/project/repo format)
pub fn validate_repo_path(org: &str, project: &str, repo: &str) -> Result<(), SandboxError> {
    if !is_safe_path_component(org) {
        return Err(SandboxError::PathValidation(format!(
            "Invalid organization name: {}",
            org
        )));
    }
    if !is_safe_path_component(project) {
        return Err(SandboxError::PathValidation(format!(
            "Invalid project name: {}",
            project
        )));
    }
    if !is_safe_path_component(repo) {
        return Err(SandboxError::PathValidation(format!(
            "Invalid repository name: {}",
            repo
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_safe_path_component() {
        assert!(is_safe_path_component("hello"));
        assert!(is_safe_path_component("hello-world"));
        assert!(is_safe_path_component("hello_world"));
        assert!(is_safe_path_component("hello.txt"));
        assert!(is_safe_path_component(".gitignore"));
        assert!(is_safe_path_component(".git"));

        assert!(!is_safe_path_component(""));
        assert!(!is_safe_path_component("."));
        assert!(!is_safe_path_component(".."));
        assert!(!is_safe_path_component("path/to"));
        assert!(!is_safe_path_component("path\\to"));
        assert!(!is_safe_path_component("path\0null"));
    }

    #[test]
    fn test_validate_repo_path() {
        assert!(validate_repo_path("org", "project", "repo").is_ok());
        assert!(validate_repo_path("my-org", "my-project", "my-repo").is_ok());
        assert!(validate_repo_path("org_1", "project_2", "repo_3").is_ok());

        assert!(validate_repo_path("..", "project", "repo").is_err());
        assert!(validate_repo_path("org", "..", "repo").is_err());
        assert!(validate_repo_path("org", "project", "..").is_err());
        assert!(validate_repo_path("org/evil", "project", "repo").is_err());
        assert!(validate_repo_path("org", "project", "repo\0evil").is_err());
    }

    #[test]
    fn test_validate_path_within_base() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create subdirectory
        let sub_dir = base_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Create a file in the subdirectory
        let file_path = sub_dir.join("test.txt");
        fs::write(&file_path, "test").unwrap();

        // Valid paths should pass
        let result = validate_path_within_base(&file_path, base_path);
        assert!(result.is_ok());

        let result = validate_path_within_base(&sub_dir, base_path);
        assert!(result.is_ok());

        // Path traversal should fail (using string-based check)
        let traversal_path = base_path.join("subdir/../../../etc/passwd");
        let result = validate_path_within_base(&traversal_path, base_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_symlink_escape() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create a subdirectory
        let sub_dir = base_path.join("repos");
        fs::create_dir(&sub_dir).unwrap();

        // Create a symlink that points outside the base
        let symlink_path = sub_dir.join("evil_link");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink("/etc", &symlink_path).unwrap();

            // Trying to access via symlink should fail
            let evil_path = symlink_path.join("passwd");
            let result = validate_path_within_base(&evil_path, base_path);
            assert!(result.is_err(), "Symlink escape should be detected");
        }
    }

    #[test]
    fn test_sandbox_builder() {
        let sandbox = Sandbox::new()
            .allow_read("/tmp/read")
            .allow_read_write("/tmp/readwrite")
            .allow_execute("/usr/bin");

        assert_eq!(sandbox.allowed_paths.len(), 3);
        assert_eq!(sandbox.allowed_paths[0].access, SandboxAccess::ReadOnly);
        assert_eq!(sandbox.allowed_paths[1].access, SandboxAccess::ReadWrite);
        assert_eq!(sandbox.allowed_paths[2].access, SandboxAccess::Execute);
    }

    #[test]
    fn test_create_repo_sandbox() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("repo.git");
        fs::create_dir(&repo_path).unwrap();

        let sandbox = create_repo_sandbox(&repo_path);

        // Should have multiple allowed paths
        assert!(!sandbox.allowed_paths.is_empty());

        // First path should be the repo with read-write access
        assert_eq!(sandbox.allowed_paths[0].path, repo_path);
        assert_eq!(sandbox.allowed_paths[0].access, SandboxAccess::ReadWrite);
    }

    #[test]
    fn test_is_landlock_supported() {
        // This just tests that the function runs without panicking
        // The actual result depends on the kernel
        let _supported = is_landlock_supported();
    }
}
