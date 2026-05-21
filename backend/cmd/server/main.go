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

type OrganizationMember struct {
	UserID int    `json:"userId"`
	Role   string `json:"role"`
}

type Project struct {
	ID            int    `json:"id"`
	Name          string `json:"name"`
	OrgID         int    `json:"orgId"`
	RepoRel       string `json:"repoPath"`
	Description   string `json:"description"`
	DefaultBranch string `json:"defaultBranch"`
	Archived      bool   `json:"archived"`
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
	ID             int                   `json:"id"`
	ProjectID      int                   `json:"projectId"`
	AuthorID       int                   `json:"authorId"`
	Title          string                `json:"title"`
	Description    string                `json:"description"`
	SourceBranch   string                `json:"sourceBranch"`
	TargetBranch   string                `json:"targetBranch"`
	Status         string                `json:"status"`
	Comments       []MergeRequestComment `json:"comments"`
	Mergeable      bool                  `json:"mergeable"`
	HasConflicts   bool                  `json:"hasConflicts"`
	AlreadyMerged  bool                  `json:"alreadyMerged"`
	MergedBy       *int                  `json:"mergedBy,omitempty"`
	MergedAt       *time.Time            `json:"mergedAt,omitempty"`
	MergedCommitID string                `json:"mergedCommitId,omitempty"`
	CreatedAt      time.Time             `json:"createdAt"`
	UpdatedAt      time.Time             `json:"updatedAt"`
}

type RepoTag struct {
	Name      string    `json:"name"`
	Target    string    `json:"target"`
	CreatedAt time.Time `json:"createdAt"`
}

type RepoCommit struct {
	Hash        string    `json:"hash"`
	ShortHash   string    `json:"shortHash"`
	AuthorName  string    `json:"authorName"`
	AuthorEmail string    `json:"authorEmail"`
	Subject     string    `json:"subject"`
	Body        string    `json:"body"`
	Parents     []string  `json:"parents"`
	AuthoredAt  time.Time `json:"authoredAt"`
}

type RepoCommitDetails struct {
	RepoCommit
	Diff string `json:"diff"`
}

type RepoBlameLine struct {
	LineNumber  int       `json:"lineNumber"`
	CommitHash  string    `json:"commitHash"`
	AuthorName  string    `json:"authorName"`
	AuthorEmail string    `json:"authorEmail"`
	Summary     string    `json:"summary"`
	CommittedAt time.Time `json:"committedAt"`
	Content     string    `json:"content"`
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
	orgMembers    map[int]map[int]*OrganizationMember
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
		orgMembers:    map[int]map[int]*OrganizationMember{},
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
	// noHooksDir is an empty directory used as core.hooksPath for all git
	// commands, preventing hook execution inside repositories.
	noHooksDir string
}

var repoNamePattern = regexp.MustCompile(`^[a-zA-Z0-9][a-zA-Z0-9._-]*$`)
var lfsOIDPattern = regexp.MustCompile(`^[0-9a-f]{64}$`)
var errBinaryFile = errors.New("binary file")
var errNoChanges = errors.New("no changes")
var errSymlinkAttack = errors.New("symlink attack detected")
var errMergeConflict = errors.New("merge conflict")

const gitCommandTimeout = 10 * time.Minute

const (
	orgRoleOwner     = "owner"
	orgRoleAdmin     = "admin"
	orgRoleDeveloper = "developer"
	orgRoleViewer    = "viewer"
)

// Request body size limits applied by the limitRequestBody middleware.
const (
	maxAPIBodySize     = 1 << 20       // 1 MiB  – API JSON payloads
	maxGitHTTPBodySize = 100 << 20     // 100 MiB – git smart-HTTP push/pull
	maxLFSObjectSize   = 5 * (1 << 30) // 5 GiB  – LFS object uploads
)

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

	// noHooksDir is a permanent empty directory used as core.hooksPath so
	// that git never executes hooks stored inside a repository.
	noHooksDir := filepath.Join(reposRoot, ".nohooks")
	if err := os.MkdirAll(noHooksDir, 0o755); err != nil {
		log.Fatalf("failed to create no-hooks dir: %v", err)
	}

	// Resolve staticRoot to an absolute path so Landlock can allow it.
	staticRoot, err = filepath.Abs(staticRoot)
	if err != nil {
		log.Fatalf("failed to resolve static root: %v", err)
	}

	// Apply Landlock filesystem sandboxing to this process and all children.
	setupSandbox(gitBinary, reposRoot, staticRoot)

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot, gitBinary: gitBinary, httpBaseURL: strings.TrimRight(httpBaseURL, "/"), noHooksDir: noHooksDir}
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
		api.Use(limitRequestBody(maxAPIBodySize))
		api.Post("/users", a.createUser)
		api.Get("/users", a.listUsers)
		api.With(a.requireBearerUser).Post("/users/{userID}/ssh-keys", a.addSSHKey)
		api.With(a.requireBearerUser).Post("/orgs", a.createOrganization)
		api.Get("/orgs", a.listOrganizations)
		api.Get("/orgs/{orgID}/members", a.listOrganizationMembers)
		api.With(a.requireBearerUser).Post("/orgs/{orgID}/members", a.addOrganizationMember)
		api.With(a.requireBearerUser).Patch("/orgs/{orgID}/members/{memberUserID}", a.updateOrganizationMember)
		api.With(a.requireBearerUser).Delete("/orgs/{orgID}/members/{memberUserID}", a.removeOrganizationMember)
		api.Get("/projects", a.listProjects)
		api.With(a.requireBearerUser).Post("/orgs/{orgID}/projects", a.createProject)
		api.Get("/projects/{projectID}/settings", a.getProjectSettings)
		api.With(a.requireBearerUser).Patch("/projects/{projectID}/settings", a.updateProjectSettings)
		api.Get("/projects/{projectID}/repo/branches", a.listRepoBranches)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/repo/branches", a.createRepoBranch)
		api.With(a.requireBearerUser).Delete("/projects/{projectID}/repo/branches/{branchName}", a.deleteRepoBranch)
		api.Get("/projects/{projectID}/repo/tags", a.listRepoTags)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/repo/tags", a.createRepoTag)
		api.Get("/projects/{projectID}/repo/commits", a.listRepoCommits)
		api.Get("/projects/{projectID}/repo/commits/{commitHash}", a.getRepoCommit)
		api.Get("/projects/{projectID}/repo/blame", a.getRepoBlame)
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
		api.Get("/projects/{projectID}/merge-requests/{mergeRequestID}/merge-status", a.getMergeRequestMergeStatus)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/merge-requests/{mergeRequestID}/merge", a.mergeMergeRequest)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/merge-requests/{mergeRequestID}/comments", a.addMergeRequestComment)
	})

	// Git smart-HTTP and LFS endpoints share a larger body limit; LFS object
	// uploads are limited further inside handleLFSUpload.
	r.With(limitRequestBody(maxGitHTTPBodySize)).HandleFunc("/git/*", a.handleGitHTTP)

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
	execCmd := a.gitCmd(ctx, gitMode, repoPath)
	// GIT_PROTOCOL signals the desired wire protocol version to git-upload-pack
	// and git-receive-pack. gitConfigHardenEnv also sets protocol.version=2 via
	// git config, but GIT_PROTOCOL is the SSH-transport-level signal that the
	// client and server negotiate during capability advertisement; both are needed.
	execCmd.Env = append(execCmd.Env, "GIT_PROTOCOL=version=2")
	execCmd.Stdin = s
	execCmd.Stdout = s
	execCmd.Stderr = s.Stderr()

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

