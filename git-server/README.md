# Git Server

A Git server over SSH implemented in Rust using the `russh` crate.

## Features

- SSH-based Git server for hosting repositories
- Public key authentication using OpenSSH format keys
- SQLite database for repository metadata storage
- CLI for server management

## Configuration

Create a `config.json` file with the following structure:

```json
{
    "ssh_port": 2222,
    "public_keys": [
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample... user@example.com"
    ]
}
```

### Configuration Options

- `ssh_port`: The port number the SSH server listens on (default: 2222)
- `public_keys`: Array of public keys in OpenSSH format that are authorized to access the server

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

Starts the Git SSH server using the configuration from `config.json`.

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

## Security

The server implements the following security measures:

- Public key authentication only (no password auth)
- Path traversal protection for repository access
- Repository paths are validated against the configured repos directory
