//! Database layer for ShopSaaS platform

use sqlx::{sqlite::SqlitePool, FromRow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Store not found: {0}")]
    StoreNotFound(i64),
    #[error("Store slug already exists: {0}")]
    StoreSlugExists(String),
    #[error("Product not found: {0}")]
    ProductNotFound(i64),
    #[error("Page not found: {0}")]
    PageNotFound(i64),
    #[error("Order not found: {0}")]
    OrderNotFound(i64),
    #[error("Customer not found: {0}")]
    CustomerNotFound(i64),
    #[error("Contact not found: {0}")]
    ContactNotFound(i64),
    #[error("Mailing list not found: {0}")]
    MailingListNotFound(i64),
    #[error("Invalid session")]
    InvalidSession,
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

// Shop owner (admin user)
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
}

// Store
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Store {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

// Product types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductType {
    Physical,
    Digital,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::Physical => write!(f, "physical"),
            ProductType::Digital => write!(f, "digital"),
        }
    }
}

impl std::str::FromStr for ProductType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physical" => Ok(ProductType::Physical),
            "digital" => Ok(ProductType::Digital),
            _ => Err(format!("Unknown product type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Product {
    pub id: i64,
    pub store_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price: i64, // Price in cents
    pub product_type: String,
    pub stock_quantity: Option<i64>,
    pub image_url: Option<String>,
    pub digital_file_path: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// Page builder
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Page {
    pub id: i64,
    pub store_id: i64,
    pub title: String,
    pub slug: String,
    pub is_published: bool,
    pub is_homepage: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct PageBlock {
    pub id: i64,
    pub page_id: i64,
    pub block_type: String,
    pub content: String, // JSON content
    pub position: i32,
    pub created_at: String,
}

// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Paid,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Paid => write!(f, "paid"),
            OrderStatus::Processing => write!(f, "processing"),
            OrderStatus::Shipped => write!(f, "shipped"),
            OrderStatus::Delivered => write!(f, "delivered"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Refunded => write!(f, "refunded"),
        }
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(OrderStatus::Pending),
            "paid" => Ok(OrderStatus::Paid),
            "processing" => Ok(OrderStatus::Processing),
            "shipped" => Ok(OrderStatus::Shipped),
            "delivered" => Ok(OrderStatus::Delivered),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "refunded" => Ok(OrderStatus::Refunded),
            _ => Err(format!("Unknown order status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Order {
    pub id: i64,
    pub store_id: i64,
    pub customer_id: Option<i64>,
    pub customer_email: String,
    pub customer_name: String,
    pub shipping_address: Option<String>,
    pub total_amount: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
}

// Customer (storefront user)
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Customer {
    pub id: i64,
    pub store_id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct CustomerSession {
    pub id: i64,
    pub customer_id: i64,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
}

// Contact (newsletter subscriber or contact form submission)
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Contact {
    pub id: i64,
    pub store_id: i64,
    pub email: String,
    pub name: Option<String>,
    pub subscribed_to_newsletter: bool,
    pub created_at: String,
}

// Mailing list
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct MailingList {
    pub id: i64,
    pub store_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct MailingListSubscriber {
    pub id: i64,
    pub mailing_list_id: i64,
    pub contact_id: i64,
    pub subscribed_at: String,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        // Users table (shop owners)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                token TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Stores table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                description TEXT,
                logo_url TEXT,
                currency TEXT NOT NULL DEFAULT 'USD',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Products table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS products (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                price INTEGER NOT NULL,
                product_type TEXT NOT NULL DEFAULT 'physical',
                stock_quantity INTEGER,
                image_url TEXT,
                digital_file_path TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Pages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                slug TEXT NOT NULL,
                is_published INTEGER NOT NULL DEFAULT 0,
                is_homepage INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE,
                UNIQUE(store_id, slug)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Page blocks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS page_blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                page_id INTEGER NOT NULL,
                block_type TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '{}',
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Customers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS customers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                email TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE,
                UNIQUE(store_id, email)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Customer sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS customer_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_id INTEGER NOT NULL,
                token TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL,
                FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Orders table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                customer_id INTEGER,
                customer_email TEXT NOT NULL,
                customer_name TEXT NOT NULL,
                shipping_address TEXT,
                total_amount INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE,
                FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Order items table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                product_id INTEGER NOT NULL,
                product_name TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                unit_price INTEGER NOT NULL,
                FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Contacts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contacts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                email TEXT NOT NULL,
                name TEXT,
                subscribed_to_newsletter INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE,
                UNIQUE(store_id, email)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Mailing lists table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mailing_lists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                store_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (store_id) REFERENCES stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Mailing list subscribers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mailing_list_subscribers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mailing_list_id INTEGER NOT NULL,
                contact_id INTEGER NOT NULL,
                subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (mailing_list_id) REFERENCES mailing_lists(id) ON DELETE CASCADE,
                FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
                UNIQUE(mailing_list_id, contact_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_stores_user ON stores(user_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_stores_slug ON stores(slug)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_products_store ON products(store_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_pages_store ON pages(store_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_store ON orders(store_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_customer ON orders(customer_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_customer_sessions_token ON customer_sessions(token)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // User operations
    pub async fn create_user(&self, email: &str, password_hash: &str, name: &str) -> Result<User> {
        if self.get_user_by_email(email).await?.is_some() {
            return Err(DatabaseError::UserAlreadyExists(email.to_string()));
        }

        let result = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, name)
            VALUES (?, ?, ?)
            RETURNING id, email, password_hash, name, created_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, created_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let result = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, created_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    // Session operations
    pub async fn create_session(&self, user_id: i64, token: &str, expires_at: &str) -> Result<Session> {
        let result = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, token, expires_at)
            VALUES (?, ?, ?)
            RETURNING id, user_id, token, created_at, expires_at
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>> {
        let result = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, token, created_at, expires_at FROM sessions WHERE token = ? AND expires_at > datetime('now')",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_session(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Store operations
    pub async fn create_store(&self, user_id: i64, name: &str, slug: &str, description: Option<&str>, currency: &str) -> Result<Store> {
        if self.get_store_by_slug(slug).await?.is_some() {
            return Err(DatabaseError::StoreSlugExists(slug.to_string()));
        }

        let result = sqlx::query_as::<_, Store>(
            r#"
            INSERT INTO stores (user_id, name, slug, description, currency)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, user_id, name, slug, description, logo_url, currency, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(currency)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_store_by_id(&self, id: i64) -> Result<Option<Store>> {
        let result = sqlx::query_as::<_, Store>(
            "SELECT id, user_id, name, slug, description, logo_url, currency, created_at, updated_at FROM stores WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_store_by_slug(&self, slug: &str) -> Result<Option<Store>> {
        let result = sqlx::query_as::<_, Store>(
            "SELECT id, user_id, name, slug, description, logo_url, currency, created_at, updated_at FROM stores WHERE slug = ?",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_stores_by_user(&self, user_id: i64) -> Result<Vec<Store>> {
        let result = sqlx::query_as::<_, Store>(
            "SELECT id, user_id, name, slug, description, logo_url, currency, created_at, updated_at FROM stores WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_store(&self, id: i64, name: &str, description: Option<&str>, logo_url: Option<&str>, currency: &str) -> Result<Store> {
        let result = sqlx::query_as::<_, Store>(
            r#"
            UPDATE stores SET name = ?, description = ?, logo_url = ?, currency = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING id, user_id, name, slug, description, logo_url, currency, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(logo_url)
        .bind(currency)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_store(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM stores WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Product operations
    pub async fn create_product(
        &self,
        store_id: i64,
        name: &str,
        description: Option<&str>,
        price: i64,
        product_type: ProductType,
        stock_quantity: Option<i64>,
        image_url: Option<&str>,
        digital_file_path: Option<&str>,
    ) -> Result<Product> {
        let result = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path, is_active, created_at, updated_at
            "#,
        )
        .bind(store_id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(product_type.to_string())
        .bind(stock_quantity)
        .bind(image_url)
        .bind(digital_file_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_product_by_id(&self, id: i64) -> Result<Option<Product>> {
        let result = sqlx::query_as::<_, Product>(
            "SELECT id, store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path, is_active, created_at, updated_at FROM products WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_products_by_store(&self, store_id: i64, active_only: bool) -> Result<Vec<Product>> {
        let result = if active_only {
            sqlx::query_as::<_, Product>(
                "SELECT id, store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path, is_active, created_at, updated_at FROM products WHERE store_id = ? AND is_active = 1 ORDER BY created_at DESC",
            )
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Product>(
                "SELECT id, store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path, is_active, created_at, updated_at FROM products WHERE store_id = ? ORDER BY created_at DESC",
            )
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(result)
    }

    pub async fn update_product(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
        price: i64,
        stock_quantity: Option<i64>,
        image_url: Option<&str>,
        digital_file_path: Option<&str>,
        is_active: bool,
    ) -> Result<Product> {
        let result = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products SET name = ?, description = ?, price = ?, stock_quantity = ?, image_url = ?, digital_file_path = ?, is_active = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING id, store_id, name, description, price, product_type, stock_quantity, image_url, digital_file_path, is_active, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(stock_quantity)
        .bind(image_url)
        .bind(digital_file_path)
        .bind(is_active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_product(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_product_stock(&self, id: i64, quantity_change: i64) -> Result<()> {
        sqlx::query("UPDATE products SET stock_quantity = stock_quantity + ?, updated_at = datetime('now') WHERE id = ? AND stock_quantity IS NOT NULL")
            .bind(quantity_change)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Page operations
    pub async fn create_page(&self, store_id: i64, title: &str, slug: &str, is_homepage: bool) -> Result<Page> {
        // If this is set as homepage, unset any existing homepage
        if is_homepage {
            sqlx::query("UPDATE pages SET is_homepage = 0 WHERE store_id = ? AND is_homepage = 1")
                .bind(store_id)
                .execute(&self.pool)
                .await?;
        }

        let result = sqlx::query_as::<_, Page>(
            r#"
            INSERT INTO pages (store_id, title, slug, is_homepage)
            VALUES (?, ?, ?, ?)
            RETURNING id, store_id, title, slug, is_published, is_homepage, created_at, updated_at
            "#,
        )
        .bind(store_id)
        .bind(title)
        .bind(slug)
        .bind(is_homepage)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_page_by_id(&self, id: i64) -> Result<Option<Page>> {
        let result = sqlx::query_as::<_, Page>(
            "SELECT id, store_id, title, slug, is_published, is_homepage, created_at, updated_at FROM pages WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_page_by_slug(&self, store_id: i64, slug: &str) -> Result<Option<Page>> {
        let result = sqlx::query_as::<_, Page>(
            "SELECT id, store_id, title, slug, is_published, is_homepage, created_at, updated_at FROM pages WHERE store_id = ? AND slug = ?",
        )
        .bind(store_id)
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_homepage(&self, store_id: i64) -> Result<Option<Page>> {
        let result = sqlx::query_as::<_, Page>(
            "SELECT id, store_id, title, slug, is_published, is_homepage, created_at, updated_at FROM pages WHERE store_id = ? AND is_homepage = 1",
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_pages_by_store(&self, store_id: i64) -> Result<Vec<Page>> {
        let result = sqlx::query_as::<_, Page>(
            "SELECT id, store_id, title, slug, is_published, is_homepage, created_at, updated_at FROM pages WHERE store_id = ? ORDER BY is_homepage DESC, created_at DESC",
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_page(&self, id: i64, title: &str, slug: &str, is_published: bool, is_homepage: bool) -> Result<Page> {
        // Get current page to find store_id
        let current = self.get_page_by_id(id).await?.ok_or(DatabaseError::PageNotFound(id))?;

        // If setting as homepage, unset others
        if is_homepage {
            sqlx::query("UPDATE pages SET is_homepage = 0 WHERE store_id = ? AND is_homepage = 1 AND id != ?")
                .bind(current.store_id)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        let result = sqlx::query_as::<_, Page>(
            r#"
            UPDATE pages SET title = ?, slug = ?, is_published = ?, is_homepage = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING id, store_id, title, slug, is_published, is_homepage, created_at, updated_at
            "#,
        )
        .bind(title)
        .bind(slug)
        .bind(is_published)
        .bind(is_homepage)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_page(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM pages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Page block operations
    pub async fn create_page_block(&self, page_id: i64, block_type: &str, content: &str, position: i32) -> Result<PageBlock> {
        let result = sqlx::query_as::<_, PageBlock>(
            r#"
            INSERT INTO page_blocks (page_id, block_type, content, position)
            VALUES (?, ?, ?, ?)
            RETURNING id, page_id, block_type, content, position, created_at
            "#,
        )
        .bind(page_id)
        .bind(block_type)
        .bind(content)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_page_blocks(&self, page_id: i64) -> Result<Vec<PageBlock>> {
        let result = sqlx::query_as::<_, PageBlock>(
            "SELECT id, page_id, block_type, content, position, created_at FROM page_blocks WHERE page_id = ? ORDER BY position",
        )
        .bind(page_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_page_block(&self, id: i64, content: &str, position: i32) -> Result<PageBlock> {
        let result = sqlx::query_as::<_, PageBlock>(
            r#"
            UPDATE page_blocks SET content = ?, position = ?
            WHERE id = ?
            RETURNING id, page_id, block_type, content, position, created_at
            "#,
        )
        .bind(content)
        .bind(position)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_page_block(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM page_blocks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn reorder_page_blocks(&self, page_id: i64, block_ids: &[i64]) -> Result<()> {
        for (index, block_id) in block_ids.iter().enumerate() {
            sqlx::query("UPDATE page_blocks SET position = ? WHERE id = ? AND page_id = ?")
                .bind(index as i32)
                .bind(block_id)
                .bind(page_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    // Customer operations
    pub async fn create_customer(&self, store_id: i64, email: &str, password_hash: &str, name: &str) -> Result<Customer> {
        let result = sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO customers (store_id, email, password_hash, name)
            VALUES (?, ?, ?, ?)
            RETURNING id, store_id, email, password_hash, name, created_at
            "#,
        )
        .bind(store_id)
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_customer_by_email(&self, store_id: i64, email: &str) -> Result<Option<Customer>> {
        let result = sqlx::query_as::<_, Customer>(
            "SELECT id, store_id, email, password_hash, name, created_at FROM customers WHERE store_id = ? AND email = ?",
        )
        .bind(store_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_customer_by_id(&self, id: i64) -> Result<Option<Customer>> {
        let result = sqlx::query_as::<_, Customer>(
            "SELECT id, store_id, email, password_hash, name, created_at FROM customers WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_customers_by_store(&self, store_id: i64) -> Result<Vec<Customer>> {
        let result = sqlx::query_as::<_, Customer>(
            "SELECT id, store_id, email, password_hash, name, created_at FROM customers WHERE store_id = ? ORDER BY created_at DESC",
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn create_customer_session(&self, customer_id: i64, token: &str, expires_at: &str) -> Result<CustomerSession> {
        let result = sqlx::query_as::<_, CustomerSession>(
            r#"
            INSERT INTO customer_sessions (customer_id, token, expires_at)
            VALUES (?, ?, ?)
            RETURNING id, customer_id, token, created_at, expires_at
            "#,
        )
        .bind(customer_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_customer_session_by_token(&self, token: &str) -> Result<Option<CustomerSession>> {
        let result = sqlx::query_as::<_, CustomerSession>(
            "SELECT id, customer_id, token, created_at, expires_at FROM customer_sessions WHERE token = ? AND expires_at > datetime('now')",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_customer_session(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM customer_sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Order operations
    pub async fn create_order(
        &self,
        store_id: i64,
        customer_id: Option<i64>,
        customer_email: &str,
        customer_name: &str,
        shipping_address: Option<&str>,
        total_amount: i64,
    ) -> Result<Order> {
        let result = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (store_id, customer_id, customer_email, customer_name, shipping_address, total_amount)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, store_id, customer_id, customer_email, customer_name, shipping_address, total_amount, status, created_at, updated_at
            "#,
        )
        .bind(store_id)
        .bind(customer_id)
        .bind(customer_email)
        .bind(customer_name)
        .bind(shipping_address)
        .bind(total_amount)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn add_order_item(&self, order_id: i64, product_id: i64, product_name: &str, quantity: i32, unit_price: i64) -> Result<OrderItem> {
        let result = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO order_items (order_id, product_id, product_name, quantity, unit_price)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, order_id, product_id, product_name, quantity, unit_price
            "#,
        )
        .bind(order_id)
        .bind(product_id)
        .bind(product_name)
        .bind(quantity)
        .bind(unit_price)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_order_by_id(&self, id: i64) -> Result<Option<Order>> {
        let result = sqlx::query_as::<_, Order>(
            "SELECT id, store_id, customer_id, customer_email, customer_name, shipping_address, total_amount, status, created_at, updated_at FROM orders WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_orders_by_store(&self, store_id: i64) -> Result<Vec<Order>> {
        let result = sqlx::query_as::<_, Order>(
            "SELECT id, store_id, customer_id, customer_email, customer_name, shipping_address, total_amount, status, created_at, updated_at FROM orders WHERE store_id = ? ORDER BY created_at DESC",
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_orders_by_customer(&self, customer_id: i64) -> Result<Vec<Order>> {
        let result = sqlx::query_as::<_, Order>(
            "SELECT id, store_id, customer_id, customer_email, customer_name, shipping_address, total_amount, status, created_at, updated_at FROM orders WHERE customer_id = ? ORDER BY created_at DESC",
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_order_items(&self, order_id: i64) -> Result<Vec<OrderItem>> {
        let result = sqlx::query_as::<_, OrderItem>(
            "SELECT id, order_id, product_id, product_name, quantity, unit_price FROM order_items WHERE order_id = ?",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_order_status(&self, id: i64, status: OrderStatus) -> Result<Order> {
        let result = sqlx::query_as::<_, Order>(
            r#"
            UPDATE orders SET status = ?, updated_at = datetime('now')
            WHERE id = ?
            RETURNING id, store_id, customer_id, customer_email, customer_name, shipping_address, total_amount, status, created_at, updated_at
            "#,
        )
        .bind(status.to_string())
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    // Contact operations
    pub async fn create_or_update_contact(&self, store_id: i64, email: &str, name: Option<&str>, subscribed: bool) -> Result<Contact> {
        let existing = sqlx::query_as::<_, Contact>(
            "SELECT id, store_id, email, name, subscribed_to_newsletter, created_at FROM contacts WHERE store_id = ? AND email = ?",
        )
        .bind(store_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(_) = existing {
            let result = sqlx::query_as::<_, Contact>(
                r#"
                UPDATE contacts SET name = COALESCE(?, name), subscribed_to_newsletter = ?
                WHERE store_id = ? AND email = ?
                RETURNING id, store_id, email, name, subscribed_to_newsletter, created_at
                "#,
            )
            .bind(name)
            .bind(subscribed)
            .bind(store_id)
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

            Ok(result)
        } else {
            let result = sqlx::query_as::<_, Contact>(
                r#"
                INSERT INTO contacts (store_id, email, name, subscribed_to_newsletter)
                VALUES (?, ?, ?, ?)
                RETURNING id, store_id, email, name, subscribed_to_newsletter, created_at
                "#,
            )
            .bind(store_id)
            .bind(email)
            .bind(name)
            .bind(subscribed)
            .fetch_one(&self.pool)
            .await?;

            Ok(result)
        }
    }

    pub async fn list_contacts_by_store(&self, store_id: i64) -> Result<Vec<Contact>> {
        let result = sqlx::query_as::<_, Contact>(
            "SELECT id, store_id, email, name, subscribed_to_newsletter, created_at FROM contacts WHERE store_id = ? ORDER BY created_at DESC",
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_contact(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Mailing list operations
    pub async fn create_mailing_list(&self, store_id: i64, name: &str, description: Option<&str>) -> Result<MailingList> {
        let result = sqlx::query_as::<_, MailingList>(
            r#"
            INSERT INTO mailing_lists (store_id, name, description)
            VALUES (?, ?, ?)
            RETURNING id, store_id, name, description, created_at
            "#,
        )
        .bind(store_id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_mailing_list_by_id(&self, id: i64) -> Result<Option<MailingList>> {
        let result = sqlx::query_as::<_, MailingList>(
            "SELECT id, store_id, name, description, created_at FROM mailing_lists WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn list_mailing_lists_by_store(&self, store_id: i64) -> Result<Vec<MailingList>> {
        let result = sqlx::query_as::<_, MailingList>(
            "SELECT id, store_id, name, description, created_at FROM mailing_lists WHERE store_id = ? ORDER BY created_at DESC",
        )
        .bind(store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_mailing_list(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM mailing_lists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_to_mailing_list(&self, mailing_list_id: i64, contact_id: i64) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO mailing_list_subscribers (mailing_list_id, contact_id) VALUES (?, ?)",
        )
        .bind(mailing_list_id)
        .bind(contact_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_from_mailing_list(&self, mailing_list_id: i64, contact_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM mailing_list_subscribers WHERE mailing_list_id = ? AND contact_id = ?")
            .bind(mailing_list_id)
            .bind(contact_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_mailing_list_subscribers(&self, mailing_list_id: i64) -> Result<Vec<Contact>> {
        let result = sqlx::query_as::<_, Contact>(
            r#"
            SELECT c.id, c.store_id, c.email, c.name, c.subscribed_to_newsletter, c.created_at
            FROM contacts c
            JOIN mailing_list_subscribers mls ON c.id = mls.contact_id
            WHERE mls.mailing_list_id = ?
            ORDER BY mls.subscribed_at DESC
            "#,
        )
        .bind(mailing_list_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}
