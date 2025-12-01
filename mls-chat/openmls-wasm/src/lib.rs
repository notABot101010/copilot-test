//! OpenMLS WebAssembly bindings for browser-based MLS group management.
//!
//! This module provides WASM-compatible wrappers around OpenMLS functionality
//! for use in web applications.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};
use wasm_bindgen::prelude::*;

// Global crypto provider
static CRYPTO_PROVIDER: std::sync::OnceLock<OpenMlsRustCrypto> = std::sync::OnceLock::new();

fn get_crypto() -> &'static OpenMlsRustCrypto {
    CRYPTO_PROVIDER.get_or_init(OpenMlsRustCrypto::default)
}

// Global state storage (in-memory for WASM)
struct MlsState {
    identity: Option<Identity>,
    groups: HashMap<String, MlsGroupWrapper>,
    key_packages: Vec<KeyPackage>,
}

struct Identity {
    credential_with_key: CredentialWithKey,
    signer: SignatureKeyPair,
}

struct MlsGroupWrapper {
    group: MlsGroup,
}

// Safety: We're in single-threaded WASM environment
unsafe impl Send for MlsState {}
unsafe impl Sync for MlsState {}

static STATE: std::sync::OnceLock<Mutex<MlsState>> = std::sync::OnceLock::new();

fn get_state() -> &'static Mutex<MlsState> {
    STATE.get_or_init(|| {
        Mutex::new(MlsState {
            identity: None,
            groups: HashMap::new(),
            key_packages: Vec::new(),
        })
    })
}

#[derive(Serialize, Deserialize)]
pub struct KeyPackageBundle {
    pub key_package: String,
}

#[derive(Serialize, Deserialize)]
pub struct InviteResult {
    pub welcome: String,
    pub commit: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsError {
    pub error: String,
}

/// Initialize the MLS client with a username
#[wasm_bindgen]
pub fn init_mls(username: &str) -> Result<(), JsError> {
    console_error_panic_hook::set_once();
    
    let crypto = get_crypto();
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
    
    // Generate a new signature key pair
    let signature_keypair = SignatureKeyPair::new(ciphersuite.signature_algorithm())
        .map_err(|e| JsError { error: format!("Failed to create signature keypair: {:?}", e) })?;
    signature_keypair.store(crypto.storage())
        .map_err(|e| JsError { error: format!("Failed to store signature keypair: {:?}", e) })?;
    
    // Create a basic credential with the username
    let credential = Credential::new(CredentialType::Basic, username.as_bytes().to_vec());
    
    let credential_with_key = CredentialWithKey {
        credential,
        signature_key: signature_keypair.public().into(),
    };
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    state.identity = Some(Identity {
        credential_with_key,
        signer: signature_keypair,
    });
    
    Ok(())
}

/// Generate key packages for the client
#[wasm_bindgen]
pub fn generate_key_packages(count: u32) -> Result<JsValue, JsError> {
    let crypto = get_crypto();
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    // Clone identity data to avoid borrow issues
    let (signer, credential_with_key) = {
        let identity = state.identity.as_ref()
            .ok_or_else(|| JsError { error: "MLS not initialized".to_string() })?;
        (identity.signer.clone(), identity.credential_with_key.clone())
    };
    
    let mut packages: Vec<String> = Vec::new();
    
    for _ in 0..count {
        let key_package_bundle = KeyPackage::builder()
            .build(
                ciphersuite,
                crypto,
                &signer,
                credential_with_key.clone(),
            )
            .map_err(|e| JsError { error: format!("Failed to create key package: {:?}", e) })?;
        
        let key_package = key_package_bundle.key_package().clone();
        let serialized = key_package.tls_serialize_detached()
            .map_err(|e| JsError { error: format!("Failed to serialize key package: {:?}", e) })?;
        
        packages.push(BASE64.encode(&serialized));
        state.key_packages.push(key_package);
    }
    
    serde_wasm_bindgen::to_value(&packages)
        .map_err(|e| JsError { error: format!("Serialization error: {}", e) })
}

/// Create a new MLS group
#[wasm_bindgen]
pub fn create_group(group_id: &str) -> Result<(), JsError> {
    let crypto = get_crypto();
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    let identity = state.identity.as_ref()
        .ok_or_else(|| JsError { error: "MLS not initialized".to_string() })?;
    
    let mls_group_config = MlsGroupCreateConfig::builder()
        .use_ratchet_tree_extension(true)
        .ciphersuite(ciphersuite)
        .build();
    
    let group = MlsGroup::new_with_group_id(
        crypto,
        &identity.signer,
        &mls_group_config,
        GroupId::from_slice(group_id.as_bytes()),
        identity.credential_with_key.clone(),
    )
    .map_err(|e| JsError { error: format!("Failed to create group: {:?}", e) })?;
    
    state.groups.insert(group_id.to_string(), MlsGroupWrapper { group });
    
    Ok(())
}

/// Create an invitation for a new member
#[wasm_bindgen]
pub fn create_invite(group_id: &str, invitee_key_package_b64: &str) -> Result<JsValue, JsError> {
    let crypto = get_crypto();
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    // Clone needed data first to avoid borrow issues
    let signer = state.identity.as_ref()
        .ok_or_else(|| JsError { error: "MLS not initialized".to_string() })?
        .signer.clone();
    
    let group_wrapper = state.groups.get_mut(group_id)
        .ok_or_else(|| JsError { error: "Group not found".to_string() })?;
    
    // Deserialize the invitee's key package
    let kp_bytes = BASE64.decode(invitee_key_package_b64)
        .map_err(|e| JsError { error: format!("Invalid base64: {}", e) })?;
    let key_package_in = KeyPackageIn::tls_deserialize_exact(&kp_bytes)
        .map_err(|e| JsError { error: format!("Invalid key package format: {:?}", e) })?;
    let key_package = key_package_in.validate(crypto.crypto(), ProtocolVersion::Mls10)
        .map_err(|e| JsError { error: format!("Key package validation failed: {:?}", e) })?;
    
    // Add the member to the group
    let (commit, welcome, _group_info) = group_wrapper.group.add_members(
        crypto,
        &signer,
        &[key_package],
    )
    .map_err(|e| JsError { error: format!("Failed to add member: {:?}", e) })?;
    
    // Merge the pending commit
    group_wrapper.group.merge_pending_commit(crypto)
        .map_err(|e| JsError { error: format!("Failed to merge commit: {:?}", e) })?;
    
    // Serialize welcome and commit
    let welcome_bytes = welcome.tls_serialize_detached()
        .map_err(|e| JsError { error: format!("Failed to serialize welcome: {:?}", e) })?;
    let commit_bytes = commit.tls_serialize_detached()
        .map_err(|e| JsError { error: format!("Failed to serialize commit: {:?}", e) })?;
    
    let result = InviteResult {
        welcome: BASE64.encode(&welcome_bytes),
        commit: BASE64.encode(&commit_bytes),
    };
    
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError { error: format!("Serialization error: {}", e) })
}

