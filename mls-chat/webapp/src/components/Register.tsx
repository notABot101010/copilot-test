import { useSignal } from '@preact/signals';
import { useNavigation } from '@copilot-test/preact-router';
import { register, uploadKeyPackages } from '../services/api';
import { setCurrentUser, currentUser } from '../app';
import { useEffect } from 'preact/hooks';
import { generateKeyPackages } from '../services/mls';

export default function Register() {
  const username = useSignal('');
  const password = useSignal('');
  const confirmPassword = useSignal('');
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

    if (password.value !== confirmPassword.value) {
      error.value = 'Passwords do not match';
      return;
    }

    if (password.value.length < 6) {
      error.value = 'Password must be at least 6 characters';
      return;
    }

    if (username.value.length < 3) {
      error.value = 'Username must be at least 3 characters';
      return;
    }

    loading.value = true;

    try {
      const response = await register(username.value, password.value);
      if (response.success && response.user_id) {
        // Generate and upload key packages
        const keyPackages = await generateKeyPackages(username.value, 5);
        await uploadKeyPackages(username.value, keyPackages);

        setCurrentUser({
          username: response.username,
          userId: response.user_id,
        });
        push('/groups');
      } else {
        error.value = response.error || 'Registration failed';
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Registration failed';
    } finally {
      loading.value = false;
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div class="max-w-md w-full space-y-8">
        <div>
          <h1 class="mt-6 text-center text-3xl font-extrabold text-gray-900">MLS Chat</h1>
          <h2 class="mt-2 text-center text-xl text-gray-600">Create your account</h2>
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
                class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                placeholder="Password"
                value={password.value}
                onInput={(event) => (password.value = (event.target as HTMLInputElement).value)}
              />
            </div>
            <div>
              <label htmlFor="confirmPassword" class="sr-only">Confirm Password</label>
              <input
                id="confirmPassword"
                name="confirmPassword"
                type="password"
                required
                class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                placeholder="Confirm Password"
                value={confirmPassword.value}
                onInput={(event) => (confirmPassword.value = (event.target as HTMLInputElement).value)}
              />
            </div>
          </div>

          <div>
            <button
              type="submit"
              disabled={loading.value}
              class="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            >
              {loading.value ? 'Creating account...' : 'Create account'}
            </button>
          </div>

          <div class="text-center">
            <a href="/login" class="text-blue-600 hover:text-blue-500">
              Already have an account? Sign in
            </a>
          </div>
        </form>
      </div>
    </div>
  );
}
