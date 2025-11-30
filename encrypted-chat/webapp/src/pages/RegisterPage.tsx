import { useSignal } from '@preact/signals';
import { Button, TextInput, PasswordInput, Paper, Container, Title, Alert } from '@mantine/core';
import { register, currentUser } from '../services/chatService';
import { router } from '../router';

export function RegisterPage() {
  const username = useSignal('');
  const password = useSignal('');
  const confirmPassword = useSignal('');
  const error = useSignal('');
  const loading = useSignal(false);

  async function handleSubmit(event: Event) {
    event.preventDefault();
    error.value = '';

    if (password.value !== confirmPassword.value) {
      error.value = 'Passwords do not match';
      return;
    }

    if (password.value.length < 8) {
      error.value = 'Password must be at least 8 characters';
      return;
    }

    if (username.value.length < 3) {
      error.value = 'Username must be at least 3 characters';
      return;
    }

    loading.value = true;

    try {
      await register(username.value, password.value);
      router.push('/conversations');
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Registration failed';
    } finally {
      loading.value = false;
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

        {error.value && (
          <Alert color="red" className="mb-4">
            {error.value}
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <TextInput
            label="Username"
            placeholder="Choose a username"
            value={username.value}
            onChange={(event: Event) => { username.value = (event.target as HTMLInputElement).value; }}
            required
            className="mb-4"
            size="md"
          />

          <PasswordInput
            label="Password"
            placeholder="Choose a password"
            value={password.value}
            onChange={(event: Event) => { password.value = (event.target as HTMLInputElement).value; }}
            required
            className="mb-4"
            size="md"
          />

          <PasswordInput
            label="Confirm Password"
            placeholder="Confirm your password"
            value={confirmPassword.value}
            onChange={(event: Event) => { confirmPassword.value = (event.target as HTMLInputElement).value; }}
            required
            className="mb-6"
            size="md"
          />

          <Button 
            type="submit" 
            fullWidth 
            loading={loading.value}
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
