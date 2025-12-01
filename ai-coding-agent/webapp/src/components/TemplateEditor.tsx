import { useSignal } from '@preact/signals';
import type { PromptTemplate } from '../types';
import { updateTemplate } from '../api';

interface Props {
  template: PromptTemplate;
  onUpdate: (template: PromptTemplate) => void;
}

export function TemplateEditor({ template, onUpdate }: Props) {
  const content = useSignal(template.system_prompt);
  const saving = useSignal(false);
  const expanded = useSignal(false);

  const handleSave = async () => {
    saving.value = true;
    try {
      const updated = await updateTemplate(template.id, content.value);
      onUpdate(updated);
    } catch (err) {
      console.error('Failed to save template:', err);
    } finally {
      saving.value = false;
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <button
        onClick={() => expanded.value = !expanded.value}
        className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-gray-50"
      >
        <div>
          <h3 className="font-medium text-gray-900">{template.name}</h3>
          <p className="text-sm text-gray-500">
            Variables: {template.variables.join(', ')}
          </p>
        </div>
        <span className="text-gray-400">{expanded.value ? '▼' : '▶'}</span>
      </button>
      
      {expanded.value && (
        <div className="border-t border-gray-200 p-4">
          <textarea
            value={content.value}
            onInput={(e) => content.value = (e.target as HTMLTextAreaElement).value}
            className="w-full h-64 p-3 font-mono text-sm border border-gray-200 rounded resize-y focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <div className="mt-3 flex justify-end">
            <button
              onClick={handleSave}
              disabled={saving.value}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            >
              {saving.value ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
