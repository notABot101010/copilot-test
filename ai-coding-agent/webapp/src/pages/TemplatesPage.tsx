import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import type { PromptTemplate } from '../types';
import { listTemplates } from '../api';
import { TemplateEditor } from '../components/TemplateEditor';

export function TemplatesPage() {
  const templates = useSignal<PromptTemplate[]>([]);
  const loading = useSignal(true);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    loading.value = true;
    try {
      templates.value = await listTemplates();
    } catch (err) {
      console.error('Failed to load templates:', err);
    } finally {
      loading.value = false;
    }
  };

  const handleTemplateUpdate = (updated: PromptTemplate) => {
    templates.value = templates.value.map((t) =>
      t.id === updated.id ? updated : t
    );
  };

  return (
    <div>
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-900">Prompt Templates</h2>
        <p className="text-gray-600 mt-1">
          Customize the system prompts for each sub-agent. Use {'{{'} variable {'}}'} syntax for dynamic values.
        </p>
      </div>

      {loading.value ? (
        <div className="text-center py-8 text-gray-500">Loading templates...</div>
      ) : templates.value.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          No templates available.
        </div>
      ) : (
        <div className="space-y-4">
          {templates.value.map((template) => (
            <TemplateEditor
              key={template.id}
              template={template}
              onUpdate={handleTemplateUpdate}
            />
          ))}
        </div>
      )}
    </div>
  );
}
