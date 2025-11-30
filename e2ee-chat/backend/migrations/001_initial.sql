-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    encrypted_identity_key TEXT NOT NULL,
    identity_public_key TEXT NOT NULL,
    prekey_signature TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_username ON users(username);

-- Messages table (stores encrypted messages)
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_user TEXT NOT NULL,
    to_user TEXT NOT NULL,
    encrypted_content TEXT NOT NULL,
    ephemeral_public_key TEXT NOT NULL,
    sender_identity_key TEXT,
    sender_signature TEXT,
    message_number INTEGER NOT NULL DEFAULT 0,
    previous_chain_length INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    delivered_at DATETIME
);

CREATE INDEX idx_messages_to_user ON messages(to_user, delivered_at);
CREATE INDEX idx_messages_conversation ON messages(from_user, to_user);

-- Ratchet state table (stores double ratchet state for each conversation)
CREATE TABLE IF NOT EXISTS ratchet_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    peer_username TEXT NOT NULL,
    root_key TEXT NOT NULL,
    chain_key_send TEXT,
    chain_key_receive TEXT,
    sending_chain_length INTEGER NOT NULL DEFAULT 0,
    receiving_chain_length INTEGER NOT NULL DEFAULT 0,
    previous_sending_chain_length INTEGER NOT NULL DEFAULT 0,
    public_key_send TEXT,
    public_key_receive TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, peer_username)
);

CREATE INDEX idx_ratchet_user_peer ON ratchet_states(user_id, peer_username);

-- One-time prekeys for initial key exchange
CREATE TABLE IF NOT EXISTS prekeys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    public_key TEXT NOT NULL,
    key_id INTEGER NOT NULL,
    used BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_prekeys_user_unused ON prekeys(user_id, used);
