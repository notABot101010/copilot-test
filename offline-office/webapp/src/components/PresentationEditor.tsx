import { useEffect, useState } from 'preact/hooks';
import { signal, computed } from '@preact/signals';
import type { PresentationContent } from '../types';
import { AutomergeDocumentManager } from '../automerge-manager';
import { Button } from '@mantine/core';

interface PresentationEditorProps {
  documentId: string;
  title: string;
}

const currentSlideIndex = signal(0);

export function PresentationEditor({ documentId, title }: PresentationEditorProps) {
  const [docManager, setDocManager] = useState<AutomergeDocumentManager<PresentationContent> | null>(null);
  const content = docManager?.content || signal<PresentationContent | null>(null);

  const currentSlide = computed(() => {
    const slides = content.value?.slides || [];
    return slides[currentSlideIndex.value] || null;
  });

  useEffect(() => {
    const manager = new AutomergeDocumentManager<PresentationContent>(documentId);
    setDocManager(manager);

    manager.loadFromServer().then(() => {
      if (!manager.content.value || !manager.content.value.slides) {
        manager.change((doc) => {
          doc.slides = [{
            id: crypto.randomUUID(),
            title: 'Slide 1',
            content: '',
          }];
        });
      }
      manager.connectWebSocket();
    });

    return () => {
      manager.disconnect();
    };
  }, [documentId]);

  const addSlide = () => {
    if (!docManager) return;

    docManager.change((doc) => {
      if (!doc.slides) {
        doc.slides = [];
      }
      doc.slides.push({
        id: crypto.randomUUID(),
        title: `Slide ${doc.slides.length + 1}`,
        content: '',
      });
    });

    currentSlideIndex.value = (content.value?.slides.length || 1) - 1;
  };

  const deleteSlide = (index: number) => {
    if (!docManager) return;

    docManager.change((doc) => {
      if (doc.slides && doc.slides.length > 1) {
        doc.slides.splice(index, 1);
      }
    });

    if (currentSlideIndex.value >= (content.value?.slides.length || 1) - 1) {
      currentSlideIndex.value = Math.max(0, (content.value?.slides.length || 1) - 2);
    }
  };

  const updateSlideTitle = (index: number, newTitle: string) => {
    if (!docManager) return;

    docManager.change((doc) => {
      if (doc.slides && doc.slides[index]) {
        doc.slides[index].title = newTitle;
      }
    });
  };

  const updateSlideContent = (index: number, newContent: string) => {
    if (!docManager) return;

    docManager.change((doc) => {
      if (doc.slides && doc.slides[index]) {
        doc.slides[index].content = newContent;
      }
    });
  };

  const slides = content.value?.slides || [];

  return (
    <div class="w-full h-screen bg-gray-100 dark:bg-gray-900 flex">
      <div class="w-64 bg-white dark:bg-gray-800 border-r border-gray-300 dark:border-gray-700 overflow-y-auto">
        <div class="p-4">
          <h2 class="text-lg font-bold mb-4 text-gray-900 dark:text-white">
            {title}
          </h2>
          <Button onClick={addSlide} fullWidth class="mb-4">
            Add Slide
          </Button>
          <div class="space-y-2">
            {slides.map((slide, index) => (
              <div
                key={slide.id}
                class={`p-3 rounded cursor-pointer border-2 ${
                  currentSlideIndex.value === index
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900'
                    : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                }`}
                onClick={() => (currentSlideIndex.value = index)}
              >
                <div class="flex justify-between items-center">
                  <span class="text-sm font-medium text-gray-900 dark:text-white">
                    {slide.title}
                  </span>
                  {slides.length > 1 && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteSlide(index);
                      }}
                      class="text-red-500 hover:text-red-700 text-xs"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div class="flex-1 p-8 overflow-y-auto">
        {currentSlide.value && (
          <div class="max-w-4xl mx-auto">
            <input
              type="text"
              value={currentSlide.value.title}
              onInput={(e) => updateSlideTitle(currentSlideIndex.value, (e.target as HTMLInputElement).value)}
              class="w-full text-4xl font-bold mb-6 p-2 border-0 border-b-2 border-gray-300 dark:border-gray-700 focus:border-blue-500 focus:outline-none bg-transparent text-gray-900 dark:text-white"
              placeholder="Slide Title"
            />
            <textarea
              value={currentSlide.value.content}
              onInput={(e) => updateSlideContent(currentSlideIndex.value, (e.target as HTMLTextAreaElement).value)}
              class="w-full h-96 p-4 border border-gray-300 dark:border-gray-700 rounded-lg focus:border-blue-500 focus:outline-none resize-none bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              placeholder="Slide content (supports markdown)"
            />
          </div>
        )}
      </div>
    </div>
  );
}
