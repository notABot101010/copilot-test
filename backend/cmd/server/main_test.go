package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestRepositoryAndMergeRequestFlow(t *testing.T) {
	t.Parallel()

	gitBinary, err := exec.LookPath("git")
	if err != nil {
		t.Fatalf("git not found: %v", err)
	}

	reposRoot := t.TempDir()
	staticRoot := t.TempDir()
	if err := os.MkdirAll(staticRoot, 0o755); err != nil {
		t.Fatalf("create static root: %v", err)
	}

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot, gitBinary: gitBinary}
	server := httptest.NewServer(a.httpRouter())
	defer server.Close()

	var user User
	requestJSON(t, server, http.MethodPost, "/api/users", "", map[string]string{"username": "alice"}, &user)

	var org Organization
	requestJSON(t, server, http.MethodPost, "/api/orgs", user.Token, map[string]string{"name": "acme"}, &org)

	var project Project
	requestJSON(t, server, http.MethodPost, "/api/orgs/1/projects", user.Token, map[string]string{"name": "demo"}, &project)

	var issue Issue
	requestJSON(t, server, http.MethodPost, "/api/projects/1/issues", user.Token, map[string]any{
		"title":       "Bug in README",
		"description": "Needs update",
		"tags":        []string{"bug", " docs ", "bug"},
	}, &issue)
	if issue.Title != "Bug in README" || issue.Status != "open" {
		t.Fatalf("unexpected issue payload: %+v", issue)
	}
	if len(issue.Tags) != 2 || issue.Tags[0] != "bug" || issue.Tags[1] != "docs" {
		t.Fatalf("unexpected issue tags: %+v", issue.Tags)
	}

	requestJSON(t, server, http.MethodPatch, "/api/projects/1/issues/1", user.Token, map[string]any{
		"status": "closed",
		"tags":   []string{"maintenance"},
	}, &issue)
	if issue.Status != "closed" {
		t.Fatalf("unexpected issue status: %q", issue.Status)
	}
	if len(issue.Tags) != 1 || issue.Tags[0] != "maintenance" {
		t.Fatalf("unexpected updated tags: %+v", issue.Tags)
	}

	var issues []Issue
	requestJSON(t, server, http.MethodGet, "/api/projects/1/issues", "", nil, &issues)
	if len(issues) != 1 || issues[0].Status != "closed" || len(issues[0].Tags) != 1 || issues[0].Tags[0] != "maintenance" {
		t.Fatalf("unexpected issues list: %+v", issues)
	}

	var saved RepoFile
	requestJSON(t, server, http.MethodPut, "/api/projects/1/repo/file", user.Token, map[string]string{
		"branch":  "main",
		"path":    "README.md",
		"content": "hello\n",
	}, &saved)

	var branches []RepoBranch
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/branches", "", nil, &branches)
	if len(branches) == 0 || branches[0].Name != "main" {
		t.Fatalf("unexpected branches: %+v", branches)
	}

	var tree []RepoEntry
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/tree?branch=main", "", nil, &tree)
	if len(tree) != 1 || tree[0].Path != "README.md" {
		t.Fatalf("unexpected tree: %+v", tree)
	}

	var file RepoFile
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/file?branch=main&path=README.md", "", nil, &file)
	if file.Content != "hello\n" {
		t.Fatalf("unexpected file content: %q", file.Content)
	}

	requestJSON(t, server, http.MethodPut, "/api/projects/1/repo/file", user.Token, map[string]string{
		"branch":  "feature",
		"path":    "README.md",
		"content": "hello\nworld\n",
	}, &saved)

	var mr MergeRequest
	requestJSON(t, server, http.MethodPost, "/api/projects/1/merge-requests", user.Token, map[string]string{
		"title":        "Add world",
		"description":  "demo",
		"sourceBranch": "feature",
		"targetBranch": "main",
	}, &mr)

	var diffResponse map[string]string
	requestJSON(t, server, http.MethodGet, "/api/projects/1/merge-requests/1/diff", "", nil, &diffResponse)
	if !bytes.Contains([]byte(diffResponse["diff"]), []byte("world")) {
		t.Fatalf("expected diff to include world: %q", diffResponse["diff"])
	}

	var comment MergeRequestComment
	requestJSON(t, server, http.MethodPost, "/api/projects/1/merge-requests/1/comments", user.Token, map[string]string{"body": "looks good"}, &comment)
	if comment.Body != "looks good" {
		t.Fatalf("unexpected comment body: %q", comment.Body)
	}

	requestJSON(t, server, http.MethodDelete, "/api/projects/1/repo/file", user.Token, map[string]string{
		"branch": "feature",
		"path":   "README.md",
	}, nil)
}