func (a *app) listUsers(w http.ResponseWriter, _ *http.Request) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	users := make([]*User, 0, len(a.store.users))
	for _, user := range a.store.users {
		copyUser := *user
		copyUser.Token = ""
		users = append(users, &copyUser)
	}
	sort.Slice(users, func(i, j int) bool { return users[i].ID < users[j].ID })
	writeJSON(w, http.StatusOK, users)
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
	a.store.orgMembers[id] = map[int]*OrganizationMember{
		uid: {UserID: uid, Role: orgRoleOwner},
	}
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

func (a *app) listOrganizationMembers(w http.ResponseWriter, r *http.Request) {
	orgID, err := strconv.Atoi(chi.URLParam(r, "orgID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid org id")
		return
	}

	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	if _, ok := a.store.orgs[orgID]; !ok {
		writeError(w, http.StatusNotFound, "organization not found")
		return
	}
	orgMembers := a.store.orgMembers[orgID]
	members := make([]*OrganizationMember, 0, len(orgMembers))
	for _, member := range orgMembers {
		copyMember := *member
		members = append(members, &copyMember)
	}
	sort.Slice(members, func(i, j int) bool { return members[i].UserID < members[j].UserID })
	writeJSON(w, http.StatusOK, members)
}

func (a *app) addOrganizationMember(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	orgID, err := strconv.Atoi(chi.URLParam(r, "orgID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid org id")
		return
	}
	if !a.canUserAdminOrganization(uid, orgID) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	var req struct {
		UserID int    `json:"userId"`
		Role   string `json:"role"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	role, err := normalizeOrganizationRole(req.Role)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	if _, ok := a.store.orgs[orgID]; !ok {
		writeError(w, http.StatusNotFound, "organization not found")
		return
	}
	if _, ok := a.store.users[req.UserID]; !ok {
		writeError(w, http.StatusNotFound, "user not found")
		return
	}
	if req.UserID == a.store.orgs[orgID].OwnerID {
		role = orgRoleOwner
	}
	if _, ok := a.store.orgMembers[orgID]; !ok {
		a.store.orgMembers[orgID] = map[int]*OrganizationMember{}
	}
	member := &OrganizationMember{UserID: req.UserID, Role: role}
	a.store.orgMembers[orgID][req.UserID] = member
	writeJSON(w, http.StatusCreated, member)
}

func (a *app) updateOrganizationMember(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	orgID, err := strconv.Atoi(chi.URLParam(r, "orgID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid org id")
		return
	}
	memberUserID, err := strconv.Atoi(chi.URLParam(r, "memberUserID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid member user id")
		return
	}
	if !a.canUserAdminOrganization(uid, orgID) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	var req struct {
		Role string `json:"role"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}
	role, err := normalizeOrganizationRole(req.Role)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error())
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	org, ok := a.store.orgs[orgID]
	if !ok {
		writeError(w, http.StatusNotFound, "organization not found")
		return
	}
	if memberUserID == org.OwnerID {
		writeError(w, http.StatusBadRequest, "cannot change owner role")
		return
	}
	member, ok := a.store.orgMembers[orgID][memberUserID]
	if !ok {
		writeError(w, http.StatusNotFound, "organization member not found")
		return
	}
	member.Role = role
	writeJSON(w, http.StatusOK, member)
}

