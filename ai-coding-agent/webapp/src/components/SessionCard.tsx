import type { Session } from '../types';

interface Props {
  session: Session;
  isActive?: boolean;
}

export function SessionCard({ session, isActive }: Props) {
  const formattedDate = new Date(session.updated_at).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <a
      href={`/session/${session.id}`}
      className={`block p-4 rounded-lg border transition-colors ${
        isActive
          ? 'border-blue-500 bg-blue-50'
          : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow-sm'
      }`}
    >
      <div className="flex items-center justify-between">
        <h3 className="font-medium text-gray-900">{session.name}</h3>
        <span
          className={`px-2 py-1 text-xs rounded-full ${
            session.status === 'active'
              ? 'bg-green-100 text-green-800'
              : 'bg-gray-100 text-gray-600'
          }`}
        >
          {session.status}
        </span>
      </div>
      <p className="mt-1 text-sm text-gray-500">{formattedDate}</p>
    </a>
  );
}
