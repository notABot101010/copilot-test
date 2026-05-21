package main

import (
	"bufio"
	"bytes"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
	"unicode/utf8"

	gliderssh "github.com/gliderlabs/ssh"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	gossh "golang.org/x/crypto/ssh"
)

type User struct {
	ID       int      `json:"id"`
	Username string   `json:"username"`
	Token    string   `json:"token,omitempty"`
	SSHKeys  []string `json:"sshKeys,omitempty"`
}

type Organization struct {
	ID      int    `json:"id"`
	Name    string `json:"name"`
	OwnerID int    `json:"ownerId"`
}

type Project struct {
	ID      int    `json:"id"`
	Name    string `json:"name"`
	OrgID   int    `json:"orgId"`
	RepoRel string `json:"repoPath"`
}

type IssueComment struct {
	ID        int       `json:"id"`
	AuthorID  int       `json:"authorId"`
	Body      string    `json:"body"`
	CreatedAt time.Time `json:"createdAt"`
}

type Issue struct {
	ID          int            `json:"id"`
	ProjectID   int            `json:"projectId"`
	Title       string         `json:"title"`
	Description string         `json:"description"`
	Status      string         `json:"status"`
	Tags        []string       `json:"tags"`
	Comments    []IssueComment `json:"comments"`
	CreatedAt   time.Time      `json:"createdAt"`
	UpdatedAt   time.Time      `json:"updatedAt"`
}

type RepoBranch struct {
	Name      string `json:"name"`
	IsDefault bool   `json:"isDefault"`
}

type RepoEntry struct {
	Name string `json:"name"`
	Path string `json:"path"`
	Type string `json:"type"`
}

type RepoFile struct {
	Branch  string `json:"branch"`
	Path    string `json:"path"`
	Content string `json:"content"`
}

type MergeRequestComment struct {
	ID        int       `json:"id"`
	AuthorID  int       `json:"authorId"`
	Body      string    `json:"body"`
	CreatedAt time.Time `json:"createdAt"`
}

type MergeRequest struct {
	ID           int                   `json:"id"`
	ProjectID    int                   `json:"projectId"`
	AuthorID     int                   `json:"authorId"`
	Title        string                `json:"title"`
	Description  string                `json:"description"`
	SourceBranch string                `json:"sourceBranch"`
	TargetBranch string                `json:"targetBranch"`
	Status       string                `json:"status"`
	Comments     []MergeRequestComment `json:"comments"`
	CreatedAt    time.Time             `json:"createdAt"`
	UpdatedAt    time.Time             `json:"updatedAt"`
}

type Store struct {
	mu            sync.RWMutex
	nextUserID    int
	nextOrgID     int
	nextProjectID int
	nextIssueID   int
	nextCommentID int
	nextMergeID   int
	nextMRComment int
	users         map[int]*User
	tokens        map[string]int
	orgs          map[int]*Organization
	projects      map[int]*Project
	issues        map[int]map[int]*Issue
	mergeRequests map[int]map[int]*MergeRequest
}

func newStore() *Store {
	return &Store{
		nextUserID:    1,
		nextOrgID:     1,
		nextProjectID: 1,
		nextIssueID:   1,
		nextCommentID: 1,
		nextMergeID:   1,
		nextMRComment: 1,
		users:         map[int]*User{},
		tokens:        map[string]int{},
		orgs:          map[int]*Organization{},
		projects:      map[int]*Project{},
		issues:        map[int]map[int]*Issue{},
		mergeRequests: map[int]map[int]*MergeRequest{},
	}
}

type app struct {
	store       *Store
	reposRoot   string
	staticRoot  string
	gitBinary   string
	httpBaseURL string
}

var repoNamePattern = regexp.MustCompile(`^[a-zA-Z0-9][a-zA-Z0-9._-]*$`)
var lfsOIDPattern = regexp.MustCompile(`^[0-9a-f]{64}$`)
var errBinaryFile = errors.New("binary file")
var errNoChanges = errors.New("no changes")

const gitCommandTimeout = 10 * time.Minute

// lfsObjectRef is a single object entry used in LFS batch requests and responses.
type lfsObjectRef struct {
	OID  string `json:"oid"`
	Size int64  `json:"size"`
}

type lfsBatchRequest struct {
	Operation string         `json:"operation"`
	Objects   []lfsObjectRef `json:"objects"`
}

type lfsAction struct {
	Href   string            `json:"href"`
	Header map[string]string `json:"header,omitempty"`
}

type lfsObjectError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type lfsObjectResponse struct {
	OID           string               `json:"oid"`
	Size          int64                `json:"size"`
	Authenticated bool                 `json:"authenticated,omitempty"`
	Actions       map[string]lfsAction `json:"actions,omitempty"`
	Error         *lfsObjectError      `json:"error,omitempty"`
}

type lfsBatchResponse struct {
	Transfer string              `json:"transfer"`
	Objects  []lfsObjectResponse `json:"objects"`
}

func main() {
	httpPort := envOrDefault("HTTP_PORT", "8080")
	sshPort := envOrDefault("SSH_PORT", "2222")
	reposRoot := envOrDefault("REPOS_ROOT", "./data/repos")
	staticRoot := envOrDefault("STATIC_ROOT", "./frontend/dist")
	httpBaseURL := envOrDefault("HTTP_BASE_URL", "http://localhost:"+httpPort)
	gitBinary, err := exec.LookPath("git")
	if err != nil {
		log.Fatalf("git binary not found: %v", err)
	}

	reposRoot, err = filepath.Abs(reposRoot)
	if err != nil {
		log.Fatalf("failed to resolve repos root: %v", err)
	}

	if err := os.MkdirAll(reposRoot, 0o755); err != nil {
		log.Fatalf("failed to create repos root: %v", err)
	}

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot, gitBinary: gitBinary, httpBaseURL: strings.TrimRight(httpBaseURL, "/")}
	httpServer := &http.Server{
		Addr:              ":" + httpPort,
		Handler:           a.httpRouter(),
		ReadHeaderTimeout: 30 * time.Second,
	}
	sshServer := a.sshServer(":" + sshPort)

	errCh := make(chan error, 2)
	go func() {
		log.Printf("HTTP listening on %s", httpServer.Addr)
		errCh <- httpServer.ListenAndServe()
	}()
	go func() {
		log.Printf("SSH listening on %s", sshServer.Addr)
		errCh <- sshServer.ListenAndServe()
	}()

	err = <-errCh
	log.Fatalf("server stopped: %v", err)
}

