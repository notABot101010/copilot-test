let db: any = null;
let sqlite3: any = null;

export async function initDatabase() {
  if (db) return db;

  try {
    const sqlite3InitModule = (await import('@sqlite.org/sqlite-wasm')).default;
    
    sqlite3 = await sqlite3InitModule({
      print: console.log,
      printErr: console.error,
    });

    console.log('SQLite3 initialized');
    
    db = new sqlite3.oo1.DB(':memory:', 'c');
    
    // Create sample tables with data
    db.exec(`
      CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
      
      CREATE TABLE IF NOT EXISTS products (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        price REAL NOT NULL,
        stock INTEGER DEFAULT 0,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
      
      CREATE TABLE IF NOT EXISTS orders (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        total REAL NOT NULL,
        status TEXT DEFAULT 'pending',
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(user_id) REFERENCES users(id)
      );
      
      -- Insert sample data
      INSERT INTO users (name, email) VALUES 
        ('John Doe', 'john@example.com'),
        ('Jane Smith', 'jane@example.com'),
        ('Bob Johnson', 'bob@example.com');
      
      INSERT INTO products (name, price, stock) VALUES 
        ('Laptop', 999.99, 10),
        ('Mouse', 29.99, 50),
        ('Keyboard', 79.99, 30),
        ('Monitor', 299.99, 15);
      
      INSERT INTO orders (user_id, total, status) VALUES 
        (1, 1029.98, 'completed'),
        (2, 379.98, 'pending'),
        (1, 29.99, 'shipped');
    `);

    console.log('Database initialized with sample data');
    return db;
  } catch (err) {
    console.error('Error initializing database:', err);
    throw err;
  }
}

export function getDatabase() {
  return db;
}

export function executeQuery(query: string): { columns: string[], rows: any[][] } {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }

  try {
    const results: any[][] = [];
    const columns: string[] = [];
    
    db.exec({
      sql: query,
      callback: (row: any) => {
        results.push(row);
      },
      columnNames: columns,
    });

    return { columns, rows: results };
  } catch (err: any) {
    throw new Error(`Query error: ${err.message}`);
  }
}

export function listTables(): string[] {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }

  const result = executeQuery(
    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
  );
  
  return result.rows.map((row: any) => row[0]);
}

export function getTableSchema(tableName: string): { columns: string[], rows: any[][] } {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }

  return executeQuery(`PRAGMA table_info(${tableName})`);
}
