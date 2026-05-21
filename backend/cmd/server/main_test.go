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
