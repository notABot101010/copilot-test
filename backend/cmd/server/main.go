package main

import (
	"bufio"
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	gliderssh "github.com/gliderlabs/ssh"
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
	Comments    []IssueComment `json:"comments"`
	CreatedAt   time.Time      `json:"createdAt"`
	UpdatedAt   time.Time      `json:"updatedAt"`
}

type Store struct {
	mu            sync.RWMutex
	nextUserID    int
	nextOrgID     int
	nextProjectID int
	nextIssueID   int
	nextCommentID int
	users         map[int]*User
	orgs          map[int]*Organization
	projects      map[int]*Project
	issues        map[int]map[int]*Issue
}

func newStore() *Store {
	return &Store{
		nextUserID:    1,
		nextOrgID:     1,
		nextProjectID: 1,
		nextIssueID:   1,
		nextCommentID: 1,
		users:         map[int]*User{},
		orgs:          map[int]*Organization{},
		projects:      map[int]*Project{},
		issues:        map[int]map[int]*Issue{},
	}
}

type app struct {
	store      *Store
	reposRoot  string
	staticRoot string
}

func main() {
	httpPort := envOrDefault("HTTP_PORT", "8080")
	sshPort := envOrDefault("SSH_PORT", "2222")
	reposRoot := envOrDefault("REPOS_ROOT", "./data/repos")
	staticRoot := envOrDefault("STATIC_ROOT", "./frontend/dist")

	if err := os.MkdirAll(reposRoot, 0o755); err != nil {
		log.Fatalf("failed to create repos root: %v", err)
	}

	a := &app{store: newStore(), reposRoot: reposRoot, staticRoot: staticRoot}
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

	err := <-errCh
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
		api.With(a.requireBearerUser).Post("/projects/{projectID}/issues", a.createIssue)
		api.Get("/projects/{projectID}/issues", a.listIssues)
		api.With(a.requireBearerUser).Patch("/projects/{projectID}/issues/{issueID}", a.updateIssue)
		api.With(a.requireBearerUser).Post("/projects/{projectID}/issues/{issueID}/comments", a.addIssueComment)
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
	absRoot, _ := filepath.Abs(staticRoot)
	return func(w http.ResponseWriter, r *http.Request) {
		path := filepath.Clean(strings.TrimPrefix(r.URL.Path, "/"))
		if path == "." {
			next.ServeHTTP(w, r)
			return
		}
		candidate := filepath.Join(staticRoot, path)
		absCandidate, err := filepath.Abs(candidate)
		if err != nil {
			http.ServeFile(w, r, filepath.Join(staticRoot, "index.html"))
			return
		}
		if absCandidate != absRoot && !strings.HasPrefix(absCandidate, absRoot+string(os.PathSeparator)) {
			http.ServeFile(w, r, filepath.Join(staticRoot, "index.html"))
			return
		}
		if st, err := os.Stat(absCandidate); err == nil && !st.IsDir() {
			next.ServeHTTP(w, r)
			return
		}
		http.ServeFile(w, r, filepath.Join(staticRoot, "index.html"))
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

	repoPath := filepath.Join(a.reposRoot, project.RepoRel)
	execCmd := exec.CommandContext(s.Context(), gitCommand, repoPath)
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
	if strings.Contains(name, "/") || strings.Contains(name, "..") {
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
	repoPath := filepath.Join(a.reposRoot, repoRel)
	if err := os.MkdirAll(filepath.Dir(repoPath), 0o755); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to prepare repository directory")
		return
	}
	if err := exec.Command("git", "init", "--bare", repoPath).Run(); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to initialize repository")
		return
	}

	p := &Project{ID: id, Name: name, OrgID: orgID, RepoRel: repoRel}
	a.store.projects[id] = p
	a.store.issues[id] = map[int]*Issue{}
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
	issue.UpdatedAt = time.Now().UTC()
	writeJSON(w, http.StatusOK, issue)
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

func (a *app) handleGitHTTP(w http.ResponseWriter, r *http.Request) {
	pathInfo := strings.TrimPrefix(r.URL.Path, "/git")
	if pathInfo == "" || pathInfo == "/" {
		writeError(w, http.StatusNotFound, "repository path required")
		return
	}

	if a.isReceivePackRequest(r) {
		username, token, ok := r.BasicAuth()
		if !ok {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			writeError(w, http.StatusUnauthorized, "basic auth required for push")
			return
		}
		if _, err := a.authenticateHTTPGit(username, token); err != nil {
			w.Header().Set("WWW-Authenticate", `Basic realm="git"`)
			writeError(w, http.StatusUnauthorized, "invalid credentials")
			return
		}
	}

	if _, err := a.findProjectByRepoArg(pathInfo); err != nil {
		writeError(w, http.StatusNotFound, "repository not found")
		return
	}

	cmd := exec.Command("git", "http-backend")
	cmd.Env = append(os.Environ(),
		"GIT_PROJECT_ROOT="+a.reposRoot,
		"GIT_HTTP_EXPORT_ALL=1",
		"PATH_INFO="+pathInfo,
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

func (a *app) authenticateHTTPGit(username, token string) (*User, error) {
	a.store.mu.RLock()
	defer a.store.mu.RUnlock()
	for _, u := range a.store.users {
		if u.Username == username && u.Token == token {
			return u, nil
		}
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
	repo := filepath.Join(parts[0], parts[1])
	if !strings.HasSuffix(repo, ".git") {
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
		defer a.store.mu.RUnlock()
		for _, u := range a.store.users {
			if u.Token == token {
				ctx := context.WithValue(r.Context(), userContextKey{}, u.ID)
				next.ServeHTTP(w, r.WithContext(ctx))
				return
			}
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
