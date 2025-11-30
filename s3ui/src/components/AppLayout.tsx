import type { ComponentChildren } from 'preact';

interface AppLayoutProps {
  children: ComponentChildren;
}

export function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <a href="/" className="text-xl font-bold text-gray-900 no-underline">
              S3 Browser
            </a>
            <nav className="flex gap-4">
              <a href="/" className="text-gray-600 hover:text-gray-900 no-underline">
                Buckets
              </a>
            </nav>
          </div>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 py-6">
        {children}
      </main>
    </div>
  );
}
