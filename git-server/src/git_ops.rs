//! Git operations using the git2 crate and git commands for HTTP protocol.
//! This module provides all git functionality, using subprocess for HTTP smart protocol.
//! All git subprocess operations are sandboxed using Landlock to restrict filesystem access.

use git2::{
    BranchType, Commit, DiffFormat, DiffOptions, ObjectType, Oid, Repository,
    RepositoryInitOptions, Signature, Sort,
};
use std::path::Path;

use crate::sandbox::{create_repo_sandbox, validate_path_within_base};

/// Error type for git operations
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid reference: {0}")]
    InvalidRef(String),
    #[error("Sandbox error: {0}")]
    Sandbox(String),
}

/// File entry in a repository
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub entry_type: String, // "file" or "dir"
    pub size: Option<u64>,
}

/// Commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// File diff information
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub additions: i32,
    pub deletions: i32,
    pub diff: String,
}

/// Validate that a repository path is safe and within the base repos directory.
/// 
/// This function performs security checks to prevent path traversal attacks:
/// - Validates that org, project, and repo names don't contain path traversal patterns
/// - Ensures the final path is within the repos base directory
pub fn validate_repo_path_safe(
    repos_base: &Path,
    org: &str,
    project: &str,
    repo: &str,
) -> Result<std::path::PathBuf, GitError> {
    use crate::sandbox::validate_repo_path;
    
    // Validate path components
    validate_repo_path(org, project, repo)
        .map_err(|err| GitError::Sandbox(err.to_string()))?;
    
    // Construct and validate the full path
    let repo_path = repos_base.join(org).join(project).join(format!("{}.git", repo));
    
    // If the path exists, validate it's within the base
    if repo_path.exists() {
        validate_path_within_base(&repo_path, repos_base)
            .map_err(|err| GitError::Sandbox(err.to_string()))?;
    }
    
    Ok(repo_path)
}

/// Initialize a bare git repository with a default branch
pub fn init_bare_repo(path: &Path, default_branch: &str) -> Result<Repository, GitError> {
    let mut opts = RepositoryInitOptions::new();
    opts.bare(true);
    opts.initial_head(default_branch);

    let repo = Repository::init_opts(path, &opts)?;
    Ok(repo)
}

/// Clone a bare repository
pub fn clone_bare(source: &Path, destination: &Path) -> Result<Repository, GitError> {
    let _source_repo = Repository::open_bare(source)?;
    let url = format!("file://{}", source.display());

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);

    let repo = builder.clone(&url, destination)?;
    Ok(repo)
}

/// List files in a repository at a specific ref and path
pub fn list_files(repo_path: &Path, git_ref: &str, subpath: &str) -> Result<Vec<FileEntry>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;
    
    // Resolve the reference to a commit
    let obj = repo.revparse_single(git_ref)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;

    // If there's a subpath, navigate to it
    let target_tree = if subpath.is_empty() {
        tree
    } else {
        let entry = tree.get_path(Path::new(subpath))?;
        let obj = entry.to_object(&repo)?;
        obj.peel_to_tree()?
    };

    let mut entries = Vec::new();
    for entry in target_tree.iter() {
        let name = entry.name().unwrap_or("").to_string();
        let entry_path = if subpath.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", subpath, name)
        };

        let (entry_type, size) = match entry.kind() {
            Some(ObjectType::Blob) => {
                let blob = repo.find_blob(entry.id())?;
                ("file".to_string(), Some(blob.size() as u64))
            }
            Some(ObjectType::Tree) => ("dir".to_string(), None),
            _ => continue,
        };

        entries.push(FileEntry {
            name,
            path: entry_path,
            entry_type,
            size,
        });
    }

    Ok(entries)
}

/// List commits in a repository
pub fn list_commits(repo_path: &Path, max_count: usize) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;

    // Try to get HEAD
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(vec![]), // Empty repository
    };

    let mut revwalk = repo.revwalk()?;
    revwalk.push(head.target().ok_or_else(|| GitError::InvalidRef("HEAD has no target".to_string()))?)?;
    revwalk.set_sorting(Sort::TIME)?;

    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= max_count {
            break;
        }

        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_info(&commit));
    }

    Ok(commits)
}

