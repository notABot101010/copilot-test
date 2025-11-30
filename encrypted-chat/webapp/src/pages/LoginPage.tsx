import { useSignal } from '@preact/signals';
import { Button, TextInput, PasswordInput, Paper, Container, Title, Alert } from '@mantine/core';
import { login, currentUser } from '../services/chatService';
import { router } from '../router';

export function LoginPage() {
  const username = useSignal('');
  const password = useSignal('');
  const error = useSignal('');
  const loading = useSignal(false);

  async function handleSubmit(event: Event) {
    event.preventDefault();
    error.value = '';
    loading.value = true;

    try {
      await login(username.value, password.value);
      router.push('/conversations');
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed';
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
        <Title order={3} className="text-center mb-6 text-gray-400">Login</Title>

        {error.value && (
          <Alert color="red" className="mb-4">
            {error.value}
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <TextInput
            label="Username"
            placeholder="Enter your username"
            value={username.value}
            onChange={(event: Event) => { username.value = (event.target as HTMLInputElement).value; }}
            required
            className="mb-4"
            size="md"
          />

          <PasswordInput
            label="Password"
            placeholder="Enter your password"
            value={password.value}
            onChange={(event: Event) => { password.value = (event.target as HTMLInputElement).value; }}
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
            Login
          </Button>
        </form>

        <p className="text-center mt-4 text-gray-400">
          Don't have an account?{' '}
          <a href="/register" className="text-blue-400 hover:underline">
            Register
          </a>
        </p>
      </Paper>
    </Container>
  );
}