func TestResolveRepoPathRejectsSymlinkEscape(t *testing.T) {
	t.Parallel()

	reposRoot := t.TempDir()
	outsideRoot := t.TempDir()
	linkPath := filepath.Join(reposRoot, "acme")
	if err := os.Symlink(outsideRoot, linkPath); err != nil {
		t.Fatalf("create symlink: %v", err)
	}

	a := &app{reposRoot: reposRoot}
	_, err := a.resolveRepoPath("acme/demo.git")
	if err == nil {
		t.Fatal("expected symlink attack error")
	}
	if !errors.Is(err, errSymlinkAttack) {
		t.Fatalf("expected errSymlinkAttack, got %v", err)
	}
}

func TestRepositoryManagementAPIs(t *testing.T) {
	t.Parallel()

	gitBinary, err := exec.LookPath("git")
	if err != nil {
		t.Fatalf("git not found: %v", err)
	}

	reposRoot := t.TempDir()
	staticRoot := t.TempDir()
	if err := os.MkdirAll(staticRoot, 0o755); err != nil {
		t.Fatalf("create static root: %v", err)
	}

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot, gitBinary: gitBinary, noHooksDir: filepath.Join(reposRoot, ".nohooks")}
	if err := os.MkdirAll(a.noHooksDir, 0o755); err != nil {
		t.Fatalf("create no-hooks dir: %v", err)
	}
	server := httptest.NewServer(a.httpRouter())
	defer server.Close()

	var alice User
	requestJSON(t, server, http.MethodPost, "/api/users", "", map[string]string{"username": "alice"}, &alice)

	var bob User
	requestJSON(t, server, http.MethodPost, "/api/users", "", map[string]string{"username": "bob"}, &bob)

	var users []User
	requestJSON(t, server, http.MethodGet, "/api/users", "", nil, &users)
	if len(users) != 2 {
		t.Fatalf("expected 2 users, got %+v", users)
	}

	var org Organization
	requestJSON(t, server, http.MethodPost, "/api/orgs", alice.Token, map[string]string{"name": "acme"}, &org)

	var member OrganizationMember
	requestJSON(t, server, http.MethodPost, "/api/orgs/1/members", alice.Token, map[string]any{"userId": bob.ID, "role": "developer"}, &member)
	if member.Role != "developer" {
		t.Fatalf("unexpected member role: %+v", member)
	}

	var members []OrganizationMember
	requestJSON(t, server, http.MethodGet, "/api/orgs/1/members", "", nil, &members)
	if len(members) != 2 {
		t.Fatalf("unexpected org members: %+v", members)
	}

	requestJSON(t, server, http.MethodPatch, "/api/orgs/1/members/2", alice.Token, map[string]string{"role": "admin"}, &member)
	if member.Role != "admin" {
		t.Fatalf("expected admin role, got %+v", member)
	}

	var project Project
	requestJSON(t, server, http.MethodPost, "/api/orgs/1/projects", alice.Token, map[string]string{"name": "demo"}, &project)
	if project.DefaultBranch != "main" {
		t.Fatalf("unexpected default branch: %+v", project)
	}

	requestJSON(t, server, http.MethodPut, "/api/projects/1/repo/file", alice.Token, map[string]string{
		"branch":  "main",
		"path":    "README.md",
		"content": "hello\n",
	}, nil)

	var createdBranch RepoBranch
	requestJSON(t, server, http.MethodPost, "/api/projects/1/repo/branches", bob.Token, map[string]string{
		"name":         "feature",
		"sourceBranch": "main",
	}, &createdBranch)
	if createdBranch.Name != "feature" {
		t.Fatalf("unexpected branch response: %+v", createdBranch)
	}

	requestJSON(t, server, http.MethodPut, "/api/projects/1/repo/file", bob.Token, map[string]string{
		"branch":  "feature",
		"path":    "README.md",
		"content": "hello\nworld\n",
	}, nil)

	var commits []RepoCommit
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/commits?branch=feature", "", nil, &commits)
	if len(commits) == 0 {
		t.Fatal("expected commit history")
	}

	var commit RepoCommitDetails
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/commits/"+commits[0].Hash, "", nil, &commit)
	if commit.Hash == "" || commit.Diff == "" {
		t.Fatalf("unexpected commit details: %+v", commit)
	}

	var blame []RepoBlameLine
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/blame?branch=feature&path=README.md", "", nil, &blame)
	if len(blame) != 2 || blame[1].Content != "world" {
		t.Fatalf("unexpected blame: %+v", blame)
	}

	var tag RepoTag
	requestJSON(t, server, http.MethodPost, "/api/projects/1/repo/tags", bob.Token, map[string]string{
		"name":   "v1.0.0-feature",
		"target": "feature",
	}, &tag)
	if tag.Name != "v1.0.0-feature" {
		t.Fatalf("unexpected tag response: %+v", tag)
	}

	var tags []RepoTag
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/tags", "", nil, &tags)
	if len(tags) == 0 {
		t.Fatalf("expected tags, got %+v", tags)
	}

	var mr MergeRequest
	requestJSON(t, server, http.MethodPost, "/api/projects/1/merge-requests", bob.Token, map[string]string{
		"title":        "Add world",
		"description":  "merge me",
		"sourceBranch": "feature",
		"targetBranch": "main",
	}, &mr)
	if !mr.Mergeable || mr.HasConflicts {
		t.Fatalf("unexpected merge request state: %+v", mr)
	}

	var mergeStatus map[string]bool
	requestJSON(t, server, http.MethodGet, "/api/projects/1/merge-requests/1/merge-status", "", nil, &mergeStatus)
	if !mergeStatus["mergeable"] || mergeStatus["hasConflicts"] {
		t.Fatalf("unexpected merge status: %+v", mergeStatus)
	}

	requestJSON(t, server, http.MethodPost, "/api/projects/1/merge-requests/1/merge", bob.Token, nil, &mr)
	if mr.Status != "merged" || mr.MergedCommitID == "" {
		t.Fatalf("unexpected merged MR: %+v", mr)
	}

	var file RepoFile
	requestJSON(t, server, http.MethodGet, "/api/projects/1/repo/file?branch=main&path=README.md", "", nil, &file)
	if file.Content != "hello\nworld\n" {
		t.Fatalf("unexpected merged file content: %q", file.Content)
	}

	requestJSON(t, server, http.MethodDelete, "/api/projects/1/repo/branches/feature", bob.Token, nil, nil)

	requestJSON(t, server, http.MethodPatch, "/api/projects/1/settings", bob.Token, map[string]any{
		"description":   "demo repository",
		"defaultBranch": "main",
		"archived":      true,
	}, &project)
	if !project.Archived || project.Description != "demo repository" {
		t.Fatalf("unexpected project settings: %+v", project)
	}

	status, body := requestStatus(t, server, http.MethodPost, "/api/projects/1/repo/branches", bob.Token, map[string]string{
		"name":         "blocked",
		"sourceBranch": "main",
	})
	if status != http.StatusForbidden {
		t.Fatalf("expected archived repo write to fail, got %d: %s", status, body)
	}
}