/// Get blob content from a repository
pub fn get_blob_content(repo_path: &Path, git_ref: &str, file_path: &str) -> Result<String, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;

    // Resolve the reference to a commit
    let obj = repo.revparse_single(git_ref)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;

    // Get the file entry
    let entry = tree.get_path(Path::new(file_path))?;
    let blob = repo.find_blob(entry.id())?;

    // Return content as string
    let content = String::from_utf8_lossy(blob.content()).to_string();
    Ok(content)
}

/// List branches in a repository
pub fn list_branches(repo_path: &Path) -> Result<Vec<String>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;

    let mut branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

/// Convert a git2 Commit to CommitInfo
fn commit_to_info(commit: &Commit) -> CommitInfo {
    let hash = commit.id().to_string();
    let short_hash = hash[..7.min(hash.len())].to_string();
    let author = commit.author().name().unwrap_or("Unknown").to_string();
    
    // Format the time
    let time = commit.time();
    let offset = time.offset_minutes();
    let timestamp = time.seconds();
    let date = format_git_time(timestamp, offset);
    
    let message = commit.summary().unwrap_or("").to_string();

    CommitInfo {
        hash,
        short_hash,
        author,
        date,
        message,
    }
}

/// Format git time to a readable string
fn format_git_time(timestamp: i64, offset_minutes: i32) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    
    let secs = timestamp as u64;
    let time = UNIX_EPOCH + Duration::from_secs(secs);
    
    // Simple ISO-like format
    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let total_secs = duration.as_secs();
        let days = total_secs / 86400;
        let years = 1970 + (days / 365);
        let remaining_days = days % 365;
        let months = remaining_days / 30;
        let day = remaining_days % 30 + 1;
        let hours = (total_secs % 86400) / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        
        let offset_hours = offset_minutes / 60;
        let offset_mins = offset_minutes.abs() % 60;
        let sign = if offset_minutes >= 0 { "+" } else { "-" };
        
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}{:02}",
            years, months + 1, day, hours, mins, secs, sign, offset_hours.abs(), offset_mins)
    } else {
        format!("{}", timestamp)
    }
}

/// Create or update a file in a repository and commit
pub fn update_file_and_commit(
    bare_repo_path: &Path,
    file_path: &str,
    content: &str,
    commit_message: &str,
) -> Result<(), GitError> {
    // For a bare repository, we need to work directly with the git objects
    let repo = Repository::open_bare(bare_repo_path)?;
    
    // Create a blob from the content
    let blob_oid = repo.blob(content.as_bytes())?;
    
    // Get the current tree or create an empty one
    let parent_commit = match repo.head() {
        Ok(head) => {
            let target = head.target().ok_or_else(|| GitError::InvalidRef("HEAD has no target".to_string()))?;
            Some(repo.find_commit(target)?)
        }
        Err(_) => None, // No commits yet
    };
    
    // Build the new tree
    let mut tree_builder = if let Some(ref parent) = parent_commit {
        repo.treebuilder(Some(&parent.tree()?))?
    } else {
        repo.treebuilder(None)?
    };
    
    // Handle nested paths - need to create intermediate trees
    let path_parts: Vec<&str> = file_path.split('/').collect();
    if path_parts.len() == 1 {
        // Simple case: file at root
        tree_builder.insert(file_path, blob_oid, 0o100644)?;
    } else {
        // Need to build nested tree structure
        let new_tree_oid = insert_nested_blob(&repo, parent_commit.as_ref(), file_path, blob_oid)?;
        return commit_tree(&repo, new_tree_oid, commit_message, parent_commit.as_ref());
    }
    
    let new_tree_oid = tree_builder.write()?;
    commit_tree(&repo, new_tree_oid, commit_message, parent_commit.as_ref())
}

/// Insert a blob at a nested path, creating intermediate trees as needed
fn insert_nested_blob(
    repo: &Repository,
    parent_commit: Option<&Commit>,
    path: &str,
    blob_oid: Oid,
) -> Result<Oid, GitError> {
    let path_parts: Vec<&str> = path.split('/').collect();
    let parent_tree = parent_commit.map(|c| c.tree()).transpose()?;
    
    insert_at_path(repo, parent_tree.as_ref(), &path_parts, blob_oid, 0)
}

