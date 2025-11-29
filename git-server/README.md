# Git Server

A Git server over SSH implemented in Rust using the `russh` crate, with a web UI for browsing repositories.

## Features

- SSH-based Git server for hosting repositories
- Public key authentication using OpenSSH format keys
- SQLite database for repository metadata storage
- CLI for server management
- **Web UI** for browsing repositories, files, and commits
- **REST API** for programmatic access
- **Basic authentication** for web access

## Configuration

Create a `config.json` file with the following structure:

```json
{
    "ssh_port": 2222,
    "http_port": 8080,
    "public_keys": [
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample... user@example.com"
    ],
    "auth": [
        {
            "user": "admin",
            "password_hash": "ba3253876aed6bc22d4a6ff53d8406c6ad864195ed144ab5c87621b6c233b548baeae6956df346ec8c17f5ea10f35ee3cbc514797ed7ddd3145464e2a0bab413"
        }
    ]
}
```

### Configuration Options

- `ssh_port`: The port number the SSH server listens on (default: 2222)
- `http_port`: The port number the HTTP server listens on (default: 8080)
- `public_keys`: Array of public keys in OpenSSH format that are authorized to access the server
- `auth`: Array of user credentials for HTTP basic authentication (optional)
  - `user`: Username
  - `password_hash`: SHA512 hash of the password (hex encoded)

### Generating Password Hash

To generate a SHA512 hash for a password, use:

```bash
echo -n "your_password" | sha512sum | cut -d' ' -f1
```

If no `auth` entries are configured, the web UI will be accessible without authentication.

## Usage

### Create a Repository

```bash
git-server create-repo <name>
```

Creates a new bare Git repository with the given name.

### Start the Server

```bash
git-server serve
```

Starts the Git SSH server and HTTP web server using the configuration from `config.json`.

### CLI Options

```
git-server [OPTIONS] <COMMAND>

Commands:
  create-repo  Create a new repository
  serve        Start the git server
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>          Path to the configuration file [default: config.json]
  -r, --repos-path <REPOS_PATH>  Path to the repositories directory [default: repos]
  -d, --database <DATABASE>      Path to the SQLite database [default: git-server.db]
  -h, --help                     Print help
```

## Connecting to the Server

Once the server is running, you can clone and push to repositories using SSH:

```bash
# Clone a repository
git clone ssh://git@localhost:2222/myrepo

# Or add as a remote
git remote add origin ssh://git@localhost:2222/myrepo
```

## Web UI

Access the web UI at `http://localhost:8080` (or your configured HTTP port).

### Features

- **Home page** (`/`): Lists all repositories
- **Repository view** (`/repos/{name}`): Browse files and commits
  - Toggle between Files and Commits tabs
  - Navigate directory structure
  - View file contents
- **Commit browsing**: Click on a commit hash to browse the repository at that point in time

## REST API

The server provides a REST API for programmatic access:

### Endpoints

- `GET /api/repos` - List all repositories
- `GET /api/repos/{name}` - Get repository details
- `GET /api/repos/{name}/files` - List files in root directory
- `GET /api/repos/{name}/commits` - List recent commits
- `GET /api/repos/{name}/tree?ref={ref}&path={path}` - Browse tree at ref/path
- `GET /api/repos/{name}/tree/{path}?ref={ref}` - Browse tree at path
- `GET /api/repos/{name}/blob/{path}?ref={ref}` - Get file content

### Authentication

If `auth` is configured, API requests require HTTP Basic Authentication.

## Security

The server implements the following security measures:

- Public key authentication only for SSH (no password auth)
- Path traversal protection for repository access
- Repository paths are validated against the configured repos directory
- Git refs are validated to prevent command injection
- Optional HTTP Basic Authentication with SHA512 password hashing