func (a *app) httpRouter() http.Handler {
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.Recoverer)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)

	r.Route("/api", func(api chi.Router) {
		api.Post("/users", a.createUser)
		api.With(a.requireBearerUser).Post("/users/{userID}/ssh-keys", a.addSSHKey)
		api.With(a.requireBearerUser).Post("/orgs", a.createOrganization)
		api.Get("/orgs", a.listOrganizations)
		api.Get("/projects", a.listProjects)
		api.With(a.requireBearerUser).Post("/orgs/{orgID}/projects", a.createProject)
		api.Get("/projects/{projectID}/repo/branches", a.listRepoBranches)
		api.Get("/projects/{projectID}/repo/tree", a.listRepoTree)
		api.Get("/projects/{projectID}/repo/file", a.getRepoFile)
		api.With(a.requireBearerUser).Put("/projects/{projectID}/repo/file", a.upsertRepoFile)
		api.With(a.requireBearerUser).Delete("/projects/{projectID}/repo/file", a.deleteRepoFile)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/issues", a.createIssue)
		api.Get("/projects/{projectID}/issues", a.listIssues)
		api.With(a.requireBearerUser).Patch("/projects/{projectID}/issues/{issueID}", a.updateIssue)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/issues/{issueID}/comments", a.addIssueComment)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/merge-requests", a.createMergeRequest)
		api.Get("/projects/{projectID}/merge-requests", a.listMergeRequests)
		api.Get("/projects/{projectID}/merge-requests/{mergeRequestID}", a.getMergeRequest)
		api.Get("/projects/{projectID}/merge-requests/{mergeRequestID}/diff", a.getMergeRequestDiff)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/merge-requests/{mergeRequestID}/comments", a.addMergeRequestComment)
	})

	r.HandleFunc("/git/*", a.handleGitHTTP)

	r.Get("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})

	if stat, err := os.Stat(a.staticRoot); err == nil && stat.IsDir() {
		fileServer := http.FileServer(http.Dir(a.staticRoot))
		r.Handle("/*", spaFallback(fileServer, a.staticRoot))
	} else {
		r.Get("/", func(w http.ResponseWriter, _ *http.Request) {
			_, _ = w.Write([]byte("Git hosting platform API is running"))
		})
	}

	return r
}

func spaFallback(next http.Handler, staticRoot string) http.HandlerFunc {
	indexFile := filepath.Join(staticRoot, "index.html")
	return func(w http.ResponseWriter, r *http.Request) {
		path := filepath.Clean(strings.TrimPrefix(r.URL.Path, "/"))
		if path == "." {
			next.ServeHTTP(w, r)
			return
		}
		if strings.Contains(filepath.Base(path), ".") {
			next.ServeHTTP(w, r)
			return
		}
		http.ServeFile(w, r, indexFile)
	}
}

func (a *app) sshServer(addr string) *gliderssh.Server {
	return &gliderssh.Server{
		Addr: addr,
		PublicKeyHandler: func(ctx gliderssh.Context, key gliderssh.PublicKey) bool {
			a.store.mu.RLock()
			defer a.store.mu.RUnlock()
			for _, u := range a.store.users {
				for _, saved := range u.SSHKeys {
					parsed, _, _, _, err := gossh.ParseAuthorizedKey([]byte(saved))
					if err != nil {
						continue
					}
					if string(parsed.Marshal()) == string(key.Marshal()) {
						ctx.SetValue("userID", u.ID)
						return true
					}
				}
			}
			return false
		},
		Handler: a.handleSSHSession,
	}
}

func (a *app) handleSSHSession(s gliderssh.Session) {
	cmd := s.Command()

	// git-lfs-authenticate takes three arguments: command, repo path, operation.
	if len(cmd) == 3 && cmd[0] == "git-lfs-authenticate" {
		a.handleLFSAuthenticate(s, cmd)
		return
	}

	if len(cmd) != 2 {
		_, _ = io.WriteString(s.Stderr(), "expected git command and repository path\n")
		s.Exit(1)
		return
	}

	gitCommand := cmd[0]
	if gitCommand != "git-receive-pack" && gitCommand != "git-upload-pack" {
		_, _ = io.WriteString(s.Stderr(), "only git-receive-pack and git-upload-pack are allowed\n")
		s.Exit(1)
		return
	}

	repoArg := strings.Trim(cmd[1], "'\"")
	project, err := a.findProjectByRepoArg(repoArg)
	if err != nil {
		_, _ = io.WriteString(s.Stderr(), "repository not found\n")
		s.Exit(1)
		return
	}
	userID, ok := sshUserIDFromSession(s)
	if !ok {
		_, _ = io.WriteString(s.Stderr(), "unauthorized\n")
		s.Exit(1)
		return
	}
	if gitCommand == "git-receive-pack" && !a.canUserWriteProject(userID, project) {
		_, _ = io.WriteString(s.Stderr(), "forbidden\n")
		s.Exit(1)
		return
	}
	repoPath, err := a.resolveRepoPath(project.RepoRel)
	if err != nil {
		_, _ = io.WriteString(s.Stderr(), "repository not found\n")
		s.Exit(1)
		return
	}
	ctx, cancel := context.WithTimeout(s.Context(), gitCommandTimeout)
	defer cancel()
	gitMode := "upload-pack"
	if gitCommand == "git-receive-pack" {
		gitMode = "receive-pack"
	}
	execCmd := exec.CommandContext(ctx, a.gitBinary, gitMode, repoPath)
	execCmd.Stdin = s
	execCmd.Stdout = s
	execCmd.Stderr = s.Stderr()
	execCmd.Env = append(os.Environ(), "GIT_PROTOCOL=version=2")

	if err := execCmd.Run(); err != nil {
		_, _ = io.WriteString(s.Stderr(), fmt.Sprintf("git command failed: %v\n", err))
		s.Exit(1)
		return
	}

	s.Exit(0)
}

