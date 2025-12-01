import { useSignal } from '@preact/signals';
import { useNavigation } from '@copilot-test/preact-router';
import { login } from '../services/api';
import { setCurrentUser, currentUser } from '../app';
import { useEffect } from 'preact/hooks';

export default function Login() {
  const username = useSignal('');
  const password = useSignal('');
  const error = useSignal('');
  const loading = useSignal(false);
  const { push } = useNavigation();

  useEffect(() => {
    if (currentUser.value) {
      push('/groups');
    }
  }, []);

  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    error.value = '';
    loading.value = true;

    try {
      const response = await login(username.value, password.value);
      if (response.success && response.user_id) {
        setCurrentUser({
          username: response.username,
          userId: response.user_id,
        });
        push('/groups');
      } else {
        error.value = response.error || 'Login failed';
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed';
    } finally {
      loading.value = false;
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div class="max-w-md w-full space-y-8">
        <div>
          <h1 class="mt-6 text-center text-3xl font-extrabold text-gray-900">MLS Chat</h1>
          <h2 class="mt-2 text-center text-xl text-gray-600">Sign in to your account</h2>
        </div>
        <form class="mt-8 space-y-6" onSubmit={handleSubmit}>
          {error.value && (
            <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
              {error.value}
            </div>
          )}
          <div class="rounded-md shadow-sm -space-y-px">
            <div>
              <label htmlFor="username" class="sr-only">Username</label>
              <input
                id="username"
                name="username"
                type="text"
                required
                class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                placeholder="Username"
                value={username.value}
                onInput={(event) => (username.value = (event.target as HTMLInputElement).value)}
              />
            </div>
            <div>
              <label htmlFor="password" class="sr-only">Password</label>
              <input
                id="password"
                name="password"
                type="password"
                required
                class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                placeholder="Password"
                value={password.value}
                onInput={(event) => (password.value = (event.target as HTMLInputElement).value)}
              />
            </div>
          </div>

          <div>
            <button
              type="submit"
              disabled={loading.value}
              class="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            >
              {loading.value ? 'Signing in...' : 'Sign in'}
            </button>
          </div>

          <div class="text-center">
            <a href="/register" class="text-blue-600 hover:text-blue-500">
              Don't have an account? Register
            </a>
          </div>
        </form>
      </div>
    </div>
  );
}
