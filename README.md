# Git Hosting Platform Monorepo

This monorepo contains:
- `backend/` — Go service for API + Git over HTTP/SSH
- `frontend/` — React UI using Mantine, React Router, and Preact signals

## Features

- Two listener ports:
  - HTTP API + Git HTTP: `8080`
  - Git SSH: `2222`
- User creation with token auth
- Add SSH public keys to user accounts
- Organizations and projects (`1 project = 1 bare Git repository`)
- Git push support:
  - HTTP (`git-http-backend`, basic auth with username + token)
  - SSH (public key auth)
- Git LFS support over HTTP and SSH:
  - LFS Batch API (`POST /git/<org>/<project>.git/info/lfs/objects/batch`)
  - LFS object upload (`PUT /git/<org>/<project>.git/info/lfs/objects/<oid>`)
  - LFS object download (`GET /git/<org>/<project>.git/info/lfs/objects/<oid>`)
  - SSH `git-lfs-authenticate` for credential handoff
- Issue tracking per project:
  - create/list/update issues
  - add comments

## Run locally

### Backend

```bash
cd /home/runner/work/copilot-test/copilot-test/backend
go run ./cmd/server
```

### Frontend

```bash
cd /home/runner/work/copilot-test/copilot-test/frontend
npm install
npm run dev
```

The UI expects backend API at `http://localhost:8080` by default.
You can change it with `VITE_API_BASE`.

## Docker

Build and run:

```bash
cd /home/runner/work/copilot-test/copilot-test
docker build -t git-platform .
docker run --rm -p 8080:8080 -p 2222:2222 git-platform
```

## Example flow

1. Create a user in the UI (copy token from notification).
2. Add an SSH public key.
3. Create organization + project.
4. Push over HTTP:
   - remote URL: `http://localhost:8080/git/<org>/<project>.git`
   - username: your username
   - password: token
5. Push over SSH:
   - remote URL: `ssh://git@localhost:2222/<org>/<project>.git`

## Git transport hardening notes

- Git operations are executed via the resolved system `git` binary (`exec.LookPath("git")`) with an explicit command allowlist.
- Repository paths are validated and constrained to stay under `REPOS_ROOT` before any Git command executes.
- SSH and HTTP push (`receive-pack`) enforce authorization so only the owning organization user can write to a project repo.
- Git subprocesses run with context deadlines to avoid unbounded command execution.

## Git LFS

Git LFS objects are stored per-repository on the server filesystem at
`<REPOS_ROOT>/<org>/<project>.git/lfs/objects/<oid[0:2]>/<oid[2:4]>/<oid>`.

The HTTP base URL used in LFS batch responses defaults to `http://localhost:8080` and
can be overridden with the `HTTP_BASE_URL` environment variable (useful behind a reverse proxy).

Configure your local git-lfs client:

```bash
git lfs install
git lfs track "*.bin"   # or whatever file types you want to store in LFS
git add .gitattributes
git push
```

LFS objects are pushed automatically by `git push` when git-lfs is installed.
