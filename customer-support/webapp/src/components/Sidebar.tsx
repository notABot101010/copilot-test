import { IconMessageCircle, IconChartBar, IconUsers, IconSettings } from '@tabler/icons-react';
import { currentWorkspace } from '../state';

interface SidebarProps {
  currentPath: string;
}

export function Sidebar({ currentPath }: SidebarProps) {
  const workspace = currentWorkspace.value;

  if (!workspace) return null;

  const links = [
    { path: `/w/${workspace.id}/chat`, icon: IconMessageCircle, label: 'Conversations' },
    { path: `/w/${workspace.id}/analytics`, icon: IconChartBar, label: 'Analytics' },
    { path: `/w/${workspace.id}/contacts`, icon: IconUsers, label: 'Contacts' },
  ];

  return (
    <div className="w-64 bg-gray-900 text-white flex flex-col h-screen">
      <div className="p-4 border-b border-gray-700">
        <h1 className="text-lg font-semibold">{workspace.name}</h1>
        <p className="text-xs text-gray-400 mt-1 truncate">ID: {workspace.id}</p>
      </div>
      <nav className="flex-1 p-2">
        {links.map((link) => {
          const isActive = currentPath.startsWith(link.path);
          return (
            <a
              key={link.path}
              href={link.path}
              className={`flex items-center gap-3 px-3 py-2 rounded-md mb-1 transition-colors ${
                isActive
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-800 hover:text-white'
              }`}
            >
              <link.icon size={20} />
              <span>{link.label}</span>
            </a>
          );
        })}
      </nav>
      <div className="p-2 border-t border-gray-700">
        <a
          href="/"
          className="flex items-center gap-3 px-3 py-2 rounded-md text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
        >
          <IconSettings size={20} />
          <span>Switch Workspace</span>
        </a>
      </div>
    </div>
  );
}