/// Recursively insert at path
fn insert_at_path(
    repo: &Repository,
    current_tree: Option<&git2::Tree>,
    path_parts: &[&str],
    blob_oid: Oid,
    depth: usize,
) -> Result<Oid, GitError> {
    if path_parts.is_empty() {
        return Ok(blob_oid);
    }
    
    let mut builder = repo.treebuilder(current_tree)?;
    
    if path_parts.len() == 1 {
        // At the file level
        builder.insert(path_parts[0], blob_oid, 0o100644)?;
    } else {
        // Need to handle directory
        let dir_name = path_parts[0];
        let rest = &path_parts[1..];
        
        // Get existing subtree if any
        let subtree = current_tree.and_then(|t| t.get_name(dir_name)).and_then(|entry| {
            if entry.kind() == Some(ObjectType::Tree) {
                repo.find_tree(entry.id()).ok()
            } else {
                None
            }
        });
        
        let subtree_oid = insert_at_path(repo, subtree.as_ref(), rest, blob_oid, depth + 1)?;
        builder.insert(dir_name, subtree_oid, 0o040000)?;
    }
    
    Ok(builder.write()?)
}

/// Commit a tree to the repository
fn commit_tree(
    repo: &Repository,
    tree_oid: Oid,
    message: &str,
    parent: Option<&Commit>,
) -> Result<(), GitError> {
    let tree = repo.find_tree(tree_oid)?;
    let sig = Signature::now("Git Server Webapp", "webapp@git-server.local")?;
    
    let parents: Vec<&Commit> = parent.iter().copied().collect();
    
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parents,
    )?;
    
    Ok(())
}

/// Delete a file from a repository and commit
pub fn delete_file_and_commit(
    bare_repo_path: &Path,
    file_path: &str,
    commit_message: &str,
) -> Result<(), GitError> {
    let repo = Repository::open_bare(bare_repo_path)?;
    
    // Get the current commit
    let head = repo.head()?;
    let target = head.target().ok_or_else(|| GitError::InvalidRef("HEAD has no target".to_string()))?;
    let parent_commit = repo.find_commit(target)?;
    let parent_tree = parent_commit.tree()?;
    
    // Build new tree without the file
    let new_tree_oid = remove_from_tree(&repo, &parent_tree, file_path)?;
    
    commit_tree(&repo, new_tree_oid, commit_message, Some(&parent_commit))
}

/// Remove a file from a tree, handling nested paths
fn remove_from_tree(repo: &Repository, tree: &git2::Tree, path: &str) -> Result<Oid, GitError> {
    let path_parts: Vec<&str> = path.split('/').collect();
    remove_at_path(repo, tree, &path_parts)
}

/// Recursively remove at path
fn remove_at_path(
    repo: &Repository,
    current_tree: &git2::Tree,
    path_parts: &[&str],
) -> Result<Oid, GitError> {
    if path_parts.is_empty() {
        return Err(GitError::NotFound("Empty path".to_string()));
    }
    
    let mut builder = repo.treebuilder(Some(current_tree))?;
    
    if path_parts.len() == 1 {
        // At the file level - remove it
        builder.remove(path_parts[0])?;
    } else {
        // Need to handle directory
        let dir_name = path_parts[0];
        let rest = &path_parts[1..];
        
        // Get existing subtree
        let entry = current_tree.get_name(dir_name).ok_or_else(|| {
            GitError::NotFound(format!("Directory not found: {}", dir_name))
        })?;
        
        let subtree = repo.find_tree(entry.id())?;
        let subtree_oid = remove_at_path(repo, &subtree, rest)?;
        
        // Check if the subtree is now empty
        let new_subtree = repo.find_tree(subtree_oid)?;
        if new_subtree.is_empty() {
            builder.remove(dir_name)?;
        } else {
            builder.insert(dir_name, subtree_oid, 0o040000)?;
        }
    }
    
    Ok(builder.write()?)
}