/// Process a welcome message to join a group
#[wasm_bindgen]
pub fn process_welcome(welcome_b64: &str) -> Result<String, JsError> {
    let crypto = get_crypto();
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    // Deserialize the welcome message
    let welcome_bytes = BASE64.decode(welcome_b64)
        .map_err(|e| JsError { error: format!("Invalid base64: {}", e) })?;
    let welcome = MlsMessageIn::tls_deserialize_exact(&welcome_bytes)
        .map_err(|e| JsError { error: format!("Invalid welcome format: {:?}", e) })?
        .into_welcome()
        .ok_or_else(|| JsError { error: "Not a welcome message".to_string() })?;
    
    let mls_group_config = MlsGroupJoinConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();
    
    // Process the welcome to join the group
    let staged_join = StagedWelcome::new_from_welcome(
        crypto,
        &mls_group_config,
        welcome,
        None,
    )
    .map_err(|e| JsError { error: format!("Failed to process welcome: {:?}", e) })?;
    
    let group = staged_join.into_group(crypto)
        .map_err(|e| JsError { error: format!("Failed to join group: {:?}", e) })?;
    
    let group_id = String::from_utf8(group.group_id().as_slice().to_vec())
        .unwrap_or_else(|_| BASE64.encode(group.group_id().as_slice()));
    
    state.groups.insert(group_id.clone(), MlsGroupWrapper { group });
    
    Ok(group_id)
}

/// Process a commit message to update group state
#[wasm_bindgen]
pub fn process_commit(group_id: &str, commit_b64: &str) -> Result<(), JsError> {
    let crypto = get_crypto();
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    let group_wrapper = state.groups.get_mut(group_id)
        .ok_or_else(|| JsError { error: "Group not found".to_string() })?;
    
    // Deserialize the commit message
    let commit_bytes = BASE64.decode(commit_b64)
        .map_err(|e| JsError { error: format!("Invalid base64: {}", e) })?;
    let message_in = MlsMessageIn::tls_deserialize_exact(&commit_bytes)
        .map_err(|e| JsError { error: format!("Invalid commit format: {:?}", e) })?;
    
    // Process the message
    let protocol_message = message_in.try_into_protocol_message()
        .map_err(|e| JsError { error: format!("Not a protocol message: {:?}", e) })?;
    
    let processed = group_wrapper.group.process_message(crypto, protocol_message)
        .map_err(|e| JsError { error: format!("Failed to process message: {:?}", e) })?;
    
    if let ProcessedMessageContent::StagedCommitMessage(staged_commit) = processed.into_content() {
        group_wrapper.group.merge_staged_commit(crypto, *staged_commit)
            .map_err(|e| JsError { error: format!("Failed to merge commit: {:?}", e) })?;
    }
    
    Ok(())
}

