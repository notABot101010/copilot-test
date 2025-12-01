import { useSignal } from '@preact/signals';
import { login, register } from '../store/authStore';
import { Button, TextInput, PasswordInput, Card } from '@mantine/core';
import { useRouter } from '@copilot-test/preact-router';

export function LoginPage() {
  const router = useRouter();
  const username = useSignal('');
  const password = useSignal('');
  const isRegistering = useSignal(false);
  const error = useSignal('');
  const loading = useSignal(false);
  
  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    error.value = '';
    loading.value = true;
    
    try {
      let success: boolean;
      if (isRegistering.value) {
        success = await register(username.value, password.value);
        if (!success) {
          error.value = 'Registration failed. Username may already exist.';
        }
      } else {
        success = await login(username.value, password.value);
        if (!success) {
          error.value = 'Invalid username or password.';
        }
      }
      
      if (success) {
        router.push('/');
      }
    } finally {
      loading.value = false;
    }
  };
  
  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-900 p-4">
      <Card shadow="md" padding="lg" radius="md" class="w-full max-w-md bg-gray-800">
        <h1 class="text-2xl font-bold text-center mb-6">
          {isRegistering.value ? 'Create Account' : 'Sign In'}
        </h1>
        
        <form onSubmit={handleSubmit} class="space-y-4">
          <TextInput
            label="Username"
            placeholder="Enter your username"
            value={username.value}
            onChange={(event: { target: { value: string } }) => username.value = event.target.value}
            required
          />
          
          <PasswordInput
            label="Password"
            placeholder="Enter your password"
            value={password.value}
            onChange={(event: { target: { value: string } }) => password.value = event.target.value}
            required
          />
          
          {error.value && (
            <p class="text-red-500 text-sm">{error.value}</p>
          )}
          
          <Button
            type="submit"
            fullWidth
            loading={loading.value}
          >
            {isRegistering.value ? 'Register' : 'Login'}
          </Button>
          
          <p class="text-center text-gray-400 text-sm">
            {isRegistering.value ? 'Already have an account?' : "Don't have an account?"}{' '}
            <button
              type="button"
              class="text-blue-400 hover:underline"
              onClick={() => {
                isRegistering.value = !isRegistering.value;
                error.value = '';
              }}
            >
              {isRegistering.value ? 'Sign In' : 'Register'}
            </button>
          </p>
        </form>
      </Card>
    </div>
  );
}
