import { useEffect } from 'preact/hooks'
import { useEditor, EditorContent } from '@tiptap/react'
import StarterKit from '@tiptap/starter-kit'
import { Button, TextInput } from '@mantine/core'
import { currentDocument, updateDocument } from '../store'
import { defaultMarkdownSerializer } from 'prosemirror-markdown'

export function Editor() {
  const doc = currentDocument.value

  const editor = useEditor({
    extensions: [StarterKit],
    content: doc?.content || '',
    editorProps: {
      attributes: {
        class: 'prose prose-sm sm:prose lg:prose-lg xl:prose-xl max-w-none focus:outline-none px-8 py-6',
      },
    },
    onUpdate: ({ editor }) => {
      if (doc) {
        updateDocument(doc.id, { content: editor.getHTML() })
      }
    },
  })

  // Update editor content when document changes
  useEffect(() => {
    if (editor && doc && editor.getHTML() !== doc.content) {
      editor.commands.setContent(doc.content)
    }
  }, [doc?.id])

  const exportMarkdown = () => {
    if (!editor || !doc) return

    // Convert the editor content to markdown
    const prosemirrorDoc = editor.state.doc
    const markdown = defaultMarkdownSerializer.serialize(prosemirrorDoc)

    // Create a blob and download
    const blob = new Blob([markdown], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${doc.title.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.md`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  if (!doc) return null

  return (
    <div className="flex flex-col h-full bg-white">
      <div className="border-b border-gray-200 p-4 flex items-center gap-4">
        <TextInput
          value={doc.title}
          onChange={(e) => updateDocument(doc.id, { title: e.currentTarget.value })}
          placeholder="Document title"
          className="flex-1"
          classNames={{
            input: 'text-lg font-semibold border-0 focus:border-0',
          }}
        />
        <Button onClick={exportMarkdown} variant="light">
          Export Markdown
        </Button>
      </div>

      <div className="flex-1 overflow-auto">
        <EditorContent editor={editor} />
      </div>

      <div className="border-t border-gray-200 p-3 bg-gray-50">
        <div className="flex gap-2 flex-wrap">
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleBold().run()}
            className={editor?.isActive('bold') ? 'bg-blue-100' : ''}
          >
            Bold
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleItalic().run()}
            className={editor?.isActive('italic') ? 'bg-blue-100' : ''}
          >
            Italic
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}
            className={editor?.isActive('heading', { level: 1 }) ? 'bg-blue-100' : ''}
          >
            H1
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
            className={editor?.isActive('heading', { level: 2 }) ? 'bg-blue-100' : ''}
          >
            H2
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleBulletList().run()}
            className={editor?.isActive('bulletList') ? 'bg-blue-100' : ''}
          >
            Bullet List
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleOrderedList().run()}
            className={editor?.isActive('orderedList') ? 'bg-blue-100' : ''}
          >
            Ordered List
          </Button>
          <Button
            size="xs"
            variant="light"
            onClick={() => editor?.chain().focus().toggleCodeBlock().run()}
            className={editor?.isActive('codeBlock') ? 'bg-blue-100' : ''}
          >
            Code Block
          </Button>
        </div>
      </div>
    </div>
  )
}
