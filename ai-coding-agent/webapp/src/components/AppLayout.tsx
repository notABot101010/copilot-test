import type { ComponentChildren } from 'preact';

interface Props {
  children: ComponentChildren;
}

export function AppLayout({ children }: Props) {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <a href="/" className="flex items-center gap-2">
            <span className="text-2xl">🤖</span>
            <h1 className="text-xl font-bold text-gray-900">AI Coding Agent</h1>
          </a>
          <nav className="flex gap-4">
            <a href="/" className="text-gray-600 hover:text-gray-900">Sessions</a>
            <a href="/templates" className="text-gray-600 hover:text-gray-900">Templates</a>
          </nav>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 py-6">
        {children}
      </main>
    </div>
  );
}