func (a *app) removeOrganizationMember(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	orgID, err := strconv.Atoi(chi.URLParam(r, "orgID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid org id")
		return
	}
	memberUserID, err := strconv.Atoi(chi.URLParam(r, "memberUserID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid member user id")
		return
	}
	if !a.canUserAdminOrganization(uid, orgID) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	org, ok := a.store.orgs[orgID]
	if !ok {
		writeError(w, http.StatusNotFound, "organization not found")
		return
	}
	if memberUserID == org.OwnerID {
		writeError(w, http.StatusBadRequest, "cannot remove organization owner")
		return
	}
	if _, ok := a.store.orgMembers[orgID][memberUserID]; !ok {
		writeError(w, http.StatusNotFound, "organization member not found")
		return
	}
	delete(a.store.orgMembers[orgID], memberUserID)
	w.WriteHeader(http.StatusNoContent)
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
	if !organizationRoleCanAdmin(a.organizationRoleLocked(orgID, uid)) {
		writeError(w, http.StatusForbidden, "only organization admins can create project")
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
	if _, err := a.runGit(ctx, "init", "--bare", repoPath); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to initialize repository")
		return
	}

	p := &Project{ID: id, Name: name, OrgID: orgID, RepoRel: repoRel, DefaultBranch: "main"}
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

func (a *app) getProjectSettings(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	project, ok := a.store.projects[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	writeJSON(w, http.StatusOK, project)
}

func (a *app) updateProjectSettings(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	var req struct {
		Description   *string `json:"description"`
		DefaultBranch *string `json:"defaultBranch"`
		Archived      *bool   `json:"archived"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
		return
	}

	project, repoPath, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	if !a.canUserAdminProject(uid, project) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}

	if req.DefaultBranch != nil {
		ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
		defer cancel()
		branch, err := a.resolveRequestedBranch(ctx, repoPath, *req.DefaultBranch)
		if err != nil {
			writeError(w, http.StatusBadRequest, "invalid default branch")
			return
		}
		branches, err := a.repoBranches(ctx, repoPath)
		if err != nil {
			writeError(w, http.StatusInternalServerError, "failed to inspect branches")
			return
		}
		if len(branches) > 0 && !a.branchExists(ctx, repoPath, branch) {
			writeError(w, http.StatusBadRequest, "default branch must exist")
			return
		}
		*req.DefaultBranch = branch
	}

	a.store.mu.Lock()
	defer a.store.mu.Unlock()
	project, ok := a.store.projects[projectID]
	if !ok {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	if req.Description != nil {
		project.Description = strings.TrimSpace(*req.Description)
	}
	if req.DefaultBranch != nil {
		project.DefaultBranch = strings.TrimSpace(*req.DefaultBranch)
	}
	if req.Archived != nil {
		project.Archived = *req.Archived
	}
	writeJSON(w, http.StatusOK, project)
}

func (a *app) createIssue(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	var req struct {
		Title       string   `json:"title"`
		Description string   `json:"description"`
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
		Title       *string   `json:"title"`
		Description *string   `json:"description"`
		Status      *string   `json:"status"`
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
	defaultBranch := a.projectDefaultBranch(project)
	if defaultBranch != "" {
		found := false
		for i := range branches {
			branches[i].IsDefault = branches[i].Name == defaultBranch
			found = found || branches[i].IsDefault
		}
		if !found && len(branches) == 1 {
			branches[0].IsDefault = true
		}
	}
	writeJSON(w, http.StatusOK, branches)
}

func (a *app) createRepoBranch(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	var req struct {
		Name         string `json:"name"`
		SourceBranch string `json:"sourceBranch"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
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
	branchName, err := a.resolveRequestedBranch(ctx, repoPath, req.Name)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch name")
		return
	}
	sourceBranch, err := a.resolveRequestedBranch(ctx, repoPath, req.SourceBranch)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid source branch")
		return
	}
	if err := a.createBranch(ctx, repoPath, branchName, sourceBranch); err != nil {
		if errors.Is(err, os.ErrExist) {
			writeError(w, http.StatusConflict, "branch already exists")
			return
		}
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusBadRequest, "source branch must exist")
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to create branch")
		return
	}
	writeJSON(w, http.StatusCreated, RepoBranch{Name: branchName, IsDefault: branchName == a.projectDefaultBranch(project)})
}

func (a *app) deleteRepoBranch(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}

	branchName := strings.TrimSpace(chi.URLParam(r, "branchName"))
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
	resolvedBranch, err := a.resolveRequestedBranch(ctx, repoPath, branchName)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}
	if resolvedBranch == a.projectDefaultBranch(project) {
		writeError(w, http.StatusBadRequest, "cannot delete default branch")
		return
	}
	if err := a.deleteBranch(ctx, repoPath, resolvedBranch); err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "branch not found")
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to delete branch")
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (a *app) listRepoTags(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	_, repoPath, err := a.projectRepoPath(projectID)
	if err != nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	tags, err := a.repoTags(ctx, repoPath)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to list tags")
		return
	}
	writeJSON(w, http.StatusOK, tags)
}

