import { useState } from 'preact/hooks';
import { useRouter } from '@copilot-test/preact-router';
import { Button, TextInput, Paper, Title, Alert } from '@mantine/core';
import { register as apiRegister } from '../api/client';
import { initializeUserKeys } from '../crypto/storage';

export default function Register() {
  const router = useRouter();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError('');

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    setLoading(true);

    try {
      const keys = await initializeUserKeys(username, password);

      await apiRegister(
        username,
        password,
        keys.encryptedIdentityKey,
        keys.identityPublicKey,
        keys.prekeySignature
      );

      router.push('/login');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Registration failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="flex items-center justify-center min-h-screen p-4">
      <Paper class="w-full max-w-md p-8 shadow-lg">
        <Title order={2} class="mb-6 text-center">Create Account</Title>

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

          <TextInput
            type="password"
            label="Confirm Password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword((e.target as HTMLInputElement).value)}
            required
            disabled={loading}
          />

          <Button type="submit" fullWidth loading={loading}>
            Register
          </Button>

          <div class="text-center mt-4">
            <a href="/login" class="text-blue-600 hover:text-blue-800">
              Already have an account? Login
            </a>
          </div>
        </form>
      </Paper>
    </div>
  );
}
