# SQLite WASM Demo

A fully functional MVP React application demonstrating SQLite WASM capabilities with an intuitive user interface.

## Features

- **Database Tables Panel**: Browse all tables in the database with:
  - Expandable table details
  - Schema view showing column names, types, and constraints
  - Sample data preview (first 5 rows)

- **SQL Query Interface**: Execute SQL queries with:
  - Pre-configured sample queries for quick testing
  - Custom SQL query textarea
  - Formatted table results with column headers
  - Row count display

- **Sample Data**: Pre-populated database with realistic data:
  - `users` table with 3 sample users
  - `products` table with 4 sample products
  - `orders` table with 3 sample orders (includes foreign key relationships)

## Tech Stack

- **Framework**: Preact with TypeScript
- **UI Components**: Mantine v8
- **Styling**: Tailwind CSS v4
- **Database**: SQLite WASM (in-memory)
- **State Management**: Preact Signals
- **Build Tool**: Vite v7

## Getting Started

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

The production build will be available in the `dist` folder.

### Preview Production Build

```bash
npm run preview
```

## How to Use

1. **Browse Tables**: Click on any table name in the left panel to expand and view its schema and sample data
2. **Execute Queries**: 
   - Click on any "Query" button to load a sample query
   - Or write your own SQL query in the textarea
   - Click "Execute Query" to run it
3. **View Results**: Query results are displayed in a formatted table below the query interface

## Sample Queries

The app includes 4 pre-configured sample queries:

1. `SELECT * FROM users` - View all users
2. `SELECT * FROM products WHERE price > 50` - Filter products by price
3. `SELECT u.name, o.total, o.status FROM orders o JOIN users u ON o.user_id = u.id` - Join orders with users
4. `SELECT name, COUNT(*) as total FROM users GROUP BY name` - Group by user name

## Architecture

The application runs entirely in the browser with no backend server required. SQLite is compiled to WebAssembly and runs in-memory, making it:

- **Fast**: No network latency for database operations
- **Portable**: Works offline once loaded
- **Secure**: All data stays in the browser

## Security

- Table name validation to prevent SQL injection
- Type-safe TypeScript interfaces
- No vulnerabilities in dependencies (verified with GitHub Advisory Database)

## License

MIT
