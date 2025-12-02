import { Button, ScrollArea, ActionIcon } from '@mantine/core'
import { documents, currentDocId, createDocument, selectDocument, deleteDocument } from '../store'

export function Sidebar() {
  return (
    <div className="flex flex-col h-full bg-gray-50 border-r border-gray-200">
      <div className="p-4 border-b border-gray-200">
        <Button
          fullWidth
          onClick={createDocument}
          className="bg-blue-600 hover:bg-blue-700"
        >
          New Document
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-2">
          {documents.value.map(doc => (
            <div
              key={doc.id}
              className={`
                group relative p-3 mb-2 rounded-lg cursor-pointer
                transition-colors
                ${currentDocId.value === doc.id
                  ? 'bg-blue-100 border-2 border-blue-400'
                  : 'bg-white border-2 border-transparent hover:bg-gray-100'
                }
              `}
              onClick={() => selectDocument(doc.id)}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <h3 className="font-medium text-sm truncate text-gray-900">
                    {doc.title}
                  </h3>
                  <p className="text-xs text-gray-500 mt-1">
                    {new Date(doc.updatedAt).toLocaleDateString()}
                  </p>
                </div>

                {documents.value.length > 1 && (
                  <ActionIcon
                    size="sm"
                    color="red"
                    variant="subtle"
                    className="opacity-0 group-hover:opacity-100 transition-opacity ml-2"
                    onClick={(e) => {
                      e.stopPropagation()
                      deleteDocument(doc.id)
                    }}
                  >
                    ×
                  </ActionIcon>
                )}
              </div>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}
