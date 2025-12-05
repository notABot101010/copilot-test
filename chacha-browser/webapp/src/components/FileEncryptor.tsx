import { useSignal } from '@preact/signals';
import { Button, Card, FileInput, Text, Alert, Group, Stack, Code, Badge, Progress } from '@mantine/core';
import init, { ChaCha20Cipher, generate_key, generate_nonce } from 'chacha-browser';

// 1MB chunk size for streaming encryption
const CHUNK_SIZE = 1024 * 1024;

// Convert Uint8Array to hex string for display
function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export function FileEncryptor() {
  const file = useSignal<File | null>(null);
  const isLoading = useSignal(false);
  const errorMessage = useSignal<string | null>(null);
  const lastKey = useSignal<string | null>(null);
  const lastNonce = useSignal<string | null>(null);
  const wasmInitialized = useSignal(false);
  const progress = useSignal(0);

  // Initialize WASM module on first use
  const initWasm = async () => {
    if (!wasmInitialized.value) {
      await init();
      wasmInitialized.value = true;
    }
  };

  const handleFileChange = (selectedFile: File | null) => {
    file.value = selectedFile;
    errorMessage.value = null;
    lastKey.value = null;
    lastNonce.value = null;
    progress.value = 0;
  };

  const handleEncrypt = async () => {
    if (!file.value) {
      errorMessage.value = 'Please select a file first';
      return;
    }

    isLoading.value = true;
    errorMessage.value = null;
    progress.value = 0;

    try {
      // Initialize WASM if needed
      await initWasm();

      // Generate random key and nonce
      const key = generate_key();
      const nonce = generate_nonce();

      // Store key and nonce for display
      lastKey.value = toHex(key);
      lastNonce.value = toHex(nonce);

      // Create cipher for streaming encryption
      const cipher = new ChaCha20Cipher(key, nonce);

      // Process file in chunks using streaming encryption
      const fileSize = file.value.size;
      const encryptedChunks: Uint8Array[] = [];
      let processedBytes = 0;

      // Read file in chunks
      let offset = 0;
      while (offset < fileSize) {
        const chunkEnd = Math.min(offset + CHUNK_SIZE, fileSize);
        const blob = file.value.slice(offset, chunkEnd);
        const arrayBuffer = await blob.arrayBuffer();
        const chunk = new Uint8Array(arrayBuffer);

        // Encrypt this chunk using streaming cipher
        const encryptedChunk = cipher.process_chunk(chunk);
        encryptedChunks.push(encryptedChunk);

        processedBytes += chunk.length;
        progress.value = Math.round((processedBytes / fileSize) * 100);

        offset = chunkEnd;

        // Yield to UI to prevent blocking
        await new Promise((resolve) => setTimeout(resolve, 0));
      }

      // Free the cipher resources
      cipher.free();

      // Combine all encrypted chunks into a single blob
      const downloadBlob = new Blob(encryptedChunks as unknown as BlobPart[], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(downloadBlob);

      // Create download link and trigger download
      const link = document.createElement('a');
      link.href = url;
      link.download = `${file.value.name}.encrypted`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      // Clean up URL
      URL.revokeObjectURL(url);

      progress.value = 100;
    } catch (err) {
      errorMessage.value = err instanceof Error ? err.message : 'Encryption failed';
    } finally {
      isLoading.value = false;
    }
  };

  return (
    <Card shadow="md" padding="xl" radius="md" className="w-full max-w-md">
      <Stack gap="lg">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-2">ChaCha20 File Encryption</h1>
          <p className="text-gray-400">
            Encrypt files using the ChaCha20 stream cipher with WebAssembly.
            Supports large files with streaming encryption.
          </p>
        </div>

        <FileInput
          label="Select a file to encrypt"
          placeholder="Click to select file"
          value={file.value}
          onChange={handleFileChange}
          accept="*/*"
        />

        {file.value && (
          <Group gap="xs">
            <Badge variant="light" color="blue">
              {file.value.name}
            </Badge>
            <Badge variant="light" color="gray">
              {formatFileSize(file.value.size)}
            </Badge>
          </Group>
        )}

        {errorMessage.value && (
          <Alert color="red" title="Error">
            {errorMessage.value}
          </Alert>
        )}

        {isLoading.value && progress.value > 0 && (
          <div>
            <Text size="sm" mb="xs">Encrypting... {progress.value}%</Text>
            <Progress value={progress.value} animated />
          </div>
        )}

        <Button
          onClick={handleEncrypt}
          loading={isLoading.value}
          disabled={!file.value}
          fullWidth
          size="lg"
        >
          Encrypt and Download
        </Button>

        {lastKey.value && lastNonce.value && (
          <Alert color="green" title="Encryption Successful">
            <Stack gap="xs">
              <Text size="sm">
                <strong>Key (32 bytes):</strong>
              </Text>
              <Code block className="break-all text-xs">
                {lastKey.value}
              </Code>
              <Text size="sm">
                <strong>Nonce (8 bytes, DJB variant):</strong>
              </Text>
              <Code block className="break-all text-xs">
                {lastNonce.value}
              </Code>
              <Text size="xs" c="dimmed">
                Save these values to decrypt the file later!
              </Text>
            </Stack>
          </Alert>
        )}
      </Stack>
    </Card>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
