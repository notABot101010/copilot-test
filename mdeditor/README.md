# Markdown Editor

A beautiful and minimalist markdown editor built with Preact, TailwindCSS, Mantine, and Tiptap.

## Features

- Rich-text markdown editing with Tiptap
- Document management (create, edit, delete)
- Auto-save to localStorage
- Export documents as markdown files
- Responsive design
- Minimalist and clean UI

## Tech Stack

- **Preact** - Fast 3kB alternative to React
- **TailwindCSS** - Utility-first CSS framework
- **Mantine** - React components library
- **Tiptap** - Headless rich-text editor
- **Vite** - Next generation frontend tooling
- **TypeScript** - Type safety

## Getting Started

### Prerequisites

- Node.js (v18 or higher)
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

The app will be available at `http://localhost:4000`

### Build

```bash
npm run build
```

### Preview Production Build

```bash
npm run preview
```

## Usage

### Creating Documents

Click the "New Document" button in the sidebar to create a new document.

### Editing

- Use the toolbar at the bottom of the editor to format text
- Supported formats: Bold, Italic, Headings (H1, H2), Lists, Code Blocks
- All changes are automatically saved to localStorage

### Exporting

Click the "Export Markdown" button at the top right to download the current document as a `.md` file.

### Deleting Documents

Hover over a document in the sidebar and click the "×" button to delete it (you must have at least one document).

## Project Structure

```
mdeditor/
├── src/
│   ├── components/
│   │   ├── Sidebar.tsx     # Document list and management
│   │   └── Editor.tsx      # Tiptap editor with toolbar
│   ├── app.tsx             # Main app component
│   ├── main.tsx           # Entry point
│   ├── store.ts           # State management with Preact signals
│   ├── types.ts           # TypeScript types
│   └── index.css          # Global styles
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```

## License

ISC