func (a *app) createRepoTag(w http.ResponseWriter, r *http.Request) {
	uid, _ := userIDFromContext(r.Context())
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	var req struct {
		Name   string `json:"name"`
		Target string `json:"target"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json")
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
	target := strings.TrimSpace(req.Target)
	if target == "" {
		target = a.projectDefaultBranch(project)
	}
	tag, err := a.createTag(ctx, repoPath, req.Name, target)
	if err != nil {
		if errors.Is(err, os.ErrExist) {
			writeError(w, http.StatusConflict, "tag already exists")
			return
		}
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusBadRequest, "target not found")
			return
		}
		writeError(w, http.StatusBadRequest, "invalid tag")
		return
	}
	writeJSON(w, http.StatusCreated, tag)
}

func (a *app) listRepoCommits(w http.ResponseWriter, r *http.Request) {
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
	limit := 20
	if raw := strings.TrimSpace(r.URL.Query().Get("limit")); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 || parsed > 100 {
			writeError(w, http.StatusBadRequest, "invalid limit")
			return
		}
		limit = parsed
	}
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"), a.projectDefaultBranch(project))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}
	commits, err := a.repoCommitLog(ctx, repoPath, branch, repoPathValue, limit)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to load commits")
		return
	}
	writeJSON(w, http.StatusOK, commits)
}

func (a *app) getRepoCommit(w http.ResponseWriter, r *http.Request) {
	projectID, err := strconv.Atoi(chi.URLParam(r, "projectID"))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid project id")
		return
	}
	_, repoPath, err := a.projectRepoPath(projectID)
	if err != nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	commit, err := a.repoCommitDetails(ctx, repoPath, chi.URLParam(r, "commitHash"))
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "commit not found")
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to load commit")
		return
	}
	writeJSON(w, http.StatusOK, commit)
}

func (a *app) getRepoBlame(w http.ResponseWriter, r *http.Request) {
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
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"), a.projectDefaultBranch(project))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}
	lines, err := a.repoBlame(ctx, repoPath, branch, filePath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusNotFound, "file not found")
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to load blame")
		return
	}
	writeJSON(w, http.StatusOK, lines)
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
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"), a.projectDefaultBranch(project))
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
	branch, err := a.resolveRequestedBranch(ctx, repoPath, r.URL.Query().Get("branch"), a.projectDefaultBranch(project))
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
	branch, err := a.resolveRequestedBranch(ctx, repoPath, req.Branch, a.projectDefaultBranch(project))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid branch")
		return
	}

	if err := a.commitRepoFile(ctx, project, uid, branch, filePath, req.Content, req.Message); err != nil {
		if errors.Is(err, errNoChanges) {
			writeError(w, http.StatusBadRequest, "no changes to commit")
			return
		}
		if errors.Is(err, errSymlinkAttack) {
			writeError(w, http.StatusConflict, "symlink attack detected")
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
	branch, err := a.resolveRequestedBranch(ctx, repoPath, req.Branch, a.projectDefaultBranch(project))
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
		if errors.Is(err, errSymlinkAttack) {
			writeError(w, http.StatusConflict, "symlink attack detected")
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
	sourceBranch, err := a.resolveRequestedBranch(ctx, repoPath, req.SourceBranch, a.projectDefaultBranch(project))
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid source branch")
		return
	}
	targetBranch, err := a.resolveRequestedBranch(ctx, repoPath, req.TargetBranch, a.projectDefaultBranch(project))
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
	hydrated, err := a.enrichMergeRequest(r.Context(), repoPath, mr)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to inspect merge request")
		return
	}
	writeJSON(w, http.StatusCreated, hydrated)
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
		list = append(list, cloneMergeRequest(mr))
	}
	sort.Slice(list, func(i, j int) bool { return list[i].ID < list[j].ID })
	_, repoPath, err := a.projectRepoPath(projectID)
	if err != nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	for i, mr := range list {
		list[i], err = a.enrichMergeRequest(ctx, repoPath, mr)
		if err != nil {
			writeError(w, http.StatusInternalServerError, "failed to inspect merge requests")
			return
		}
	}
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
	_, repoPath, err := a.projectRepoPath(projectID)
	if err != nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	hydrated, err := a.enrichMergeRequest(r.Context(), repoPath, mr)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to inspect merge request")
		return
	}
	writeJSON(w, http.StatusOK, hydrated)
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

func (a *app) getMergeRequestMergeStatus(w http.ResponseWriter, r *http.Request) {
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
	mergeable, hasConflicts, alreadyMerged, err := a.mergeRequestState(ctx, repoPath, mr)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "failed to inspect merge status")
		return
	}
	writeJSON(w, http.StatusOK, map[string]bool{
		"mergeable":     mergeable,
		"hasConflicts":  hasConflicts,
		"alreadyMerged": alreadyMerged,
	})
}

func (a *app) mergeMergeRequest(w http.ResponseWriter, r *http.Request) {
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
	project, _, err := a.projectRepoPath(projectID)
	if err != nil || project == nil {
		writeError(w, http.StatusNotFound, "project not found")
		return
	}
	if !a.canUserWriteProject(uid, project) {
		writeError(w, http.StatusForbidden, "forbidden")
		return
	}
	mr, err := a.lookupMergeRequest(projectID, mergeRequestID)
	if err != nil {
		writeError(w, http.StatusNotFound, "merge request not found")
		return
	}
	if mr.Status != "open" {
		writeError(w, http.StatusBadRequest, "merge request is not open")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), gitCommandTimeout)
	defer cancel()
	commitID, err := a.performMergeRequest(ctx, project, mr, uid)
	if err != nil {
		if errors.Is(err, errMergeConflict) {
			writeError(w, http.StatusConflict, "merge request has conflicts")
			return
		}
		if errors.Is(err, os.ErrNotExist) {
			writeError(w, http.StatusBadRequest, "merge request branches are missing")
			return
		}
		writeError(w, http.StatusInternalServerError, "failed to merge merge request")
		return
	}

	now := time.Now().UTC()
	a.store.mu.Lock()
	if stored, ok := a.store.mergeRequests[projectID][mergeRequestID]; ok {
		stored.Status = "merged"
		stored.Mergeable = false
		stored.HasConflicts = false
		stored.AlreadyMerged = true
		stored.MergedAt = &now
		stored.MergedBy = &uid
		stored.MergedCommitID = commitID
		stored.UpdatedAt = now
		mr = stored
	}
	a.store.mu.Unlock()
	writeJSON(w, http.StatusOK, mr)
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
	cmd := a.gitCmd(ctx, "http-backend")
	cmd.Env = append(cmd.Env,
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
	objPath, err := securePathWithinRoot(repoPath, filepath.Join("lfs", "objects", oid[:2], oid[2:4], oid))
	if err != nil {
		return ""
	}
	return objPath
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

	// Batch payloads are small JSON; cap to prevent oversized requests.
	r.Body = http.MaxBytesReader(w, r.Body, maxAPIBodySize)

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
		if objPath == "" {
			objResp.Error = &lfsObjectError{Code: http.StatusConflict, Message: "symlink attack detected"}
			objects = append(objects, objResp)
			continue
		}
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
	if objPath == "" {
		writeError(w, http.StatusConflict, "symlink attack detected")
		return
	}
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

	// Enforce a maximum LFS object size to prevent storage exhaustion.
	r.Body = http.MaxBytesReader(w, r.Body, maxLFSObjectSize)

	objPath := a.lfsObjectPath(repoPath, oid)
	if objPath == "" {
		writeLFSError(w, http.StatusConflict, "symlink attack detected")
		return
	}

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

func normalizeOrganizationRole(role string) (string, error) {
	normalized := strings.ToLower(strings.TrimSpace(role))
	switch normalized {
	case orgRoleOwner, orgRoleAdmin, orgRoleDeveloper, orgRoleViewer:
		return normalized, nil
	default:
		return "", errors.New("role must be owner, admin, developer, or viewer")
	}
}

func organizationRoleCanWrite(role string) bool {
	return role == orgRoleOwner || role == orgRoleAdmin || role == orgRoleDeveloper
}

func organizationRoleCanAdmin(role string) bool {
	return role == orgRoleOwner || role == orgRoleAdmin
}

func (a *app) organizationRoleLocked(orgID, userID int) string {
	if orgMembers, ok := a.store.orgMembers[orgID]; ok {
		if member, ok := orgMembers[userID]; ok {
			return member.Role
		}
	}
	if org, ok := a.store.orgs[orgID]; ok && org.OwnerID == userID {
		return orgRoleOwner
	}
	return ""
}

func (a *app) organizationRole(orgID, userID int) string {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	return a.organizationRoleLocked(orgID, userID)
}

func (a *app) canUserAdminOrganization(userID, orgID int) bool {
	return organizationRoleCanAdmin(a.organizationRole(orgID, userID))
}

func (a *app) canUserAdminProject(userID int, project *Project) bool {
	return organizationRoleCanAdmin(a.organizationRole(project.OrgID, userID))
}

func (a *app) canUserWriteProject(userID int, project *Project) bool {
	if project == nil || project.Archived {
		return false
	}
	return organizationRoleCanWrite(a.organizationRole(project.OrgID, userID))
}

func (a *app) projectDefaultBranch(project *Project) string {
	if project != nil && strings.TrimSpace(project.DefaultBranch) != "" {
		return strings.TrimSpace(project.DefaultBranch)
	}
	return "main"
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

func cloneMergeRequest(mr *MergeRequest) *MergeRequest {
	if mr == nil {
		return nil
	}
	copyMR := *mr
	copyMR.Comments = append([]MergeRequestComment(nil), mr.Comments...)
	return &copyMR
}

func (a *app) enrichMergeRequest(ctx context.Context, repoPath string, mr *MergeRequest) (*MergeRequest, error) {
	copyMR := cloneMergeRequest(mr)
	mergeable, hasConflicts, alreadyMerged, err := a.mergeRequestState(ctx, repoPath, copyMR)
	if err != nil {
		return nil, err
	}
	copyMR.Mergeable = mergeable
	copyMR.HasConflicts = hasConflicts
	copyMR.AlreadyMerged = alreadyMerged
	return copyMR, nil
}

func (a *app) mergeRequestState(ctx context.Context, repoPath string, mr *MergeRequest) (bool, bool, bool, error) {
	if mr.Status == "merged" {
		return false, false, true, nil
	}
	if mr.Status != "open" {
		return false, false, false, nil
	}
	if !a.branchExists(ctx, repoPath, mr.SourceBranch) || !a.branchExists(ctx, repoPath, mr.TargetBranch) {
		return false, false, false, os.ErrNotExist
	}
	alreadyMerged, err := a.isAncestor(ctx, repoPath, mr.SourceBranch, mr.TargetBranch)
	if err != nil {
		return false, false, false, err
	}
	if alreadyMerged {
		return false, false, true, nil
	}
	mergeable, err := a.canMergeBranches(ctx, repoPath, mr.SourceBranch, mr.TargetBranch)
	if err != nil {
		if errors.Is(err, errMergeConflict) {
			return false, true, false, nil
		}
		return false, false, false, err
	}
	return mergeable, false, false, nil
}

func (a *app) runGit(ctx context.Context, args ...string) ([]byte, error) {
	cmd := a.gitCmd(ctx, args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("git %s: %w (%s)", strings.Join(args, " "), err, strings.TrimSpace(string(out)))
	}
	return out, nil
}

// gitCmd returns a hardened exec.Cmd for the given git arguments. It uses a
// sanitized environment (no inherited secrets) and applies OS-level process
// isolation via hardenCmd.
func (a *app) gitCmd(ctx context.Context, args ...string) *exec.Cmd {
	cmd := exec.CommandContext(ctx, a.gitBinary, args...)
	cmd.Env = a.gitSafeEnv()
	hardenCmd(cmd)
	return cmd
}

// gitSafeEnv returns a minimal, sanitized environment for git subprocesses.
// Inheriting the full process environment risks leaking secrets (database URLs,
// cloud credentials, etc.) into untrusted git hook processes or via git's own
// config loading. The returned env includes just what git needs for local
// operations plus hardened config overrides.
//
// The optional extra arguments append additional KEY=VALUE pairs after the
// base environment. Callers use this for transport-specific variables (e.g.
// GIT_PROTOCOL for SSH sessions, or CGI variables for http-backend) that are
// not secret and are safe to expose to the subprocess.
func (a *app) gitSafeEnv(extra ...string) []string {
	env := []string{
		"PATH=" + os.Getenv("PATH"),
		// Use noHooksDir as HOME so git cannot read a ~/.gitconfig that might
		// contain unsafe settings or credential helpers.
		"HOME=" + a.noHooksDir,
		// Prevent git from prompting for passwords (no TTY in a server context).
		"GIT_TERMINAL_PROMPT=0",
		// Skip /etc/gitconfig so the server's system git config doesn't affect
		// repository operations.
		"GIT_CONFIG_NOSYSTEM=1",
		// Use a consistent locale for predictable git output parsing.
		"LANG=C",
		"LC_ALL=C",
	}
	env = append(env, a.gitConfigHardenEnv()...)
	return append(env, extra...)
}

// gitConfigHardenEnv returns environment variables that inject hardened git
// configuration overrides (GIT_CONFIG_COUNT / GIT_CONFIG_KEY_N / GIT_CONFIG_VALUE_N).
// These take precedence over repo-local and user git config, so they cannot be
// overridden by content inside a repository. Requires git ≥ 2.31; older
// versions silently ignore the unknown env vars (safe degradation).
func (a *app) gitConfigHardenEnv() []string {
	configs := [][2]string{
		// Redirect hooks to an empty directory, preventing hook execution for
		// both upload-pack (pre/post-receive) and local operations.
		{"core.hooksPath", a.noHooksDir},
		// Disable automatic GC to prevent resource-exhaustion via crafted repos.
		{"gc.auto", "0"},
		// Enable fsck on transfer, receive, and fetch to reject malformed objects.
		{"transfer.fsckObjects", "true"},
		{"receive.fsckObjects", "true"},
		{"fetch.fsckObjects", "true"},
		// Disable partial-clone filters (reduces attack surface).
		{"uploadpack.allowFilter", "false"},
		// Disable custom pack-objects hook.
		{"uploadpack.packObjectsHook", ""},
		// Request protocol v2 for push/pull (more efficient and more auditable).
		{"protocol.version", "2"},
		// Cap pack threads to prevent CPU exhaustion via a large push.
		{"pack.threads", "1"},
		// Block all network protocols except local file:// so git subprocesses
		// cannot make outbound connections (SSRF via submodule URLs, etc.).
		{"protocol.allow", "never"},
		{"protocol.file.allow", "always"},
		// Disable automatic submodule recursion.
		{"fetch.recurseSubmodules", "no"},
		{"submodule.recurse", "false"},
		// Clear any credential helper that might exfiltrate credentials.
		{"credential.helper", ""},
	}

	result := []string{fmt.Sprintf("GIT_CONFIG_COUNT=%d", len(configs))}
	for i, kv := range configs {
		result = append(result,
			fmt.Sprintf("GIT_CONFIG_KEY_%d=%s", i, kv[0]),
			fmt.Sprintf("GIT_CONFIG_VALUE_%d=%s", i, kv[1]),
		)
	}
	return result
}

// limitRequestBody returns a middleware that caps the request body size to
// maxBytes. Requests that exceed the limit receive a 413 response.
func limitRequestBody(maxBytes int64) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			r.Body = http.MaxBytesReader(w, r.Body, maxBytes)
			next.ServeHTTP(w, r)
		})
	}
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

func (a *app) resolveRequestedBranch(ctx context.Context, repoPath, raw string, preferredDefault ...string) (string, error) {
	branch := strings.TrimSpace(raw)
	if branch == "" {
		if len(preferredDefault) > 0 {
			preferred := strings.TrimSpace(preferredDefault[0])
			if preferred != "" {
				if branches, err := a.repoBranches(ctx, repoPath); err == nil {
					if len(branches) == 0 || a.branchExists(ctx, repoPath, preferred) {
						return preferred, nil
					}
				}
			}
		}
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

func (a *app) createBranch(ctx context.Context, repoPath, branchName, sourceBranch string) error {
	if a.branchExists(ctx, repoPath, branchName) {
		return os.ErrExist
	}
	if !a.branchExists(ctx, repoPath, sourceBranch) {
		return os.ErrNotExist
	}
	_, err := a.gitBareOutput(ctx, repoPath, "branch", branchName, "refs/heads/"+sourceBranch)
	return err
}

func (a *app) deleteBranch(ctx context.Context, repoPath, branchName string) error {
	if !a.branchExists(ctx, repoPath, branchName) {
		return os.ErrNotExist
	}
	_, err := a.gitBareOutput(ctx, repoPath, "update-ref", "-d", "refs/heads/"+branchName)
	return err
}

func (a *app) repoTags(ctx context.Context, repoPath string) ([]RepoTag, error) {
	out, err := a.gitBareOutput(ctx, repoPath, "for-each-ref", "--sort=-creatordate", "--format=%(refname:short)%00%(objectname)%00%(creatordate:iso-strict)", "refs/tags")
	if err != nil {
		return nil, err
	}
	tags := []RepoTag{}
	for _, line := range strings.Split(strings.TrimSpace(string(out)), "\n") {
		if strings.TrimSpace(line) == "" {
			continue
		}
		parts := strings.Split(line, "\x00")
		if len(parts) != 3 {
			continue
		}
		createdAt, _ := time.Parse(time.RFC3339, strings.TrimSpace(parts[2]))
		tags = append(tags, RepoTag{Name: parts[0], Target: parts[1], CreatedAt: createdAt})
	}
	return tags, nil
}

func (a *app) createTag(ctx context.Context, repoPath, tagName, target string) (*RepoTag, error) {
	tagName = strings.TrimSpace(tagName)
	if tagName == "" {
		return nil, errors.New("tag name required")
	}
	if _, err := a.runGit(ctx, "check-ref-format", "refs/tags/"+tagName); err != nil {
		return nil, err
	}
	if _, err := a.gitBareOutput(ctx, repoPath, "rev-parse", "--verify", "--quiet", "refs/tags/"+tagName); err == nil {
		return nil, os.ErrExist
	}
	out, err := a.gitBareOutput(ctx, repoPath, "rev-parse", "--verify", "--quiet", target)
	if err != nil {
		if strings.TrimSpace(target) != "" {
			out, err = a.gitBareOutput(ctx, repoPath, "rev-parse", "--verify", "--quiet", "refs/heads/"+target)
		}
		if err != nil {
			return nil, os.ErrNotExist
		}
	}
	targetHash := strings.TrimSpace(string(out))
	if _, err := a.gitBareOutput(ctx, repoPath, "update-ref", "refs/tags/"+tagName, targetHash); err != nil {
		return nil, err
	}
	return &RepoTag{Name: tagName, Target: targetHash, CreatedAt: time.Now().UTC()}, nil
}

func parseGitTime(value string) time.Time {
	parsed, err := time.Parse(time.RFC3339, strings.TrimSpace(value))
	if err != nil {
		return time.Time{}
	}
	return parsed
}

func parseParents(value string) []string {
	value = strings.TrimSpace(value)
	if value == "" {
		return nil
	}
	return strings.Fields(value)
}

func (a *app) repoCommitLog(ctx context.Context, repoPath, branch, repoPathValue string, limit int) ([]RepoCommit, error) {
	args := []string{
		"log",
		"--max-count", strconv.Itoa(limit),
		"--date=iso-strict",
		"--pretty=format:%H%x00%h%x00%an%x00%ae%x00%aI%x00%s%x00%b%x00%P%x01",
		"refs/heads/" + branch,
	}
	if repoPathValue != "" {
		args = append(args, "--", repoPathValue)
	}
	out, err := a.gitBareOutput(ctx, repoPath, args...)
	if err != nil {
		return nil, err
	}
	records := bytes.Split(out, []byte{1})
	commits := make([]RepoCommit, 0, len(records))
	for _, record := range records {
		record = bytes.TrimSpace(record)
		if len(record) == 0 {
			continue
		}
		parts := strings.Split(string(record), "\x00")
		if len(parts) < 8 {
			continue
		}
		commits = append(commits, RepoCommit{
			Hash:        parts[0],
			ShortHash:   parts[1],
			AuthorName:  parts[2],
			AuthorEmail: parts[3],
			AuthoredAt:  parseGitTime(parts[4]),
			Subject:     parts[5],
			Body:        strings.TrimSpace(parts[6]),
			Parents:     parseParents(parts[7]),
		})
	}
	return commits, nil
}

func (a *app) repoCommitDetails(ctx context.Context, repoPath, commitHash string) (*RepoCommitDetails, error) {
	out, err := a.gitBareOutput(ctx, repoPath, "show", "--date=iso-strict", "--format=%H%x00%h%x00%an%x00%ae%x00%aI%x00%s%x00%b%x00%P%x00", "--patch", "--stat", "--summary", commitHash)
	if err != nil {
		return nil, os.ErrNotExist
	}
	parts := bytes.SplitN(out, []byte{0}, 9)
	if len(parts) < 9 {
		return nil, errors.New("invalid git show output")
	}
	return &RepoCommitDetails{
		RepoCommit: RepoCommit{
			Hash:        string(parts[0]),
			ShortHash:   string(parts[1]),
			AuthorName:  string(parts[2]),
			AuthorEmail: string(parts[3]),
			AuthoredAt:  parseGitTime(string(parts[4])),
			Subject:     string(parts[5]),
			Body:        strings.TrimSpace(string(parts[6])),
			Parents:     parseParents(string(parts[7])),
		},
		Diff: strings.TrimSpace(string(parts[8])),
	}, nil
}

func (a *app) repoBlame(ctx context.Context, repoPath, branch, filePath string) ([]RepoBlameLine, error) {
	if !a.branchExists(ctx, repoPath, branch) {
		return nil, os.ErrNotExist
	}
	if _, err := a.repoObjectType(ctx, repoPath, "refs/heads/"+branch+":"+filePath); err != nil {
		return nil, os.ErrNotExist
	}
	out, err := a.gitBareOutput(ctx, repoPath, "blame", "--line-porcelain", "refs/heads/"+branch, "--", filePath)
	if err != nil {
		return nil, err
	}
	lines := []RepoBlameLine{}
	var current RepoBlameLine
	for _, line := range strings.Split(string(out), "\n") {
		switch {
		case strings.HasPrefix(line, "\t"):
			current.Content = strings.TrimPrefix(line, "\t")
			current.LineNumber = len(lines) + 1
			lines = append(lines, current)
		case len(strings.Fields(line)) >= 3 && len(current.CommitHash) == 0:
			current = RepoBlameLine{CommitHash: strings.Fields(line)[0]}
		case strings.HasPrefix(line, "author "):
			current.AuthorName = strings.TrimPrefix(line, "author ")
		case strings.HasPrefix(line, "author-mail "):
			current.AuthorEmail = strings.Trim(strings.TrimPrefix(line, "author-mail "), "<>")
		case strings.HasPrefix(line, "author-time "):
			seconds, _ := strconv.ParseInt(strings.TrimPrefix(line, "author-time "), 10, 64)
			current.CommittedAt = time.Unix(seconds, 0).UTC()
		case strings.HasPrefix(line, "summary "):
			current.Summary = strings.TrimPrefix(line, "summary ")
		}
	}
	return lines, nil
}

func (a *app) isAncestor(ctx context.Context, repoPath, sourceBranch, targetBranch string) (bool, error) {
	cmd := a.gitCmd(ctx, "--git-dir", repoPath, "merge-base", "--is-ancestor", "refs/heads/"+sourceBranch, "refs/heads/"+targetBranch)
	out, err := cmd.CombinedOutput()
	if err == nil {
		return true, nil
	}
	if exitErr, ok := err.(*exec.ExitError); ok && exitErr.ExitCode() == 1 {
		return false, nil
	}
	return false, fmt.Errorf("git merge-base --is-ancestor: %w (%s)", err, strings.TrimSpace(string(out)))
}

func (a *app) canMergeBranches(ctx context.Context, repoPath, sourceBranch, targetBranch string) (bool, error) {
	tmpDir, err := os.MkdirTemp("", "merge-check-*")
	if err != nil {
		return false, err
	}
	defer os.RemoveAll(tmpDir)
	worktreeDir := filepath.Join(tmpDir, "repo")
	if _, err := a.runGit(ctx, "clone", "--quiet", repoPath, worktreeDir); err != nil {
		return false, err
	}
	if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "checkout", "--quiet", "-B", targetBranch, "origin/"+targetBranch); err != nil {
		return false, err
	}
	if _, err := a.gitWorktreeOutput(
		ctx,
		worktreeDir,
		"-c",
		"user.name=merge-check",
		"-c",
		"user.email=merge-check@example.invalid",
		"merge",
		"--no-ff",
		"--no-commit",
		"origin/"+sourceBranch,
	); err != nil {
		_, _ = a.gitWorktreeOutput(ctx, worktreeDir, "merge", "--abort")
		return false, errMergeConflict
	}
	_, _ = a.gitWorktreeOutput(ctx, worktreeDir, "merge", "--abort")
	return true, nil
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
		baseBranch := a.projectDefaultBranch(project)
		if baseBranch == "" || !a.branchExists(ctx, repoPath, baseBranch) {
			baseBranch = ""
			for _, item := range branches {
				if item.IsDefault {
					baseBranch = item.Name
					break
				}
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

func (a *app) performMergeRequest(ctx context.Context, project *Project, mr *MergeRequest, userID int) (string, error) {
	repoPath, err := a.resolveRepoPath(project.RepoRel)
	if err != nil {
		return "", err
	}
	if !a.branchExists(ctx, repoPath, mr.SourceBranch) || !a.branchExists(ctx, repoPath, mr.TargetBranch) {
		return "", os.ErrNotExist
	}
	alreadyMerged, err := a.isAncestor(ctx, repoPath, mr.SourceBranch, mr.TargetBranch)
	if err != nil {
		return "", err
	}
	if alreadyMerged {
		out, err := a.gitBareOutput(ctx, repoPath, "rev-parse", "--verify", "refs/heads/"+mr.TargetBranch)
		if err != nil {
			return "", err
		}
		return strings.TrimSpace(string(out)), nil
	}

	tmpDir, err := os.MkdirTemp("", "mr-merge-*")
	if err != nil {
		return "", err
	}
	defer os.RemoveAll(tmpDir)
	worktreeDir := filepath.Join(tmpDir, "repo")
	if _, err := a.runGit(ctx, "clone", "--quiet", repoPath, worktreeDir); err != nil {
		return "", err
	}
	if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "checkout", "--quiet", "-B", mr.TargetBranch, "origin/"+mr.TargetBranch); err != nil {
		return "", err
	}
	authorName, authorEmail := a.authorIdentity(userID)
	if _, err := a.gitWorktreeOutput(ctx, worktreeDir, "-c", "user.name="+authorName, "-c", "user.email="+authorEmail, "merge", "--no-ff", "--no-edit", "origin/"+mr.SourceBranch); err != nil {
		_, _ = a.gitWorktreeOutput(ctx, worktreeDir, "merge", "--abort")
		return "", errMergeConflict
	}
	if err := a.pushExistingCommit(ctx, worktreeDir, mr.TargetBranch); err != nil {
		return "", err
	}
	out, err := a.gitWorktreeOutput(ctx, worktreeDir, "rev-parse", "HEAD")
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

func (a *app) commitAndPush(ctx context.Context, dir, branch string, userID int, message string) error {
	status, err := a.gitWorktreeOutput(ctx, dir, "status", "--porcelain")
	if err != nil {
		return err
	}
	if len(bytes.TrimSpace(status)) == 0 {
		return errNoChanges
	}
	authorName, authorEmail := a.authorIdentity(userID)
	if _, err := a.gitWorktreeOutput(ctx, dir, "-c", "user.name="+authorName, "-c", "user.email="+authorEmail, "commit", "-m", message); err != nil {
		return err
	}
	return a.pushExistingCommit(ctx, dir, branch)
}

func (a *app) authorIdentity(userID int) (string, string) {
	authorName := fmt.Sprintf("user-%d", userID)
	a.store.mu.RLock()
	if user, ok := a.store.users[userID]; ok && strings.TrimSpace(user.Username) != "" {
		authorName = strings.ReplaceAll(strings.TrimSpace(user.Username), "\n", " ")
	}
	a.store.mu.RUnlock()
	authorEmail := fmt.Sprintf("%s@example.invalid", strings.ReplaceAll(authorName, " ", "."))
	return authorName, authorEmail
}

func (a *app) pushExistingCommit(ctx context.Context, dir, branch string) error {
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
	return securePathWithinRoot(dir, filepath.FromSlash(filePath))
}

func (a *app) resolveRepoPath(repoRel string) (string, error) {
	return securePathWithinRoot(a.reposRoot, repoRel)
}

func securePathWithinRoot(root, rel string) (string, error) {
	rel = filepath.Clean(strings.TrimSpace(rel))
	if rel == "." || rel == "" || filepath.IsAbs(rel) {
		return "", errors.New("invalid path")
	}

	rootPath, err := filepath.EvalSymlinks(root)
	if err != nil {
		return "", err
	}
	rootPath = filepath.Clean(rootPath)

	full := filepath.Clean(filepath.Join(rootPath, rel))
	if !pathWithinRoot(rootPath, full) {
		return "", errors.New("invalid path")
	}

	resolved, err := resolvePathWithExistingSymlinks(full)
	if err != nil {
		return "", err
	}
	if resolved != full {
		return "", errSymlinkAttack
	}
	if !pathWithinRoot(rootPath, resolved) {
		return "", errSymlinkAttack
	}
	return resolved, nil
}

func resolvePathWithExistingSymlinks(full string) (string, error) {
	var suffix []string
	current := filepath.Clean(full)
	for {
		_, err := os.Lstat(current)
		switch {
		case err == nil:
			resolved, err := filepath.EvalSymlinks(current)
			if err != nil {
				return "", err
			}
			for i := len(suffix) - 1; i >= 0; i-- {
				resolved = filepath.Join(resolved, suffix[i])
			}
			return filepath.Clean(resolved), nil
		case errors.Is(err, os.ErrNotExist):
			parent := filepath.Dir(current)
			if parent == current {
				return "", err
			}
			suffix = append(suffix, filepath.Base(current))
			current = parent
		default:
			return "", err
		}
	}
}

func pathWithinRoot(root, full string) bool {
	rel, err := filepath.Rel(root, full)
	if err != nil {
		return false
	}
	return rel != ".." && !strings.HasPrefix(rel, ".."+string(os.PathSeparator))
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
