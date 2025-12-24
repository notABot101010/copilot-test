# TUI Password Manager

A terminal-based password manager built with Rust and Ratatui, featuring encrypted vaults and a modern cryptographic architecture.

## Features

- 🔐 Strong encryption using modern cryptographic algorithms
- 🖥️ Beautiful terminal user interface with mouse support
- 📁 Encrypted vault files for secure password storage
- ⌨️ Keyboard shortcuts for efficient navigation
- 🔑 Password visibility toggle
- ➕ Add, edit, and delete credentials
- 💾 Auto-save on quit

## Security

This password manager uses industry-standard cryptographic algorithms:

- **Argon2id** for password-based key derivation (PBKDF)
- **ChaCha20-Poly1305** for authenticated encryption
- **BLAKE3** for key derivation and MAC

See [CRYPTOGRAPHIC_ARCHITECTURE.md](./CRYPTOGRAPHIC_ARCHITECTURE.md) for detailed security documentation.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/tui-pass`.

## Usage

### Create a new vault

```bash
tui-pass create my-passwords.vault
```

You'll be prompted to enter and confirm a master password.

### Open an existing vault

```bash
tui-pass my-passwords.vault
```

Enter your master password when prompted.

## Keyboard Shortcuts

### Navigation
- **↑/↓**: Navigate up/down in the credential list
- **Enter**: Select a credential (if none selected)
- **Mouse Click**: Click on a credential to select it
- **Mouse Scroll**: Scroll the credential list

### Actions
- **Space**: Toggle password visibility (show/hide)
- **c**: Copy Mode - Temporarily exit TUI to allow text selection with mouse (press any key to return)
- **a**: Add a new credential
- **e**: Edit the selected credential
- **d**: Delete the selected credential
- **s**: Save the vault
- **q**: Quit (auto-saves if modified)

### In Dialogs
- **Tab**: Next field
- **Shift+Tab**: Previous field
- **↑/↓**: Navigate between fields
- **Enter**: Save
- **Esc**: Cancel
- **Mouse Click**: Click on a field to activate it and position cursor

### Confirmation Dialogs
- **Y**: Confirm action
- **N**: Cancel action

## Text Selection and Copying

To copy passwords or other credential information:
1. Select a credential from the list
2. Press **c** to enter Copy Mode
3. The TUI will temporarily exit, showing the credential details in plain text
4. Use your mouse to select and copy the text you need
5. Press any key to return to the TUI

This feature allows you to easily copy passwords to your clipboard using standard terminal text selection.

## Interface Layout

```
┌─────────────────────┬──────────────────────────────────┐
│  Credentials        │  Details                         │
│                     │                                  │
│  → Gmail           │  Title: Gmail                    │
│    GitHub          │  Username: user@example.com      │
│    Dropbox         │  Password: ••••••••••• (hidden)  │
│                     │  URL: https://gmail.com          │
│                     │  Notes: My main email account    │
│                     │                                  │
└─────────────────────┴──────────────────────────────────┘
 ↑/↓: Navigate | Space: Toggle Password | a: Add | q: Quit
```

## Data Format

Credentials are stored in an encrypted vault file with the following structure:
- Each vault is protected by a master password
- Vaults use a binary format with cryptographic metadata in the header
- The payload is encrypted using ChaCha20-Poly1305 with authentication

### Credential Fields
- **Title**: Name or description of the credential
- **Username**: Login username or email
- **Password**: The actual password
- **URL**: Website or service URL
- **Notes**: Additional notes or information

## Security Best Practices

1. **Use a strong master password**: At least 16 characters recommended
2. **Keep vault files secure**: Store in encrypted storage
3. **Don't share your master password**: Each user should have their own vault
4. **Regular backups**: Keep encrypted backups of your vault files
5. **Unique passwords**: Use different passwords for each service

## Development

### Run tests

```bash
cargo test
```

### Run with debug logging

```bash
RUST_LOG=debug cargo run -- my-vault.vault
```

## License

MIT
