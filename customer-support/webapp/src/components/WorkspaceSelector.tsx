import { useState, useEffect } from 'preact/hooks';
import { IconPlus, IconMessageCircle } from '@tabler/icons-react';
import { workspaces, setWorkspace } from '../state';
import * as api from '../services/api';
import type { Workspace } from '../types';

export function WorkspaceSelector() {
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState('');

  useEffect(() => {
    loadWorkspaces();
  }, []);

  async function loadWorkspaces() {
    setLoading(true);
    try {
      const data = await api.listWorkspaces();
      workspaces.value = data;
    } catch (err) {
      console.error('Failed to load workspaces:', err);
    } finally {
      setLoading(false);
    }
  }

  async function handleCreate() {
    if (!newName.trim()) return;
    try {
      const workspace = await api.createWorkspace(newName.trim());
      workspaces.value = [workspace, ...workspaces.value];
      setNewName('');
      setCreating(false);
      // Navigate to new workspace
      window.location.href = `/w/${workspace.id}/chat`;
    } catch (err) {
      console.error('Failed to create workspace:', err);
    }
  }

  function selectWorkspace(workspace: Workspace) {
    setWorkspace(workspace);
    window.location.href = `/w/${workspace.id}/chat`;
  }

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 bg-blue-600 rounded-full mb-4">
            <IconMessageCircle size={32} className="text-white" />
          </div>
          <h1 className="text-2xl font-bold text-gray-900">Customer Support</h1>
          <p className="text-gray-600 mt-2">Select or create a workspace to get started</p>
        </div>

        <div className="bg-white rounded-lg shadow-sm">
          {loading ? (
            <div className="p-6 text-center text-gray-500">Loading workspaces...</div>
          ) : (
            <>
              {creating ? (
                <div className="p-4 border-b border-gray-200">
                  <input
                    type="text"
                    value={newName}
                    onChange={(e) => setNewName((e.target as HTMLInputElement).value)}
                    placeholder="Workspace name"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-blue-500 mb-3"
                    autoFocus
                    onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={handleCreate}
                      className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                      Create
                    </button>
                    <button
                      onClick={() => setCreating(false)}
                      className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  onClick={() => setCreating(true)}
                  className="w-full p-4 flex items-center gap-3 text-blue-600 hover:bg-blue-50 transition-colors border-b border-gray-200"
                >
                  <IconPlus size={20} />
                  <span className="font-medium">Create new workspace</span>
                </button>
              )}

              <div className="divide-y divide-gray-100">
                {workspaces.value.length === 0 ? (
                  <div className="p-6 text-center text-gray-500">
                    No workspaces yet. Create one to get started!
                  </div>
                ) : (
                  workspaces.value.map((ws) => (
                    <button
                      key={ws.id}
                      onClick={() => selectWorkspace(ws)}
                      className="w-full p-4 flex items-center justify-between hover:bg-gray-50 transition-colors text-left"
                    >
                      <div>
                        <p className="font-medium text-gray-900">{ws.name}</p>
                        <p className="text-sm text-gray-500 mt-0.5">
                          Created {new Date(ws.created_at).toLocaleDateString()}
                        </p>
                      </div>
                      <div className="text-gray-400">→</div>
                    </button>
                  ))
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
