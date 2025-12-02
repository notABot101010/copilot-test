import { useSignal } from '@preact/signals';
import { login as doLogin, register as doRegister } from '../hooks/state';
import { useRouter } from '@copilot-test/preact-router';

export function LoginPage() {
  const router = useRouter();
  const username = useSignal('');
  const password = useSignal('');
  const isRegistering = useSignal(false);
  const error = useSignal<string | null>(null);
  const loading = useSignal(false);

  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    error.value = null;
    loading.value = true;

    try {
      if (isRegistering.value) {
        await doRegister(username.value, password.value);
      } else {
        await doLogin(username.value, password.value);
      }
      router.push('/videos');
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Authentication failed';
    } finally {
      loading.value = false;
    }
  };

  return (
    <div class="min-h-screen bg-neutral-900 flex items-center justify-center p-4">
      <div class="w-full max-w-md">
        {/* Logo */}
        <div class="text-center mb-8">
          <span class="text-5xl">📺</span>
          <h1 class="text-4xl font-bold text-red-600 mt-4">TVflix</h1>
          <p class="text-neutral-400 mt-2">Your personal media center</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} class="bg-neutral-800 rounded-lg p-8">
          <h2 class="text-2xl font-bold text-white mb-6">
            {isRegistering.value ? 'Create Account' : 'Sign In'}
          </h2>

          {error.value && (
            <div class="bg-red-600/20 border border-red-600 text-red-400 rounded p-3 mb-4 text-sm">
              {error.value}
            </div>
          )}

          <div class="space-y-4">
            <div>
              <label class="block text-sm text-neutral-400 mb-1">Username</label>
              <input
                type="text"
                value={username.value}
                onInput={(event) => username.value = (event.target as HTMLInputElement).value}
                required
                class="w-full px-4 py-3 bg-neutral-700 rounded text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-red-600"
                placeholder="Enter username"
              />
            </div>

            <div>
              <label class="block text-sm text-neutral-400 mb-1">Password</label>
              <input
                type="password"
                value={password.value}
                onInput={(event) => password.value = (event.target as HTMLInputElement).value}
                required
                minLength={6}
                class="w-full px-4 py-3 bg-neutral-700 rounded text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-red-600"
                placeholder="Enter password"
              />
            </div>

            <button
              type="submit"
              disabled={loading.value}
              class="w-full py-3 bg-red-600 hover:bg-red-700 text-white font-medium rounded transition-colors disabled:opacity-50"
            >
              {loading.value ? 'Please wait...' : (isRegistering.value ? 'Create Account' : 'Sign In')}
            </button>
          </div>

          <div class="mt-6 text-center text-sm text-neutral-400">
            {isRegistering.value ? (
              <>
                Already have an account?{' '}
                <button
                  type="button"
                  onClick={() => isRegistering.value = false}
                  class="text-red-500 hover:text-red-400"
                >
                  Sign In
                </button>
              </>
            ) : (
              <>
                Don't have an account?{' '}
                <button
                  type="button"
                  onClick={() => isRegistering.value = true}
                  class="text-red-500 hover:text-red-400"
                >
                  Create one
                </button>
              </>
            )}
          </div>
        </form>
      </div>
    </div>
  );
}
