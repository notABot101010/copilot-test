import { useState } from 'preact/hooks';
import { Button, TextInput, PasswordInput, Paper, Container, Title, Alert } from '@mantine/core';
import { register, currentUser } from '../services/chatService';
import { router } from '../router';

export function RegisterPage() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(event: Event) {
    event.preventDefault();
    setError('');

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    if (username.length < 3) {
      setError('Username must be at least 3 characters');
      return;
    }

    setLoading(true);

    try {
      await register(username, password);
      router.push('/conversations');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Registration failed');
    } finally {
      setLoading(false);
    }
  }

  // Redirect if already logged in
  if (currentUser.value) {
    router.push('/conversations');
    return null;
  }

  return (
    <Container size="xs" className="min-h-screen flex items-center justify-center">
      <Paper shadow="md" p="xl" radius="md" className="w-full">
        <Title order={1} className="text-center mb-6">Encrypted Chat</Title>
        <Title order={3} className="text-center mb-6 text-gray-400">Create Account</Title>

        {error && (
          <Alert color="red" className="mb-4">
            {error}
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <TextInput
            label="Username"
            placeholder="Choose a username"
            value={username}
            onChange={(event: Event) => setUsername((event.target as HTMLInputElement).value)}
            required
            className="mb-4"
            size="md"
          />

          <PasswordInput
            label="Password"
            placeholder="Choose a password"
            value={password}
            onChange={(event: Event) => setPassword((event.target as HTMLInputElement).value)}
            required
            className="mb-4"
            size="md"
          />

          <PasswordInput
            label="Confirm Password"
            placeholder="Confirm your password"
            value={confirmPassword}
            onChange={(event: Event) => setConfirmPassword((event.target as HTMLInputElement).value)}
            required
            className="mb-6"
            size="md"
          />

          <Button 
            type="submit" 
            fullWidth 
            loading={loading}
            size="md"
          >
            Create Account
          </Button>
        </form>

        <p className="text-center mt-4 text-gray-400">
          Already have an account?{' '}
          <a href="/" className="text-blue-400 hover:underline">
            Login
          </a>
        </p>
      </Paper>
    </Container>
  );
}