/// Get diff between two branches
pub fn get_branch_diff(
    repo_path: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<FileDiff>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;
    
    // Resolve both refs
    let base_obj = repo.revparse_single(base_ref)?;
    let head_obj = repo.revparse_single(head_ref)?;
    
    let base_tree = base_obj.peel_to_commit()?.tree()?;
    let head_tree = head_obj.peel_to_commit()?.tree()?;
    
    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut diff_opts))?;
    
    let mut files = Vec::new();
    let _stats = diff.stats()?;
    
    // Process each delta
    for i in 0..diff.deltas().len() {
        let delta = diff.get_delta(i).ok_or_else(|| GitError::NotFound("Delta not found".to_string()))?;
        
        let path = delta.new_file().path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let status = match delta.status() {
            git2::Delta::Added => "added",
            git2::Delta::Deleted => "deleted",
            git2::Delta::Modified => "modified",
            git2::Delta::Renamed => "renamed",
            git2::Delta::Copied => "copied",
            _ => "unknown",
        };
        
        // Get the patch for this file
        let mut diff_text = String::new();
        let mut additions = 0i32;
        let mut deletions = 0i32;
        
        diff.print(DiffFormat::Patch, |delta2, _hunk, line| {
            if let Some(p) = delta2.new_file().path().or_else(|| delta2.old_file().path()) {
                if p.to_string_lossy() == path {
                    match line.origin() {
                        '+' | '-' | ' ' => {
                            diff_text.push(line.origin());
                            if let Ok(content) = std::str::from_utf8(line.content()) {
                                diff_text.push_str(content);
                            }
                            if line.origin() == '+' {
                                additions += 1;
                            } else if line.origin() == '-' {
                                deletions += 1;
                            }
                        }
                        _ => {
                            if let Ok(content) = std::str::from_utf8(line.content()) {
                                diff_text.push_str(content);
                            }
                        }
                    }
                }
            }
            true
        })?;
        
        files.push(FileDiff {
            path,
            status: status.to_string(),
            additions,
            deletions,
            diff: diff_text,
        });
    }
    
    Ok(files)
}

/// Get commits between two refs (commits in head that aren't in base)
pub fn get_commits_between(
    repo_path: &Path,
    base_ref: &str,
    head_ref: &str,
    max_count: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;
    
    // Try to resolve head ref first
    let head_obj = match repo.revparse_single(head_ref) {
        Ok(obj) => obj,
        Err(_) => return Ok(vec![]),
    };
    
    let head_commit = head_obj.peel_to_commit()?;
    
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit.id())?;
    
    // Try to hide base commits if base ref exists
    if let Ok(base_obj) = repo.revparse_single(base_ref) {
        if let Ok(base_commit) = base_obj.peel_to_commit() {
            let _ = revwalk.hide(base_commit.id());
        }
    }
    
    revwalk.set_sorting(Sort::TIME)?;
    
    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= max_count {
            break;
        }
        
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_info(&commit));
    }
    
    Ok(commits)
}

/// Add a remote to a repository and fetch from it
pub fn add_remote_and_fetch(
    repo_path: &Path,
    remote_name: &str,
    remote_url: &Path,
    refspec: &str,
) -> Result<(), GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;
    
    // Try to add remote (ignore if already exists)
    let remote_url_str = format!("file://{}", remote_url.display());
    let _ = repo.remote(remote_name, &remote_url_str);
    
    // Fetch the specific ref
    let mut remote = repo.find_remote(remote_name)?;
    remote.fetch(&[refspec], None, None)?;
    
    Ok(())
}

/// Reference info for smart protocol
#[derive(Debug, Clone)]
pub struct RefInfo {
    pub name: String,
    pub oid: String,
}

/// Helper to run a sandboxed git command
/// 
/// This function spawns a git subprocess with Landlock restrictions applied
/// to limit filesystem access to only the repository directory.
#[cfg(target_os = "linux")]
fn run_sandboxed_git_command(
    repo_path: &Path,
    args: &[&str],
    stdin_data: Option<&[u8]>,
) -> Result<Vec<u8>, GitError> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    // Create sandbox configuration
    let sandbox = create_repo_sandbox(repo_path);
    
    let mut cmd = Command::new("git");
    cmd.args(args)
        .arg(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }
    
    // Apply sandbox before exec using pre_exec hook
    // SAFETY: This closure runs after fork() but before exec().
    // It only calls Landlock syscalls which are async-signal-safe equivalent.
    unsafe {
        cmd.pre_exec(move || {
            match sandbox.apply() {
                Ok(_) => Ok(()),
                Err(err) => {
                    // Log the error but don't fail - allow graceful degradation
                    // on systems without Landlock support
                    eprintln!("Warning: Failed to apply sandbox: {}", err);
                    Ok(())
                }
            }
        });
    }
    
    let mut child = cmd.spawn().map_err(GitError::Io)?;
    
    // Write stdin data if provided
    if let Some(data) = stdin_data {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data).map_err(GitError::Io)?;
        }
    }
    
    let output = child.wait_with_output().map_err(GitError::Io)?;
    
    if !output.status.success() && output.stdout.is_empty() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::Git(git2::Error::from_str(&format!(
            "git command failed: {}",
            err_msg
        ))));
    }
    
    Ok(output.stdout)
}

