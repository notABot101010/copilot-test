// Simplified MLS-like service
// In a production app, this would use OpenMLS via WebAssembly
// For this demo, we use a simplified approach

// Storage for MLS state
interface MlsGroupState {
  groupId: string;
  groupSecret: string;
  epoch: number;
}

const groupStates: Map<string, MlsGroupState> = new Map();

// Generate random bytes
function randomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return bytes;
}

// Convert bytes to base64
function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

// Convert base64 to bytes (exported for potential future use)
export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

// Generate key packages (simulated)
export async function generateKeyPackages(username: string, count: number): Promise<string[]> {
  const packages: string[] = [];
  for (let i = 0; i < count; i++) {
    // Create a simulated key package
    const keyPackage = {
      version: 1,
      username,
      publicKey: bytesToBase64(randomBytes(32)),
      signature: bytesToBase64(randomBytes(64)),
      timestamp: Date.now(),
      index: i,
    };
    packages.push(btoa(JSON.stringify(keyPackage)));
  }
  return packages;
}

// Create a new MLS group
export async function createMlsGroup(groupId: string): Promise<void> {
  const groupSecret = bytesToBase64(randomBytes(32));
  groupStates.set(groupId, {
    groupId,
    groupSecret,
    epoch: 0,
  });
  // Store in localStorage for persistence
  saveGroupState(groupId);
}

// Create welcome and commit for inviting a member
export async function createInvite(
  groupId: string,
  _inviteeKeyPackage: string
): Promise<{ welcome: string; commit: string }> {
  const state = groupStates.get(groupId);
  if (!state) {
    throw new Error('Group not found');
  }

  // Create welcome message with group secret (simplified)
  const welcome = {
    version: 1,
    groupId,
    groupSecret: state.groupSecret,
    epoch: state.epoch + 1,
  };

  // Create commit message (simplified)
  const commit = {
    version: 1,
    groupId,
    epoch: state.epoch + 1,
    action: 'add',
  };

  // Update epoch
  state.epoch += 1;
  saveGroupState(groupId);

  return {
    welcome: btoa(JSON.stringify(welcome)),
    commit: btoa(JSON.stringify(commit)),
  };
}

// Process a welcome message to join a group
export async function processWelcome(welcomeData: string): Promise<string> {
  const welcome = JSON.parse(atob(welcomeData));
  groupStates.set(welcome.groupId, {
    groupId: welcome.groupId,
    groupSecret: welcome.groupSecret,
    epoch: welcome.epoch,
  });
  saveGroupState(welcome.groupId);
  return welcome.groupId;
}

// Process a commit message
export async function processCommit(commitData: string): Promise<void> {
  const commit = JSON.parse(atob(commitData));
  const state = groupStates.get(commit.groupId);
  if (state && commit.epoch > state.epoch) {
    state.epoch = commit.epoch;
    saveGroupState(commit.groupId);
  }
}

// Encrypt a message for the group
export async function encryptMessage(groupId: string, plaintext: string): Promise<string> {
  const state = groupStates.get(groupId);
  if (!state) {
    // Try to load from storage
    loadGroupState(groupId);
    const loadedState = groupStates.get(groupId);
    if (!loadedState) {
      throw new Error('Group not found');
    }
  }

  // Simplified encryption (in production, use proper AEAD)
  const message = {
    ciphertext: btoa(plaintext),
    epoch: groupStates.get(groupId)?.epoch || 0,
    nonce: bytesToBase64(randomBytes(12)),
  };

  return btoa(JSON.stringify(message));
}

// Decrypt a message from the group
export async function decryptMessage(groupId: string, ciphertext: string): Promise<string> {
  const state = groupStates.get(groupId);
  if (!state) {
    loadGroupState(groupId);
  }

  try {
    const message = JSON.parse(atob(ciphertext));
    return atob(message.ciphertext);
  } catch {
    return '[Unable to decrypt]';
  }
}

// Save group state to localStorage
function saveGroupState(groupId: string): void {
  const state = groupStates.get(groupId);
  if (state) {
    localStorage.setItem(`mls_group_${groupId}`, JSON.stringify(state));
  }
}

// Load group state from localStorage
function loadGroupState(groupId: string): void {
  const stored = localStorage.getItem(`mls_group_${groupId}`);
  if (stored) {
    const state = JSON.parse(stored);
    groupStates.set(groupId, state);
  }
}

// Load all group states from localStorage
export function loadAllGroupStates(): void {
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith('mls_group_')) {
      const groupId = key.replace('mls_group_', '');
      loadGroupState(groupId);
    }
  }
}

// Check if we have state for a group
export function hasGroupState(groupId: string): boolean {
  if (groupStates.has(groupId)) {
    return true;
  }
  loadGroupState(groupId);
  return groupStates.has(groupId);
}