/// Encrypt a message for the group
#[wasm_bindgen]
pub fn encrypt_message(group_id: &str, plaintext: &str) -> Result<String, JsError> {
    let crypto = get_crypto();
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    // Clone the signer first
    let signer = state.identity.as_ref()
        .ok_or_else(|| JsError { error: "MLS not initialized".to_string() })?
        .signer.clone();
    
    let group_wrapper = state.groups.get_mut(group_id)
        .ok_or_else(|| JsError { error: "Group not found".to_string() })?;
    
    // Create an application message
    let mls_message = group_wrapper.group.create_message(
        crypto,
        &signer,
        plaintext.as_bytes(),
    )
    .map_err(|e| JsError { error: format!("Failed to encrypt message: {:?}", e) })?;
    
    let serialized = mls_message.tls_serialize_detached()
        .map_err(|e| JsError { error: format!("Failed to serialize message: {:?}", e) })?;
    
    Ok(BASE64.encode(&serialized))
}

/// Decrypt a message from the group
#[wasm_bindgen]
pub fn decrypt_message(group_id: &str, ciphertext_b64: &str) -> Result<String, JsError> {
    let crypto = get_crypto();
    
    let mut state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    let group_wrapper = state.groups.get_mut(group_id)
        .ok_or_else(|| JsError { error: "Group not found".to_string() })?;
    
    // Deserialize the message
    let message_bytes = BASE64.decode(ciphertext_b64)
        .map_err(|e| JsError { error: format!("Invalid base64: {}", e) })?;
    let message_in = MlsMessageIn::tls_deserialize_exact(&message_bytes)
        .map_err(|e| JsError { error: format!("Invalid message format: {:?}", e) })?;
    
    // Process the message
    let protocol_message = message_in.try_into_protocol_message()
        .map_err(|e| JsError { error: format!("Not a protocol message: {:?}", e) })?;
    
    let processed = group_wrapper.group.process_message(crypto, protocol_message)
        .map_err(|e| JsError { error: format!("Failed to process message: {:?}", e) })?;
    
    // Extract the application message content
    match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(app_message) => {
            String::from_utf8(app_message.into_bytes())
                .map_err(|e| JsError { error: format!("Invalid UTF-8 in message: {}", e) })
        }
        ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
            // This is a commit message, merge it and return empty
            group_wrapper.group.merge_staged_commit(crypto, *staged_commit)
                .map_err(|e| JsError { error: format!("Failed to merge commit: {:?}", e) })?;
            Err(JsError { error: "Received commit message, not application message".to_string() })
        }
        _ => Err(JsError { error: "Unexpected message type".to_string() }),
    }
}

/// Check if we have group state for a given group ID
#[wasm_bindgen]
pub fn has_group_state(group_id: &str) -> bool {
    get_state()
        .lock()
        .map(|state| state.groups.contains_key(group_id))
        .unwrap_or(false)
}

/// Clear all MLS state (for logout)
#[wasm_bindgen]
pub fn clear_state() {
    if let Ok(mut state) = get_state().lock() {
        state.identity = None;
        state.groups.clear();
        state.key_packages.clear();
    }
}

/// Save state to localStorage
#[wasm_bindgen]
pub fn save_state_to_storage(username: &str) -> Result<(), JsError> {
    let state = get_state().lock().map_err(|e| JsError { error: format!("Lock error: {}", e) })?;
    
    // We can't easily serialize MlsGroup, so we'll save minimal info
    // In a real app, you'd use openmls_memory_storage or similar
    let group_ids: Vec<String> = state.groups.keys().cloned().collect();
    let storage_data = serde_json::json!({
        "username": username,
        "group_ids": group_ids,
    });
    
    let window = web_sys::window()
        .ok_or_else(|| JsError { error: "No window object".to_string() })?;
    let storage = window.local_storage()
        .map_err(|_| JsError { error: "Failed to access localStorage".to_string() })?
        .ok_or_else(|| JsError { error: "No localStorage".to_string() })?;
    
    storage.set_item(
        &format!("mls_state_{}", username),
        &storage_data.to_string()
    ).map_err(|_| JsError { error: "Failed to save to localStorage".to_string() })?;
    
    Ok(())
}

impl From<JsError> for JsValue {
    fn from(err: JsError) -> Self {
        JsValue::from_str(&err.error)
    }
}
