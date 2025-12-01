// MLS service using OpenMLS WebAssembly bindings
// This module provides MLS group management using RFC 9420 compliant OpenMLS

import init, {
  init_mls,
  generate_key_packages,
  create_group,
  create_invite,
  process_welcome,
  process_commit,
  encrypt_message,
  decrypt_message,
  has_group_state,
  clear_state,
} from 'openmls-wasm';

// Track initialization state
let wasmInitialized = false;
let initPromise: Promise<void> | null = null;

// Initialize the WASM module
async function ensureWasmInitialized(): Promise<void> {
  if (wasmInitialized) {
    return;
  }
  if (initPromise) {
    return initPromise;
  }
  initPromise = init().then(() => {
    wasmInitialized = true;
  });
  return initPromise;
}

// Track MLS initialization per user
let currentMlsUser: string | null = null;

// Convert bytes to base64 (kept for compatibility)
export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

// Initialize MLS for a user
export async function initializeMls(username: string): Promise<void> {
  await ensureWasmInitialized();
  
  // Don't re-initialize if already initialized for this user
  if (currentMlsUser === username) {
    return;
  }
  
  // Clear previous state if switching users
  if (currentMlsUser !== null) {
    clear_state();
  }
  
  init_mls(username);
  currentMlsUser = username;
}

// Generate key packages (RFC 9420 compliant)
export async function generateKeyPackages(username: string, count: number): Promise<string[]> {
  await initializeMls(username);
  const packages = generate_key_packages(count);
  return packages as string[];
}

// Create a new MLS group (RFC 9420 compliant)
export async function createMlsGroup(groupId: string): Promise<void> {
  await ensureWasmInitialized();
  if (!currentMlsUser) {
    throw new Error('MLS not initialized - please login first');
  }
  create_group(groupId);
  // Store in localStorage for persistence
  saveGroupState(groupId);
}

// Create welcome and commit for inviting a member (RFC 9420 compliant)
export async function createInvite(
  groupId: string,
  inviteeKeyPackage: string
): Promise<{ welcome: string; commit: string }> {
  await ensureWasmInitialized();
  const result = create_invite(groupId, inviteeKeyPackage);
  return result as { welcome: string; commit: string };
}

// Process a welcome message to join a group (RFC 9420 compliant)
export async function processWelcome(welcomeData: string): Promise<string> {
  await ensureWasmInitialized();
  const groupId = process_welcome(welcomeData);
  saveGroupState(groupId);
  return groupId;
}

// Process a commit message (RFC 9420 compliant)
export async function processCommit(commitData: string): Promise<void> {
  await ensureWasmInitialized();
  // The commit contains the group ID, so we need to extract it or handle errors
  try {
    // For commits, we need to know the group ID - in this case the server should tell us
    // This is a simplified approach - in production, you'd track this better
    const storedGroups = getStoredGroupIds();
    for (const groupId of storedGroups) {
      try {
        process_commit(groupId, commitData);
        return;
      } catch {
        // Try next group
      }
    }
  } catch {
    // Ignore commit processing errors
  }
}

// Encrypt a message for the group (RFC 9420 compliant)
export async function encryptMessage(groupId: string, plaintext: string): Promise<string> {
  await ensureWasmInitialized();
  return encrypt_message(groupId, plaintext);
}

// Decrypt a message from the group (RFC 9420 compliant)
export async function decryptMessage(groupId: string, ciphertext: string): Promise<string> {
  await ensureWasmInitialized();
  return decrypt_message(groupId, ciphertext);
}

// Check if we have state for a group
export function hasGroupState(groupId: string): boolean {
  if (!wasmInitialized) {
    return checkStoredGroupState(groupId);
  }
  return has_group_state(groupId) || checkStoredGroupState(groupId);
}

// Load all group states (for page refresh)
export function loadAllGroupStates(): void {
  // With WASM, we can't easily persist the full MLS state
  // This is a simplified approach - in production, you'd use IndexedDB
  // For now, we just track group IDs and re-join on login
}

// Clear all MLS state (for logout)
export function clearMlsState(): void {
  if (wasmInitialized) {
    clear_state();
  }
  currentMlsUser = null;
}

// Helper functions for localStorage persistence

function saveGroupState(groupId: string): void {
  const groups = getStoredGroupIds();
  if (!groups.includes(groupId)) {
    groups.push(groupId);
    localStorage.setItem('mls_group_ids', JSON.stringify(groups));
  }
  localStorage.setItem(`mls_group_${groupId}`, JSON.stringify({ groupId, active: true }));
}

function getStoredGroupIds(): string[] {
  const stored = localStorage.getItem('mls_group_ids');
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch {
      return [];
    }
  }
  return [];
}

function checkStoredGroupState(groupId: string): boolean {
  const stored = localStorage.getItem(`mls_group_${groupId}`);
  return stored !== null;
}