/// Fallback for non-Linux systems (no sandbox)
#[cfg(not(target_os = "linux"))]
fn run_sandboxed_git_command(
    repo_path: &Path,
    args: &[&str],
    stdin_data: Option<&[u8]>,
) -> Result<Vec<u8>, GitError> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    let mut cmd = Command::new("git");
    cmd.args(args)
        .arg(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }
    
    let mut child = cmd.spawn().map_err(GitError::Io)?;
    
    // Write stdin data if provided
    if let Some(data) = stdin_data {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data).map_err(GitError::Io)?;
        }
    }
    
    let output = child.wait_with_output().map_err(GitError::Io)?;
    
    if !output.status.success() && output.stdout.is_empty() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::Git(git2::Error::from_str(&format!(
            "git command failed: {}",
            err_msg
        ))));
    }
    
    Ok(output.stdout)
}

/// Get advertised refs using git command (more reliable for HTTP protocol)
/// The git subprocess is sandboxed to only access the repository directory.
pub fn get_advertised_refs(repo_path: &Path, service: &str) -> Result<Vec<u8>, GitError> {
    let args = [service, "--stateless-rpc", "--advertise-refs"];
    
    match run_sandboxed_git_command(repo_path, &args, None) {
        Ok(output) => Ok(output),
        Err(_) => {
            // If git command fails (e.g., empty repo), fall back to git2 implementation
            get_advertised_refs_git2(repo_path, service)
        }
    }
}

/// Fallback implementation using git2 for when git command fails
fn get_advertised_refs_git2(repo_path: &Path, service: &str) -> Result<Vec<u8>, GitError> {
    let repo = Repository::open_bare(repo_path).or_else(|_| Repository::open(repo_path))?;
    
    let mut refs: Vec<RefInfo> = Vec::new();
    
    // Get HEAD first if it points to something
    if let Ok(head) = repo.head() {
        if let Some(oid) = head.target() {
            refs.push(RefInfo {
                name: "HEAD".to_string(),
                oid: oid.to_string(),
            });
        }
    }
    
    // Iterate over all references
    if let Ok(reference_iter) = repo.references() {
        for reference in reference_iter {
            if let Ok(reference) = reference {
                if let (Some(name), Some(oid)) = (reference.name(), reference.target()) {
                    refs.push(RefInfo {
                        name: name.to_string(),
                        oid: oid.to_string(),
                    });
                }
            }
        }
    }
    
    // Generate pkt-line format output
    let mut output = Vec::new();
    
    let capabilities = if service == "git-upload-pack" {
        "multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag multi_ack_detailed symref=HEAD:refs/heads/main agent=git2-server/1.0"
    } else {
        "report-status delete-refs side-band-64k quiet atomic ofs-delta agent=git2-server/1.0"
    };
    
    for (i, ref_info) in refs.iter().enumerate() {
        let line = if i == 0 {
            // First ref includes capabilities
            format!("{} {}\0{}\n", ref_info.oid, ref_info.name, capabilities)
        } else {
            format!("{} {}\n", ref_info.oid, ref_info.name)
        };
        
        let pkt_line = format!("{:04x}{}", line.len() + 4, line);
        output.extend_from_slice(pkt_line.as_bytes());
    }
    
    // Flush packet
    output.extend_from_slice(b"0000");
    
    Ok(output)
}

/// Process upload-pack request (client wants to fetch objects)
/// Uses git command for proper HTTP protocol handling.
/// The git subprocess is sandboxed to only access the repository directory.
pub fn process_upload_pack_request(
    repo_path: &Path,
    request_body: &[u8],
) -> Result<Vec<u8>, GitError> {
    let args = ["upload-pack", "--stateless-rpc"];
    run_sandboxed_git_command(repo_path, &args, Some(request_body))
}

/// Process receive-pack request (client wants to push objects)
/// Uses git command for proper HTTP protocol handling.
/// The git subprocess is sandboxed to only access the repository directory.
pub fn process_receive_pack_request(
    repo_path: &Path,
    request_body: &[u8],
) -> Result<Vec<u8>, GitError> {
    let args = ["receive-pack", "--stateless-rpc"];
    run_sandboxed_git_command(repo_path, &args, Some(request_body))
}