func TestSaveRepoFileBlocksSymlinkEscape(t *testing.T) {
	t.Parallel()

	gitBinary, err := exec.LookPath("git")
	if err != nil {
		t.Fatalf("git not found: %v", err)
	}

	reposRoot := t.TempDir()
	staticRoot := t.TempDir()
	if err := os.MkdirAll(staticRoot, 0o755); err != nil {
		t.Fatalf("create static root: %v", err)
	}

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot, gitBinary: gitBinary, noHooksDir: filepath.Join(reposRoot, ".nohooks")}
	if err := os.MkdirAll(a.noHooksDir, 0o755); err != nil {
		t.Fatalf("create no-hooks dir: %v", err)
	}
	server := httptest.NewServer(a.httpRouter())
	defer server.Close()

	var user User
	requestJSON(t, server, http.MethodPost, "/api/users", "", map[string]string{"username": "alice"}, &user)

	var org Organization
	requestJSON(t, server, http.MethodPost, "/api/orgs", user.Token, map[string]string{"name": "acme"}, &org)

	var project Project
	requestJSON(t, server, http.MethodPost, "/api/orgs/1/projects", user.Token, map[string]string{"name": "demo"}, &project)
	requestJSON(t, server, http.MethodPut, "/api/projects/1/repo/file", user.Token, map[string]string{
		"branch":  "main",
		"path":    "README.md",
		"content": "hello\n",
	}, nil)

	repoPath, err := a.resolveRepoPath(project.RepoRel)
	if err != nil {
		t.Fatalf("resolve repo path: %v", err)
	}

	worktreeDir := filepath.Join(t.TempDir(), "repo")
	runGit(t, "", "clone", "--quiet", repoPath, worktreeDir)
	runGit(t, worktreeDir, "checkout", "--quiet", "-B", "main", "origin/main")

	outsideDir := t.TempDir()
	if err := os.Symlink(outsideDir, filepath.Join(worktreeDir, "nested")); err != nil {
		t.Fatalf("create worktree symlink: %v", err)
	}
	runGit(t, worktreeDir, "add", "--", "nested")
	runGit(t, worktreeDir, "-c", "user.name=alice", "-c", "user.email=alice@example.invalid", "commit", "-m", "add nested symlink")
	runGit(t, worktreeDir, "push", "--quiet", "origin", "HEAD:refs/heads/main")

	status, body := requestStatus(t, server, http.MethodPut, "/api/projects/1/repo/file", user.Token, map[string]string{
		"branch":  "main",
		"path":    "nested/pwned.txt",
		"content": "blocked\n",
	})
	if status != http.StatusConflict {
		t.Fatalf("expected status %d, got %d: %s", http.StatusConflict, status, body)
	}
	if _, err := os.Stat(filepath.Join(outsideDir, "pwned.txt")); !errors.Is(err, os.ErrNotExist) {
		t.Fatalf("expected outside file to remain absent, got err=%v", err)
	}
}

