import { useState } from 'preact/hooks';
import { useRouter } from '@copilot-test/preact-router';
import { Button, TextInput, Paper, Title, Alert } from '@mantine/core';
import { login as apiLogin } from '../api/client';
import { loadUserKeys, setCurrentUser } from '../crypto/storage';
import { currentUser } from '../app';

export default function Login() {
  const router = useRouter();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await apiLogin(username, password);

      // Load and decrypt user keys
      await loadUserKeys(username, password, response.encrypted_identity_key);

      // Set current user session
      setCurrentUser(username);
      currentUser.value = username;

      router.push('/chat');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="flex items-center justify-center min-h-screen p-4">
      <Paper class="w-full max-w-md p-8 shadow-lg">
        <Title order={2} class="mb-6 text-center">Login</Title>

        {error && (
          <Alert color="red" class="mb-4">
            {error}
          </Alert>
        )}

        <form onSubmit={handleSubmit} class="space-y-4">
          <TextInput
            label="Username"
            value={username}
            onChange={(e) => setUsername((e.target as HTMLInputElement).value)}
            required
            disabled={loading}
          />

          <TextInput
            type="password"
            label="Password"
            value={password}
            onChange={(e) => setPassword((e.target as HTMLInputElement).value)}
            required
            disabled={loading}
          />

          <Button type="submit" fullWidth loading={loading}>
            Login
          </Button>

          <div class="text-center mt-4">
            <a href="/register" class="text-blue-600 hover:text-blue-800">
              Don't have an account? Register
            </a>
          </div>
        </form>
      </Paper>
    </div>
  );
}