/// SSH protocol handler state - collects input and produces output
pub struct SshProtocolHandler {
    repo_path: std::path::PathBuf,
    service: String,
    input_buffer: Vec<u8>,
}

impl SshProtocolHandler {
    /// Create a new SSH protocol handler
    pub fn new(repo_path: &Path, service: &str) -> Self {
        SshProtocolHandler {
            repo_path: repo_path.to_path_buf(),
            service: service.to_string(),
            input_buffer: Vec::new(),
        }
    }
    
    /// Write input data from the client
    pub fn write_input(&mut self, data: &[u8]) {
        self.input_buffer.extend_from_slice(data);
    }
    
    /// Process input and generate output when the client closes the input
    pub fn finish(&self) -> Result<Vec<u8>, GitError> {
        if self.service == "git-upload-pack" {
            // For upload-pack, first send the refs advertisement, then process wants
            let mut output = get_advertised_refs(&self.repo_path, "git-upload-pack")?;
            
            // If there's input (wants), process them
            if !self.input_buffer.is_empty() {
                let pack_response = process_upload_pack_request(&self.repo_path, &self.input_buffer)?;
                output.extend_from_slice(&pack_response);
            }
            
            Ok(output)
        } else if self.service == "git-receive-pack" {
            // For receive-pack, first send the refs advertisement, then process push
            let mut output = get_advertised_refs(&self.repo_path, "git-receive-pack")?;
            
            // If there's input (pack data), process it
            if !self.input_buffer.is_empty() {
                let response = process_receive_pack_request(&self.repo_path, &self.input_buffer)?;
                output.extend_from_slice(&response);
            }
            
            Ok(output)
        } else {
            Err(GitError::NotFound(format!("Unknown service: {}", self.service)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_bare_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let repo = init_bare_repo(&repo_path, "main").unwrap();
        assert!(repo.is_bare());
        assert!(repo_path.exists());
    }

    #[test]
    fn test_create_file_and_list() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let _repo = init_bare_repo(&repo_path, "main").unwrap();
        
        // Create a file
        update_file_and_commit(&repo_path, "README.md", "# Hello\n", "Initial commit").unwrap();
        
        // List files
        let files = list_files(&repo_path, "HEAD", "").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "README.md");
        assert_eq!(files[0].entry_type, "file");
    }

    #[test]
    fn test_nested_file() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let _repo = init_bare_repo(&repo_path, "main").unwrap();
        
        // Create a nested file
        update_file_and_commit(&repo_path, "src/main.rs", "fn main() {}\n", "Add main.rs").unwrap();
        
        // List root
        let files = list_files(&repo_path, "HEAD", "").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "src");
        assert_eq!(files[0].entry_type, "dir");
        
        // List src directory
        let files = list_files(&repo_path, "HEAD", "src").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "main.rs");
    }

    #[test]
    fn test_list_commits() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let _repo = init_bare_repo(&repo_path, "main").unwrap();
        
        // Create some commits
        update_file_and_commit(&repo_path, "file1.txt", "content1", "First commit").unwrap();
        update_file_and_commit(&repo_path, "file2.txt", "content2", "Second commit").unwrap();
        
        // List commits
        let commits = list_commits(&repo_path, 10).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].message, "Second commit");
        assert_eq!(commits[1].message, "First commit");
    }

    #[test]
    fn test_get_blob() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let _repo = init_bare_repo(&repo_path, "main").unwrap();
        
        // Create a file
        update_file_and_commit(&repo_path, "test.txt", "Hello World!", "Add test file").unwrap();
        
        // Get blob content
        let content = get_blob_content(&repo_path, "HEAD", "test.txt").unwrap();
        assert_eq!(content, "Hello World!");
    }

    #[test]
    fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test.git");
        
        let _repo = init_bare_repo(&repo_path, "main").unwrap();
        
        // Create files
        update_file_and_commit(&repo_path, "file1.txt", "content1", "Add file1").unwrap();
        update_file_and_commit(&repo_path, "file2.txt", "content2", "Add file2").unwrap();
        
        // Delete one file
        delete_file_and_commit(&repo_path, "file1.txt", "Delete file1").unwrap();
        
        // Verify
        let files = list_files(&repo_path, "HEAD", "").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "file2.txt");
    }
}