func requestJSON(t *testing.T, server *httptest.Server, method, path, token string, body any, into any) {
	t.Helper()

	var requestBody *bytes.Reader
	if body == nil {
		requestBody = bytes.NewReader(nil)
	} else {
		payload, err := json.Marshal(body)
		if err != nil {
			t.Fatalf("marshal body: %v", err)
		}
		requestBody = bytes.NewReader(payload)
	}

	req, err := http.NewRequest(method, server.URL+path, requestBody)
	if err != nil {
		t.Fatalf("new request: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}

	resp, err := server.Client().Do(req)
	if err != nil {
		t.Fatalf("do request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= http.StatusBadRequest {
		var failure map[string]any
		_ = json.NewDecoder(resp.Body).Decode(&failure)
		t.Fatalf("unexpected status %d for %s %s: %+v", resp.StatusCode, method, path, failure)
	}
	if into == nil || resp.StatusCode == http.StatusNoContent {
		return
	}
	if err := json.NewDecoder(resp.Body).Decode(into); err != nil {
		t.Fatalf("decode response: %v", err)
	}
}

func requestStatus(t *testing.T, server *httptest.Server, method, path, token string, body any) (int, string) {
	t.Helper()

	var requestBody *bytes.Reader
	if body == nil {
		requestBody = bytes.NewReader(nil)
	} else {
		payload, err := json.Marshal(body)
		if err != nil {
			t.Fatalf("marshal body: %v", err)
		}
		requestBody = bytes.NewReader(payload)
	}

	req, err := http.NewRequestWithContext(context.Background(), method, server.URL+path, requestBody)
	if err != nil {
		t.Fatalf("new request: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}

	resp, err := server.Client().Do(req)
	if err != nil {
		t.Fatalf("do request: %v", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read response: %v", err)
	}
	return resp.StatusCode, string(respBody)
}

func runGit(t *testing.T, dir string, args ...string) {
	t.Helper()

	cmd := exec.Command("git", args...)
	if dir != "" {
		cmd.Dir = dir
	}
	cmd.Env = append(os.Environ(), "GIT_TERMINAL_PROMPT=0")
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("git %v failed: %v (%s)", args, err, out)
	}
}
