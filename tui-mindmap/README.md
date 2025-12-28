# TUI MindMap

A feature-rich terminal-based mindmap application built with Rust and ratatui.

## Features

- **Interactive Canvas**: Pan and zoom to navigate your mindmap
- **Node Management**: Create, delete, and organize nodes freely
- **Document Editing**: Each node contains a title and body document
- **Visual Connections**: Connect and disconnect nodes to show relationships
- **Mouse Support**: Drag nodes, click to select, double-click to open
- **Undo/Redo**: Full history support with Ctrl+Z and Ctrl+Y
- **Save/Load**: Persist your mindmaps to JSON files
- **Intuitive Controls**: Keyboard shortcuts for all operations

## Installation

From the repository root:

```bash
cargo build --release -p tui-mindmap
```

## Usage

Run the application:

```bash
cargo run -p tui-mindmap
```

Or run the compiled binary:

```bash
./target/release/tui-mindmap
```

## Controls

### Navigation
- **Arrow Keys**: Pan the canvas
- **+/-**: Zoom in/out (when no node is selected)
- **Mouse Scroll**: Zoom in/out
- **Mouse Drag**: Move nodes

### Node Operations
- **N**: Create a new node at the center
- **Click**: Select a node
- **Double-Click**: Open node document
- **D**: Delete selected node
- **Esc**: Deselect node (or cancel operation)

### Connections
- **C**: Start connecting from selected node
- **Click another node**: Complete the connection
- **Esc**: Cancel connection mode

### Document Editing
When viewing a document:
- **Enter**: Start editing
- **Esc**: Close without saving

When editing a document:
- **Tab**: Switch between title and body fields
- **Type**: Enter text
- **Backspace**: Delete characters
- **Enter**: Save and close
- **Esc**: Cancel and close without saving

### File Operations
- **S**: Save mindmap to `mindmap.json`
- **L**: Load mindmap from `mindmap.json`
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Q**: Quit application

## Tips

1. **Getting Started**: The application starts with a sample mindmap to help you understand the features
2. **Organization**: Use connections to show relationships between ideas
3. **Navigation**: Zoom out to see the big picture, zoom in to focus on details
4. **Persistence**: Remember to save your work with 'S' before quitting
5. **Recovery**: Use Ctrl+Z/Ctrl+Y to undo mistakes

## File Format

Mindmaps are saved as JSON files with the following structure:

```json
{
  "nodes": [
    {
      "id": "uuid",
      "document": {
        "title": "Node Title",
        "body": "Node body text..."
      },
      "x": 40.0,
      "y": 10.0,
      "width": 20,
      "height": 3
    }
  ],
  "connections": [
    {
      "from": "uuid1",
      "to": "uuid2"
    }
  ]
}
```

## Architecture

- **models.rs**: Data structures (Node, Document, Connection, MindMap)
- **ui.rs**: Rendering components (Canvas, DocumentDialog)
- **main.rs**: Application logic and event handling

## License

MIT
