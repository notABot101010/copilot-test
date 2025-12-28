# TUI MindMap

A feature-rich terminal-based mindmap application built with Rust and ratatui.

## Features

- **Interactive Canvas**: Pan and zoom to navigate your mindmap
- **Node Management**: Create, delete, and organize nodes freely
- **Document Editing**: Each node contains a title and body document
- **Visual Connections**: Connect and disconnect nodes to show relationships
- **Color Coding**: Assign different colors to nodes for better organization
- **Search Functionality**: Quickly find nodes by searching in titles and body text
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
- **Mouse Drag on Node**: Move nodes
- **Mouse Drag on Empty Space**: Pan the canvas

### Node Operations
- **N**: Create a new node at the center
- **Click**: Select a node
- **Double-Click**: Open node document
- **Enter**: Open node document (when node is selected)
- **D**: Delete selected node
- **R**: Cycle through colors for selected node
- **Esc**: Deselect node (or cancel operation)

### Connections
- **C**: Start connecting from selected node
- **Click another node**: Complete the connection
- **X**: Disconnect all connections from selected node
- **Esc**: Cancel connection mode

### Document Editing
When viewing a document:
- **Enter**: Start editing
- **Esc**: Close without saving

When editing a document:
- **Tab**: Switch between title and body fields
- **Type**: Enter text
- **Left/Right Arrows**: Move cursor
- **Home/End**: Jump to start/end of text
- **Backspace**: Delete characters
- **Enter**: Save and close
- **Esc**: Cancel and close without saving

### Search
- **F**: Open search dialog
- **Type**: Enter search query
- **Enter**: Execute search and jump to first result
- **Down**: Navigate to next search result
- **Esc**: Close search dialog

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