func (a *app) createUser(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Username string `json:"username"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	req.Username = strings.TrimSpace(req.Username)
	if req.Username == "" {
		writeError(w, http.StatusBadRequest, "username is required")
		return
	}

	token, err := randomToken()
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to create token")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	id := a.store.nextUserID
	a.store.nextUserID++
	u := &User{ID: id, Username: req.Username, Token: token}
	a.store.users[id] = u
	a.store.tokens[token] = id
	writeJSON(w, http.StatusCreated, u)
}

func (a *app) addSSHKey(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	paramID, err := strconv.Atoi(chi.URLParam(r, "userID"))
	if err != nil || paramID != uid {
		writeError(w, http.StatusForbidden, "can only modify your own keys")
		return
	}

	var req struct {
		Key string `json:"key"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	key := strings.TrimSpace(req.Key)
	if key == "" {
		writeError(w, http.StatusBadRequest, "key is required")
		return
	}
	if _, _, _, _, err := gossh.ParseAuthorizedKey([]byte(key)); err != nil {
		writeError(w, http.StatusBadRequest, "invalid ssh public key")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	u, ok := a.store.users[uid]
	if !ok {
		writeError(w, http.StatusUnauthorized, "user not found")
		return
	}
	u.SSHKeys = append(u.SSHKeys, key)
	writeJSON(w, http.StatusCreated, map[string]any{"sshKeys": u.SSHKeys})
}

func (a *app) createOrganization(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	var req struct {
		Name string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	name := strings.TrimSpace(req.Name)
	if name == "" {
		writeError(w, http.StatusBadRequest, "name is required")
		return
	}
	if !isValidRepoSegment(name) {
		writeError(w, http.StatusBadRequest, "invalid organization name")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	id := a.store.nextOrgID
	a.store.nextOrgID++
	org := &Organization{ID: id, Name: name, OwnerID: uid}
	a.store.orgs[id] = org
	writeJSON(w, http.StatusCreated, org)
}

func (a *app) listOrganizations(w http.ResponseWriter, _ *http.Request) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	orgs := make([]*Organization, 0, len(a.store.orgs))
	for _, org := range a.store.orgs {
		orgs = append(orgs, org)
	}
	writeJSON(w, http.StatusOK, orgs)
}

func (a *app) createProject(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	orgID, err := strconv.Atoi(chi.URLParam(r, "orgID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid org id")
		return
	}
	var req struct {
		Name string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	name := strings.TrimSpace(req.Name)
	if name == "" {
		writeError(w, http.StatusBadRequest, "name is required")
		return
	}
	if !isValidRepoSegment(name) {
		writeError(w, http.StatusBadRequest, "invalid project name")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	org, ok := a.store.orgs[orgID]
	if !ok {
		writeError(w, http.StatusNotFound, "organization not found")
		return
	}
	if org.OwnerID != uid {
		writeError(w, http.StatusForbidden, "only organization owner can create project")
		return
	}

	id := a.store.nextProjectID
	a.store.nextProjectID++
	repoRel := filepath.Join(org.Name, name+".git")
	repoPath, err := a.resolveRepoPath(repoRel)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid repository path")
		return
	}
	if err := os.MkdirAll(filepath.Dir(repoPath), 0o755); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to prepare repository directory")
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	if err := exec.CommandContext(ctx, a.gitBinary, "init", "--bare", repoPath).Run(); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to initialize repository")
		return
	}

	p := &Project{ID: id, Name: name, OrgID: orgID, RepoRel: repoRel}
	a.store.projects[id] = p
	a.store.issues[id] = map[int]*Issue{}
	a.store.mergeRequests[id] = map[int]*MergeRequest{}
	writeJSON(w, http.StatusCreated, p)
}

func (a *app) listProjects(w http.ResponseWriter, _ *http.Request) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	projects := make([]*Project, 0, len(a.store.projects))
	for _, p := range a.store.projects {
		projects = append(projects, p)
	}
	writeJSON(w, http.StatusOK, projects)
}

func (a *app) createIssue(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	var req struct {
		Title       string `json:"title"`
		Description string `json:"description"`
		Tags        []string `json:"tags"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	if strings.TrimSpace(req.Title) == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	if _, ok := a.store.projects[projectID]; !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	now := time.Now().UTC()
	id := a.store.nextIssueID
	a.store.nextIssueID++
	issue := &Issue{
		ID:          id,
		ProjectID:   projectID,
		Title:       strings.TrimSpace(req.Title),
		Description: strings.TrimSpace(req.Description),
		Status:      "open",
		Tags:        normalizeIssueTags(req.Tags),
		Comments:    []IssueComment{},
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	a.store.issues[projectID][id] = issue
	writeJSON(w, http.StatusCreated, issue)
}

func (a *app) listIssues(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	projectIssues, ok := a.store.issues[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	list := make([]*Issue, 0, len(projectIssues))
	for _, issue := range projectIssues {
		list = append(list, issue)
	}
	writeJSON(w, http.StatusOK, list)
}

func (a *app) updateIssue(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	issueID, err := strconv.Atoi(chi.URLParam(r, "issueID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid issue id")
		return
	}

	var req struct {
		Title       *string `json:"title"`
		Description *string `json:"description"`
		Status      *string `json:"status"`
		Tags        *[]string `json:"tags"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	projectIssues, ok := a.store.issues[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	issue, ok := projectIssues[issueID]
	if !ok {
		writeError(w, http.StatusNotFound, "issue not found")
		return
	}
	if req.Title != nil {
		t := strings.TrimSpace(*req.Title)
		if t == "" {
			writeError(w, http.StatusBadRequest, "title cannot be empty")
			return
		}
		issue.Title = t
	}
	if req.Description != nil {
		issue.Description = strings.TrimSpace(*req.Description)
	}
	if req.Status != nil {
		status := strings.TrimSpace(strings.ToLower(*req.Status))
		if status != "open" && status != "closed" {
			writeError(w, http.StatusBadRequest, "status must be open or closed")
			return
		}
		issue.Status = status
	}
	if req.Tags != nil {
		issue.Tags = normalizeIssueTags(*req.Tags)
	}
	issue.UpdatedAt = time.Now().UTC()
	writeJSON(w, http.StatusOK, issue)
}

func normalizeIssueTags(tags []string) []string {
	if len(tags) == 0 {
		return []string{}
	}
	normalized := make([]string, 0, len(tags))
	seen := make(map[string]struct{}, len(tags))
	for _, tag := range tags {
		trimmed := strings.TrimSpace(tag)
		if trimmed == "" {
			continue
		}
		if _, ok := seen[trimmed]; ok {
			continue
		}
		seen[trimmed] = struct{}{}
		normalized = append(normalized, trimmed)
	}
	return normalized
}

func (a *app) addIssueComment(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	issueID, err := strconv.Atoi(chi.URLParam(r, "issueID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid issue id")
		return
	}
	var req struct {
		Body string `json:"body"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	if strings.TrimSpace(req.Body) == "" {
		writeError(w, http.StatusBadRequest, "body is required")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	projectIssues, ok := a.store.issues[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	issue, ok := projectIssues[issueID]
	if !ok {
		writeError(w, http.StatusNotFound, "issue not found")
		return
	}

	id := a.store.nextCommentID
	a.store.nextCommentID++
	comment := IssueComment{ID: id, AuthorID: uid, Body: strings.TrimSpace(req.Body), CreatedAt: time.Now().UTC()}
	issue.Comments = append(issue.Comments, comment)
	issue.UpdatedAt = time.Now().UTC()
	writeJSON(w, http.StatusCreated, comment)
}

func (a *app) listRepoBranches(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branches, err := a.repoBranches(ctx, repoPath)
	if err != nil {
		log.Printf("list repo branches failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to list branches")
		return
	}
	writeJSON(w, http.StatusOK, branches)
}

func (a *app) listRepoTree(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	repoPathValue, err := normalizeRepoFilePath(r.URL.Query().Get("path"), true)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid path")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}
	entries, err := a.repoTree(ctx, repoPath, branch, repoPathValue)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "path not found")
			return
		}
		log.Printf("list repo tree failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to read repository")
		return
	}
	writeJSON(w, http.StatusOK, entries)
}

func (a *app) getRepoFile(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	filePath, err := normalizeRepoFilePath(r.URL.Query().Get("path"), false)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid path")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}
	content, err := a.repoFileContent(ctx, repoPath, branch, filePath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "file not found")
			return
		}
		if errors.Is(err, errBinaryFile) {
			writeError(w, http.StatusBadRequest, "binary files are not supported")
			return
		}
		log.Printf("get repo file failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to read file")
		return
	}
	writeJSON(w, http.StatusOK, RepoFile{Branch: branch, Path: filePath, Content: content})
}

func (a *app) upsertRepoFile(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	var req struct {
		Branch  string `json:"branch"`
		Path    string `json:"path"`
		Content string `json:"content"`
		Message string `json:"message"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}

	filePath, err := normalizeRepoFilePath(req.Path, false)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid path")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	if !a.canUserWriteProject(uid, project) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branch, err := a.resolveRequestedBranch(ctx, repoPath, req.Branch)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}

	if err := a.commitRepoFile(ctx, project, uid, branch, filePath, req.Content, req.Message); err != nil {
		if errors.Is(err, errNoChanges) {
			writeError(w, http.StatusBadRequest, "no changes to commit")
			return
		}
		log.Printf("save repo file failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to save file")
		return
	}
	writeJSON(w, http.StatusOK, RepoFile{Branch: branch, Path: filePath, Content: req.Content})
}

func (a *app) deleteRepoFile(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	var req struct {
		Branch  string `json:"branch"`
		Path    string `json:"path"`
		Message string `json:"message"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}

	filePath, err := normalizeRepoFilePath(req.Path, false)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid path")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	if !a.canUserWriteProject(uid, project) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branch, err := a.resolveRequestedBranch(ctx, repoPath, req.Branch)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}

	if err := a.removeRepoFile(ctx, project, uid, branch, filePath, req.Message); err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "file not found")
			return
		}
		if errors.Is(err, errNoChanges) {
			writeError(w, http.StatusBadRequest, "no changes to commit")
			return
		}
		log.Printf("delete repo file failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to delete file")
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (a *app) createMergeRequest(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	var req struct {
		Title        string `json:"title"`
		Description  string `json:"description"`
		SourceBranch string `json:"sourceBranch"`
		TargetBranch string `json:"targetBranch"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}

	title := strings.TrimSpace(req.Title)
	if title == "" {
		writeError(w, http.StatusBadRequest, "title is required")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	sourceBranch, err := a.resolveRequestedBranch(ctx, repoPath, req.SourceBranch)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid source branch")
		return
	}
	targetBranch, err := a.resolveRequestedBranch(ctx, repoPath, req.TargetBranch)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid target branch")
		return
	}
	if sourceBranch == targetBranch {
		writeError(w, http.StatusBadRequest, "source and target branches must differ")
		return
	}
	if !a.branchExists(ctx, repoPath, sourceBranch) || !a.branchExists(ctx, repoPath, targetBranch) {
		writeError(w, http.StatusBadRequest, "both branches must exist")
		return
	}

	diff, err := a.repoDiff(ctx, repoPath, sourceBranch, targetBranch)
	if err != nil {
		log.Printf("create merge request diff failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to compare branches")
		return
	}
	if strings.TrimSpace(diff) == "" {
		writeError(w, http.StatusBadRequest, "branches have no differences")
		return
	}

	now := time.Now().UTC()
	mr := &MergeRequest{
		ProjectID:    projectID,
		AuthorID:     uid,
		Title:        title,
		Description:  strings.TrimSpace(req.Description),
		SourceBranch: sourceBranch,
		TargetBranch: targetBranch,
		Status:       "open",
		Comments:     []MergeRequestComment{},
		CreatedAt:    now,
		UpdatedAt:    now,
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	if _, ok := a.store.projects[projectID]; !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	mr.ID = a.store.nextMergeID
	a.store.nextMergeID++
	if _, ok := a.store.mergeRequests[projectID]; !ok {
		a.store.mergeRequests[projectID] = map[int]*MergeRequest{}
	}
	a.store.mergeRequests[projectID][mr.ID] = mr
	writeJSON(w, http.StatusCreated, mr)
}

func (a *app) listMergeRequests(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	if _, ok := a.store.projects[projectID]; !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	projectMRs := a.store.mergeRequests[projectID]
	list := make([]*MergeRequest, 0, len(projectMRs))
	for _, mr := range projectMRs {
		list = append(list, mr)
	}
	sort.Slice(list, func(i, j int) bool { return list[i].ID < list[j].ID })
	writeJSON(w, http.StatusOK, list)
}

func (a *app) getMergeRequest(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	mergeRequestID, err := strconv.Atoi(chi.URLParam(r, "mergeRequestID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid merge request id")
		return
	}

	mr, err := a.lookupMergeRequest(projectID, mergeRequestID)
	if err != nil {
		writeError(w, http.StatusNotFound, "merge request not found")
		return
	}
	writeJSON(w, http.StatusOK, mr)
}

func (a *app) getMergeRequestDiff(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	mergeRequestID, err := strconv.Atoi(chi.URLParam(r, "mergeRequestID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid merge request id")
		return
	}

	mr, err := a.lookupMergeRequest(projectID, mergeRequestID)
	if err != nil {
		writeError(w, http.StatusNotFound, "merge request not found")
		return
	}

	_, repoPath, err := a.projectRepoPath(projectID)
	if err != nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	diff, err := a.repoDiff(ctx, repoPath, mr.SourceBranch, mr.TargetBranch)
	if err != nil {
		log.Printf("get merge request diff failed: %v", err)
		writeError(w, http.StatusInternalServerError, "failed to load diff")
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"diff": diff})
}

func (a *app) addMergeRequestComment(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	mergeRequestID, err := strconv.Atoi(chi.URLParam(r, "mergeRequestID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid merge request id")
		return
	}

	var req struct {
		Body string `json:"body"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	body := strings.TrimSpace(req.Body)
	if body == "" {
		writeError(w, http.StatusBadRequest, "body is required")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	projectMRs, ok := a.store.mergeRequests[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	mr, ok := projectMRs[mergeRequestID]
	if !ok {
		writeError(w, http.StatusNotFound, "merge request not found")
		return
	}

	comment := MergeRequestComment{
		ID:        a.store.nextMRComment,
		AuthorID:  uid,
		Body:      body,
		CreatedAt: time.Now().UTC(),
	}
	a.store.nextMRComment++
	mr.Comments = append(mr.Comments, comment)
	mr.UpdatedAt = time.Now().UTC()
	writeJSON(w, http.StatusCreated, comment)
}

func (a *app) handleGitHTTP(w http.ResponseWriter, r *http.Request) {
	pathInfo := strings.TrimPrefix(r.URL.Path, "/git")
	if pathInfo == "" || pathInfo == "/" {
		writeError(w, http.StatusNotFound, "repository path required")
		return
	}

	// Route LFS API requests before the standard git method check (LFS uses PUT for uploads).
	if a.routeLFSRequest(w, r, pathInfo) {
		return
	}

	if r.Method != http.MethodGet && r.Method != http.MethodPost {
		w.Header().Set("Allow", "GET, POST")
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	project, err := a.findProjectByRepoArg(pathInfo)
	if err != nil {
		writeError(w, http.StatusNotFound, "repository not found")
		return
	}

	var gitUser *User
	if a.isReceivePackRequest(r) {
		username, token, ok := r.BasicAuth()
		if !ok {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			writeError(w, http.StatusUnauthorized, "basic auth required for push")
			return
		}
		gitUser, err = a.authenticateHTTPGit(username, token)
		if err != nil {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			writeError(w, http.StatusUnauthorized, "invalid credentials")
			return
		}
		if !a.canUserWriteProject(gitUser.ID, project) {
			writeError(w, http.StatusForbidden, "forbidden")
			return
		}
	}

	cmdPathInfo := "/" + filepath.ToSlash(project.RepoRel) + strings.TrimPrefix(pathInfo, "/"+project.RepoRel)
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, a.gitBinary, "http-backend")
	cmd.Env = append(os.Environ(),
		"GIT_PROJECT_ROOT="+a.reposRoot,
		"GIT_HTTP_EXPORT_ALL=1",
		"PATH_INFO="+cmdPathInfo,
		"QUERY_STRING="+r.URL.RawQuery,
		"REQUEST_METHOD="+r.Method,
		"CONTENT_TYPE="+r.Header.Get("Content-Type"),
		"REMOTE_ADDR="+r.RemoteAddr,
	)
	if r.ContentLength >= 0 {
		cmd.Env = append(cmd.Env, "CONTENT_LENGTH="+strconv.FormatInt(r.ContentLength, 10))
	}
	cmd.Stdin = r.Body
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		writeError(w, http.StatusInternalServerError, "git backend failure")
		return
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		writeError(w, http.StatusInternalServerError, "git backend failure")
		return
	}
	if err := cmd.Start(); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to start git backend")
		return
	}

	if err := writeCGIResponse(w, stdout); err != nil {
		log.Printf("failed to proxy git response: %v", err)
	}
	if err := cmd.Wait(); err != nil {
		errOut, _ := io.ReadAll(stderr)
		log.Printf("git http-backend error: %v (%s)", err, strings.TrimSpace(string(errOut)))
	}
}

func writeCGIResponse(w http.ResponseWriter, in io.Reader) error {
	reader := bufio.NewReader(in)
	status := http.StatusOK
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			if errors.Is(err, io.EOF) {
				break
			}
			return err
		}
		line = strings.TrimRight(line, "\r\n")
		if line == "" {
			break
		}
		parts := strings.SplitN(line, ":", 2)
		if len(parts) != 2 {
			continue
		}
		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])
		if strings.EqualFold(key, "Status") {
			chunks := strings.SplitN(value, " ", 2)
			if code, err := strconv.Atoi(chunks[0]); err == nil {
				status = code
			}
			continue
		}
		w.Header().Add(key, value)
	}
	w.WriteHeader(status)
	_, err := io.Copy(w, reader)
	return err
}

func (a *app) isReceivePackRequest(r *http.Request) bool {
	service := r.URL.Query().Get("service")
	path := strings.TrimSuffix(r.URL.Path, "/")
	return service == "git-receive-pack" || strings.HasSuffix(path, "/git-receive-pack")
}

// ---------------------------------------------------------------------------
// Git LFS support
// ---------------------------------------------------------------------------

// lfsObjectPath returns the filesystem path for an LFS object stored under a repository.
func (a *app) lfsObjectPath(repoPath, oid string) string {
	return filepath.Join(repoPath, "lfs", "objects", oid[:2], oid[2:4], oid)
}

// lfsBaseURL returns the base HTTP URL to use in LFS response hrefs.
// It prefers the configured httpBaseURL but falls back to constructing it from
// the request Host header, which works for simple single-server deployments.
func (a *app) lfsBaseURL(r *http.Request) string {
	if a.httpBaseURL != "" {
		return a.httpBaseURL
	}
	scheme := "http"
	if proto := r.Header.Get("X-Forwarded-Proto"); proto == "https" {
		scheme = "https"
	}
	return scheme + "://" + r.Host
}

// routeLFSRequest inspects pathInfo (the URL path with the "/git" prefix stripped)
// and, if it matches an LFS endpoint, handles the request and returns true.
// Returns false when the request is not an LFS request.
func (a *app) routeLFSRequest(w http.ResponseWriter, r *http.Request, pathInfo string) bool {
	const batchSuffix = "/info/lfs/objects/batch"
	const objectsInfix = "/info/lfs/objects/"

	// Batch API: POST /<org>/<project>.git/info/lfs/objects/batch
	if strings.HasSuffix(pathInfo, batchSuffix) && r.Method == http.MethodPost {
		repoArg := strings.TrimSuffix(pathInfo, batchSuffix)
		project, err := a.findProjectByRepoArg(repoArg)
		if err != nil {
			writeLFSError(w, http.StatusNotFound, "repository not found")
			return true
		}
		repoPath, err := a.resolveRepoPath(project.RepoRel)
		if err != nil {
			writeLFSError(w, http.StatusNotFound, "repository not found")
			return true
		}
		a.handleLFSBatch(w, r, project, repoPath)
		return true
	}

	// Object upload/download: GET or PUT /<org>/<project>.git/info/lfs/objects/<oid>
	if idx := strings.Index(pathInfo, objectsInfix); idx != -1 {
		oid := pathInfo[idx+len(objectsInfix):]
		if lfsOIDPattern.MatchString(oid) {
			repoArg := pathInfo[:idx]
			project, err := a.findProjectByRepoArg(repoArg)
			if err != nil {
				writeLFSError(w, http.StatusNotFound, "repository not found")
				return true
			}
			repoPath, err := a.resolveRepoPath(project.RepoRel)
			if err != nil {
				writeLFSError(w, http.StatusNotFound, "repository not found")
				return true
			}
			switch r.Method {
			case http.MethodGet:
				a.handleLFSDownload(w, r, repoPath, oid)
			case http.MethodPut:
				a.handleLFSUpload(w, r, project, repoPath, oid)
			default:
				w.Header().Set("Allow", "GET, PUT")
				writeLFSError(w, http.StatusMethodNotAllowed, "method not allowed")
			}
			return true
		}
	}

	return false
}

func (a *app) handleLFSBatch(w http.ResponseWriter, r *http.Request, project *Project, repoPath string) {
	w.Header().Set("Content-Type", "application/vnd.git-lfs+json")

	var req lfsBatchRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		w.WriteHeader(http.StatusBadRequest)
		_ = json.NewEncoder(w).Encode(map[string]string{"message": "invalid request body"})
		return
	}

	if req.Operation != "upload" && req.Operation != "download" {
		w.WriteHeader(http.StatusBadRequest)
		_ = json.NewEncoder(w).Encode(map[string]string{"message": "operation must be upload or download"})
		return
	}

	// Uploads require write authorization.
	if req.Operation == "upload" {
		username, token, ok := r.BasicAuth()
		if !ok {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			w.WriteHeader(http.StatusUnauthorized)
			_ = json.NewEncoder(w).Encode(map[string]string{"message": "credentials required for upload"})
			return
		}
		gitUser, err := a.authenticateHTTPGit(username, token)
		if err != nil {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			w.WriteHeader(http.StatusUnauthorized)
			_ = json.NewEncoder(w).Encode(map[string]string{"message": "invalid credentials"})
			return
		}
		if !a.canUserWriteProject(gitUser.ID, project) {
			w.WriteHeader(http.StatusForbidden)
			_ = json.NewEncoder(w).Encode(map[string]string{"message": "forbidden"})
			return
		}
	}

	baseURL := a.lfsBaseURL(r)
	repoURLPath := "/git/" + filepath.ToSlash(project.RepoRel)

	objects := make([]lfsObjectResponse, 0, len(req.Objects))
	for _, obj := range req.Objects {
		if !lfsOIDPattern.MatchString(obj.OID) {
			objects = append(objects, lfsObjectResponse{
				OID:   obj.OID,
				Size:  obj.Size,
				Error: &lfsObjectError{Code: http.StatusUnprocessableEntity, Message: "invalid OID"},
			})
			continue
		}

		objPath := a.lfsObjectPath(repoPath, obj.OID)
		objectURL := baseURL + repoURLPath + "/info/lfs/objects/" + obj.OID

		objResp := lfsObjectResponse{OID: obj.OID, Size: obj.Size}
		if req.Operation == "upload" {
			if _, err := os.Stat(objPath); os.IsNotExist(err) {
				// Object not yet stored; provide an upload action.
				objResp.Authenticated = true
				objResp.Actions = map[string]lfsAction{
					"upload": {
						Href:   objectURL,
						Header: map[string]string{"Content-Type": "application/octet-stream"},
					},
				}
			}
			// If the object already exists no action is needed.
		} else {
			if _, err := os.Stat(objPath); os.IsNotExist(err) {
				objResp.Error = &lfsObjectError{Code: http.StatusNotFound, Message: "object not found"}
			} else {
				objResp.Actions = map[string]lfsAction{
					"download": {Href: objectURL},
				}
			}
		}
		objects = append(objects, objResp)
	}

	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(lfsBatchResponse{Transfer: "basic", Objects: objects})
}

func (a *app) handleLFSDownload(w http.ResponseWriter, _ *http.Request, repoPath, oid string) {
	objPath := a.lfsObjectPath(repoPath, oid)
	f, err := os.Open(objPath)
	if err != nil {
		if os.IsNotExist(err) {
			w.Header().Set("Content-Type", "application/vnd.git-lfs+json")
			w.WriteHeader(http.StatusNotFound)
			_ = json.NewEncoder(w).Encode(map[string]string{"message": "object not found"})
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to read object")
		return
	}
	defer f.Close()

	info, err := f.Stat()
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to stat object")
		return
	}

	w.Header().Set("Content-Type", "application/octet-stream")
	w.Header().Set("Content-Length", strconv.FormatInt(info.Size(), 10))
	w.WriteHeader(http.StatusOK)
	_, _ = io.Copy(w, f)
}

func (a *app) handleLFSUpload(w http.ResponseWriter, r *http.Request, project *Project, repoPath, oid string) {
	username, token, ok := r.BasicAuth()
	if !ok {
		w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
		writeLFSError(w, http.StatusUnauthorized, "credentials required")
		return
	}
	gitUser, err := a.authenticateHTTPGit(username, token)
	if err != nil {
		w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
		writeLFSError(w, http.StatusUnauthorized, "invalid credentials")
		return
	}
	if !a.canUserWriteProject(gitUser.ID, project) {
		writeLFSError(w, http.StatusForbidden, "forbidden")
		return
	}

	objPath := a.lfsObjectPath(repoPath, oid)

	// Object already present; nothing to do.
	if _, err := os.Stat(objPath); err == nil {
		w.WriteHeader(http.StatusOK)
		return
	}

	if err := os.MkdirAll(filepath.Dir(objPath), 0o755); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to prepare object storage")
		return
	}

	// Stream into a temp file, verify SHA-256, then rename atomically.
	tmp, err := os.CreateTemp(filepath.Dir(objPath), "lfs-upload-*")
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to create temp file")
		return
	}
	tmpPath := tmp.Name()
	committed := false
	closed := false
	defer func() {
		if !closed {
			tmp.Close()
		}
		if !committed {
			os.Remove(tmpPath)
		}
	}()

	h := sha256.New()
	if _, err := io.Copy(io.MultiWriter(tmp, h), r.Body); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to write object")
		return
	}
	if err := tmp.Close(); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to finalize object")
		return
	}
	closed = true

	computed := hex.EncodeToString(h.Sum(nil))
	if computed != oid {
		writeLFSError(w, http.StatusUnprocessableEntity, "OID mismatch: content does not match expected SHA-256")
		return
	}

	if err := os.Rename(tmpPath, objPath); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to store object")
		return
	}
	committed = true

	w.WriteHeader(http.StatusOK)
}

// handleLFSAuthenticate handles the SSH "git-lfs-authenticate <repo> upload|download" command.
// It returns a JSON payload that git-lfs uses to authenticate subsequent HTTP LFS requests.
func (a *app) handleLFSAuthenticate(s gliderssh.Session, cmd []string) {
	// cmd[0] = "git-lfs-authenticate", cmd[1] = repo path, cmd[2] = operation
	repoArg := strings.Trim(cmd[1], "'\"")
	operation := cmd[2]
	if operation != "upload" && operation != "download" {
		_, _ = io.WriteString(s.Stderr(), "operation must be upload or download\n")
		s.Exit(1)
		return
	}

	project, err := a.findProjectByRepoArg(repoArg)
	if err != nil {
		_, _ = io.WriteString(s.Stderr(), "repository not found\n")
		s.Exit(1)
		return
	}

	userID, ok := sshUserIDFromSession(s)
	if !ok {
		_, _ = io.WriteString(s.Stderr(), "unauthorized\n")
		s.Exit(1)
		return
	}

	if operation == "upload" && !a.canUserWriteProject(userID, project) {
		_, _ = io.WriteString(s.Stderr(), "forbidden\n")
		s.Exit(1)
		return
	}

	a.store.mu.RLock()
	u, ok := a.store.users[userID]
	a.store.mu.RUnlock()
	if !ok {
		_, _ = io.WriteString(s.Stderr(), "user not found\n")
		s.Exit(1)
		return
	}

	lfsHref := a.httpBaseURL + "/git/" + filepath.ToSlash(project.RepoRel) + "/info/lfs"
	creds := base64.StdEncoding.EncodeToString([]byte(u.Username + ":" + u.Token))

	type lfsAuthResponse struct {
		Href      string            `json:"href"`
		Header    map[string]string `json:"header"`
		ExpiresIn int               `json:"expires_in"`
	}
	resp := lfsAuthResponse{
		Href:      lfsHref,
		Header:    map[string]string{"Authorization": "Basic " + creds},
		ExpiresIn: 24 * 60 * 60, // 24 hours in seconds
	}
	if err := json.NewEncoder(s).Encode(resp); err != nil {
		_, _ = io.WriteString(s.Stderr(), "failed to write response\n")
		s.Exit(1)
		return
	}
	s.Exit(0)
}

// writeLFSError writes a JSON error response with the application/vnd.git-lfs+json content type.
func writeLFSError(w http.ResponseWriter, status int, message string) {
	w.Header().Set("Content-Type", "application/vnd.git-lfs+json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"message": message})
}

func (a *app) authenticateHTTPGit(username, token string) (*User, error) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	userID, ok := a.store.tokens[token]
	if !ok {
		return nil, errors.New("invalid credentials")
	}
	u, ok := a.store.users[userID]
	if ok && u.Username == username {
		return u, nil
	}
	return nil, errors.New("invalid credentials")
}

func (a *app) findProjectByRepoArg(repoArg string) (*Project, error) {
	clean := strings.TrimPrefix(strings.TrimSpace(repoArg), "/")
	clean = strings.Trim(clean, "'\"")
	if strings.Contains(clean, "..") {
		return nil, errors.New("invalid path")
	}
	parts := strings.Split(clean, "/")
	if len(parts) < 2 {
		return nil, errors.New("invalid repository path")
	}
	org := strings.TrimSpace(parts[0])
	project := strings.TrimSpace(parts[1])
	if !isValidRepoSegment(org) || !isValidRepoGitName(project) {
		return nil, errors.New("invalid repository path")
	}
	repo := filepath.Join(org, project)
	if strings.HasPrefix(filepath.Clean(repo), "..") {
		return nil, errors.New("invalid repository path")
	}

	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	for _, p := range a.store.projects {
		if filepath.Clean(p.RepoRel) == filepath.Clean(repo) {
			return p, nil
		}
	}
	return nil, errors.New("project not found")
}

func isValidRepoSegment(v string) bool {
	return repoNamePattern.MatchString(v)
}

func isValidRepoGitName(v string) bool {
	if !strings.HasSuffix(v, ".git") {
		return false
	}
	return isValidRepoSegment(strings.TrimSuffix(v, ".git"))
}

func sshUserIDFromSession(s gliderssh.Session) (int, bool) {
	userID, ok := s.Context().Value("userID").(int)
	return userID, ok
}

func (a *app) canUserWriteProject(userID int, project *Project) bool {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	org, ok := a.store.orgs[project.OrgID]
	if !ok {
		return false
	}
	return org.OwnerID == userID
}

func (a *app) projectRepoPath(projectID int) (*Project, string, error) {
	a.store.mu.RLock()
	project, ok := a.store.projects[projectID]
	a.store.mu.RUnlock()
	if !ok {
		return nil, "", os.ErrNotExist
	}
	repoPath, err := a.resolveRepoPath(project.RepoRel)
	if err != nil {
		return nil, "", err
	}
	return project, repoPath, nil
}

func (a *app) lookupMergeRequest(projectID, mergeRequestID int) (*MergeRequest, error) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	projectMRs, ok := a.store.mergeRequests[projectID]
	if !ok {
		return nil, os.ErrNotExist
	}
	mr, ok := projectMRs[mergeRequestID]
	if !ok {
		return nil, os.ErrNotExist
	}
	return mr, nil
}

func (a *app) runGit(ctx context.Context, args ...string) ([]byte, error) {
	cmd := exec.CommandContext(ctx, a.gitBinary, args...)
	cmd.Env = append(os.Environ(), "GIT_TERMINAL_PROMPT=0")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("git %s: %w (%s)", strings.Join(args, " "), err, strings.TrimSpace(string(out)))
	}
	return out, nil
}

func (a *app) gitBareOutput(ctx context.Context, repoPath string, args ...string) ([]byte, error) {
	return a.runGit(ctx, append([]string{"--git-dir", repoPath}, args...)...)
}

func (a *app) gitWorktreeOutput(ctx context.Context, dir string, args ...string) ([]byte, error) {
	return a.runGit(ctx, append([]string{"-C", dir}, args...)...)
}

func (a *app) repoBranches(ctx context.Context, repoPath string) ([]RepoBranch, error) {
	out, err := a.gitBareOutput(ctx, repoPath, "for-each-ref", "--format=%(refname:short)", "refs/heads")
	if err != nil {
		return nil, err
	}
	names := make([]string, 0)
	for _, line := range strings.Split(strings.TrimSpace(string(out)), "\n") {
		line = strings.TrimSpace(line)
		if line != "" {
			names = append(names, line)
		}
	}
	sort.Strings(names)
	defaultBranch := a.defaultBranchName(names)
	if len(names) == 0 {
		return []RepoBranch{{Name: defaultBranch, IsDefault: true}}, nil
	}
	branches := make([]RepoBranch, 0, len(names))
	for _, name := range names {
		branches = append(branches, RepoBranch{Name: name, IsDefault: name == defaultBranch})
	}
	return branches, nil
}

func (a *app) defaultBranchName(branches []string) string {
	for _, candidate := range []string{"main", "master"} {
		for _, branch := range branches {
			if branch == candidate {
				return branch
			}
		}
	}
	if len(branches) > 0 {
		return branches[0]
	}
	return "main"
}

func (a *app) resolveRequestedBranch(ctx context.Context, repoPath, raw string) (string, error) {
	branch := strings.TrimSpace(raw)
	if branch == "" {
		branches, err := a.repoBranches(ctx, repoPath)
		if err != nil {
			return "", err
		}
		for _, item := range branches {
			if item.IsDefault {
				return item.Name, nil
			}
		}
		return "", errors.New("default branch not found")
	}
	if _, err := a.runGit(ctx, "check-ref-format", "--branch", branch); err != nil {
		return "", err
	}
	return branch, nil
}

func (a *app) branchExists(ctx context.Context, repoPath, branch string) bool {
	_, err := a.gitBareOutput(ctx, repoPath, "rev-parse", "--verify", "--quiet", "refs/heads/"+branch)
	return err == nil
}

func normalizeRepoFilePath(raw string, allowRoot bool) (string, error) {
	value := strings.TrimSpace(strings.ReplaceAll(raw, "\\", "/"))
	if value == "" {
		if allowRoot {
			return "", nil
		}
		return "", errors.New("path required")
	}
	if strings.HasPrefix(value, "/") {
		return "", errors.New("invalid path")
	}
	clean := path.Clean(value)
	if clean == "." {
		if allowRoot {
			return "", nil
		}
		return "", errors.New("path required")
	}
	if clean == ".." || strings.HasPrefix(clean, "../") {
		return "", errors.New("invalid path")
	}
	return clean, nil
}

func (a *app) repoObjectType(ctx context.Context, repoPath, spec string) (string, error) {
	out, err := a.gitBareOutput(ctx, repoPath, "cat-file", "-t", spec)
	if err != nil {
		return "", os.ErrNotExist
	}
	return strings.TrimSpace(string(out)), nil
}

func (a *app) repoTree(ctx context.Context, repoPath, branch, repoPathValue string) ([]RepoEntry, error) {
	if !a.branchExists(ctx, repoPath, branch) {
		if repoPathValue == "" {
			return []RepoEntry{}, nil
		}
		return nil, os.ErrNotExist
	}
	ref := "refs/heads/" + branch
	target := ref
	if repoPathValue != "" {
		objType, err := a.repoObjectType(ctx, repoPath, ref+":"+repoPathValue)
		if err != nil || objType != "tree" {
			return nil, os.ErrNotExist
		}
		target = ref + ":" + repoPathValue
	}
	out, err := a.gitBareOutput(ctx, repoPath, "ls-tree", "-z", target)
	if err != nil {
		return nil, err
	}
	items := bytes.Split(out, []byte{0})
	entries := make([]RepoEntry, 0, len(items))
	for _, item := range items {
		if len(item) == 0 {
			continue
		}
		parts := bytes.SplitN(item, []byte{'\t'}, 2)
		if len(parts) != 2 {
			continue
		}
		meta := strings.Fields(string(parts[0]))
		if len(meta) < 3 {
			continue
		}
		name := string(parts[1])
		entryType := "file"
		if meta[1] == "tree" {
			entryType = "dir"
		}
		entryPath := name
		if repoPathValue != "" {
			entryPath = path.Join(repoPathValue, name)
		}
		entries = append(entries, RepoEntry{Name: name, Path: entryPath, Type: entryType})
	}
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Type != entries[j].Type {
			return entries[i].Type == "dir"
		}
		return entries[i].Name < entries[j].Name
	})
	return entries, nil
}

func (a *app) repoFileContent(ctx context.Context, repoPath, branch, filePath string) (string, error) {
	if !a.branchExists(ctx, repoPath, branch) {
		return "", os.ErrNotExist
	}
	ref := "refs/heads/" + branch
	objType, err := a.repoObjectType(ctx, repoPath, ref+":"+filePath)
	if err != nil || objType != "blob" {
		return "", os.ErrNotExist
	}
	out, err := a.gitBareOutput(ctx, repoPath, "show", ref+":"+filePath)
	if err != nil {
		return "", err
	}
	if !utf8.Valid(out) {
		return "", errBinaryFile
	}
	return string(out), nil
}

func (a *app) repoDiff(ctx context.Context, repoPath, sourceBranch, targetBranch string) (string, error) {
	out, err := a.gitBareOutput(ctx, repoPath, "diff", "--find-renames", "--no-color", "refs/heads/"+targetBranch+"...refs/heads/"+sourceBranch)
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func (a *app) commitRepoFile(ctx context.Context, project *Project, userID int, branch, filePath, content, message string) error {
	return a.withRepoClone(ctx, project, branch, func(dir string) error {
		fullPath, err := worktreeFilePath(dir, filePath)
		if err != nil {
			return err
		}
		if err := os.MkdirAll(filepath.Dir(fullPath), 0o755); err != nil {
			return err
		}
		if err := os.WriteFile(fullPath, []byte(content), 0o644); err != nil {
			return err
		}
		if _, err := a.gitWorktreeOutput(ctx, dir, "add", "--", filePath); err != nil {
			return err
		}
		return a.commitAndPush(ctx, dir, branch, userID, messageOrDefault(message, "Update "+filePath))
	})
}

func (a *app) removeRepoFile(ctx context.Context, project *Project, userID int, branch, filePath, message string) error {
	return a.withRepoClone(ctx, project, branch, func(dir string) error {
		fullPath, err := worktreeFilePath(dir, filePath)
		if err != nil {
			return err
		}
		info, err := os.Stat(fullPath)
		if err != nil {
			if errors.Is(err, os.ErrNotExist) {
				return os.ErrNotExist
			}
			return err
		}
		if info.IsDir() {
			return os.ErrNotExist
		}
		if _, err := a.gitWorktreeOutput(ctx, dir, "rm", "--quiet", "--", filePath); err != nil {
			return err
		}
		return a.commitAndPush(ctx, dir, branch, userID, messageOrDefault(message, "Delete "+filePath))
	})
}

func (a *app) withRepoClone(ctx context.Context, project *Project, branch string, fn func(dir string) error) error {
	repoPath, err := a.resolveRepoPath(project.RepoRel)
	if err != nil {
		return err
	}
	tmpDir, err := os.MkdirTemp("", "repo-worktree-*")
	if err != nil {
		return err
	}
	defer os.RemoveAll(tmpDir)
	worktreeDir := filepath.Join(tmpDir, "repo")
	if _, err := a.runGit(ctx, "clone", "--quiet", repoPath, worktreeDir); err != nil {
		return err
	}
	if a.branchExists(ctx, repoPath, branch) {
		if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "checkout", "--quiet", "-B", branch, "origin/"+branch); err != nil {
			return err
		}
	} else {
		branches, err := a.repoBranches(ctx, repoPath)
		if err != nil {
			return err
		}
		baseBranch := ""
		for _, item := range branches {
			if item.IsDefault {
				baseBranch = item.Name
				break
			}
		}
		if baseBranch != "" && a.branchExists(ctx, repoPath, baseBranch) {
			if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "checkout", "--quiet", "-b", branch, "origin/"+baseBranch); err != nil {
				return err
			}
		} else {
			if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "checkout", "--quiet", "--orphan", branch); err != nil {
				return err
			}
		}
	}
	return fn(worktreeDir)
}

func (a *app) commitAndPush(ctx context.Context, dir, branch string, userID int, message string) error {
	status, err := a.gitWorktreeOutput(ctx, dir, "status", "--porcelain")
	if err != nil {
		return err
	}
	if len(bytes.TrimSpace(status)) == 0 {
		return errNoChanges
	}
	authorName := fmt.Sprintf("user-%d", userID)
	a.store.mu.RLock()
	if user, ok := a.store.users[userID]; ok && strings.TrimSpace(user.Username) != "" {
		authorName = strings.ReplaceAll(strings.TrimSpace(user.Username), "\n", " ")
	}
	a.store.mu.RUnlock()
	authorEmail := fmt.Sprintf("%s@example.invalid", strings.ReplaceAll(authorName, " ", "."))
	if _, err := a.gitWorktreeOutput(ctx, dir, "-c", "user.name="+authorName, "-c", "user.email="+authorEmail, "commit", "-m", message); err != nil {
		return err
	}
	if _, err := a.gitWorktreeOutput(ctx, dir, "push", "--quiet", "origin", "HEAD:refs/heads/"+branch); err != nil {
		return err
	}
	return nil
}

func messageOrDefault(message, fallback string) string {
	if strings.TrimSpace(message) == "" {
		return fallback
	}
	return strings.TrimSpace(message)
}

func worktreeFilePath(dir, filePath string) (string, error) {
	fullPath := filepath.Clean(filepath.Join(dir, filepath.FromSlash(filePath)))
	rel, err := filepath.Rel(dir, fullPath)
	if err != nil {
		return "", err
	}
	if rel == ".." || strings.HasPrefix(rel, ".."+string(os.PathSeparator)) {
		return "", errors.New("invalid path")
	}
	return fullPath, nil
}

func (a *app) resolveRepoPath(repoRel string) (string, error) {
	repoRel = filepath.Clean(strings.TrimSpace(repoRel))
	if repoRel == "." || repoRel == "" || filepath.IsAbs(repoRel) {
		return "", errors.New("invalid repository path")
	}
	full := filepath.Clean(filepath.Join(a.reposRoot, repoRel))
	rel, err := filepath.Rel(a.reposRoot, full)
	if err != nil {
		return "", errors.New("invalid repository path")
	}
	if rel == "." || strings.HasPrefix(rel, ".."+string(os.PathSeparator)) || rel == ".." {
		return "", errors.New("invalid repository path")
	}
	return full, nil
}

func (a *app) requireBearerUser(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		auth := strings.TrimSpace(r.Header.Get("Authorization"))
		if !strings.HasPrefix(strings.ToLower(auth), "bearer ") {
			writeError(w, http.StatusUnauthorized, "bearer token required")
			return
		}
		token := strings.TrimSpace(auth[len("Bearer "):])
		if token == "" {
			writeError(w, http.StatusUnauthorized, "bearer token required")
			return
		}

		a.store.mu.RLock()
		userID, found := a.store.tokens[token]
		a.store.mu.RUnlock()
		if found {
			ctx := context.WithValue(r.Context(), userContextKey{}, userID)
			next.ServeHTTP(w, r.WithContext(ctx))
			return
		}
		writeError(w, http.StatusUnauthorized, "invalid token")
	})
}

type userContextKey struct{}

func userIDFromContext(ctx context.Context) (int, bool) {
	uid, ok := ctx.Value(userContextKey{}).(int)
	return uid, ok
}

func writeJSON(w http.ResponseWriter, status int, data any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(data)
}

func writeError(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, map[string]string{"error": message})
}

func randomToken() (string, error) {
	buf := make([]byte, 16)
	if _, err := rand.Read(buf); err != nil {
		return "", err
	}
	return hex.EncodeToString(buf), nil
}

func envOrDefault(key, fallback string) string {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	return value
}
