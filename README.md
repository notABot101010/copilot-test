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
