use std::path::Path;

use sqlx::{Row, SqlitePool};

use crate::http_server::{IssueInfo, IssueCommentInfo, PullRequestInfo, PullRequestCommentInfo, OrganizationInfo};

/// Database manager for git repositories
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the SQLite database
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database schema
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        // Organizations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS organizations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Repositories table with forked_from and org_name
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                org_name TEXT NOT NULL,
                path TEXT NOT NULL,
                forked_from TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(org_name, name),
                FOREIGN KEY (org_name) REFERENCES organizations(name)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add forked_from column if it doesn't exist (migration)
        let _ = sqlx::query("ALTER TABLE repositories ADD COLUMN forked_from TEXT")
            .execute(&self.pool)
            .await;

        // Add org_name column if it doesn't exist (migration)
        let _ = sqlx::query("ALTER TABLE repositories ADD COLUMN org_name TEXT")
            .execute(&self.pool)
            .await;

        // Issues table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_name TEXT NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                state TEXT NOT NULL DEFAULT 'open',
                author TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(repo_name, number)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Issue comments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS issue_comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                issue_id INTEGER NOT NULL,
                body TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (issue_id) REFERENCES issues(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Pull requests table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pull_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_name TEXT NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                state TEXT NOT NULL DEFAULT 'open',
                source_repo TEXT NOT NULL,
                source_branch TEXT NOT NULL,
                target_branch TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(repo_name, number)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Pull request comments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pr_comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pr_id INTEGER NOT NULL,
                body TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (pr_id) REFERENCES pull_requests(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============ Organization Methods ============

    /// Create a new organization
    pub async fn create_organization(&self, name: &str, display_name: &str, description: &str) -> Result<OrganizationInfo, sqlx::Error> {
        let result = sqlx::query("INSERT INTO organizations (name, display_name, description) VALUES (?, ?, ?)")
            .bind(name)
            .bind(display_name)
            .bind(description)
            .execute(&self.pool)
            .await?;
        
        Ok(OrganizationInfo {
            id: result.last_insert_rowid(),
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            created_at: chrono_now(),
        })
    }

    /// Get organization by name
    pub async fn get_organization(&self, name: &str) -> Result<Option<OrganizationInfo>, sqlx::Error> {
        let row = sqlx::query("SELECT id, name, display_name, description, created_at FROM organizations WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| OrganizationInfo {
            id: r.get("id"),
            name: r.get("name"),
            display_name: r.get("display_name"),
            description: r.get("description"),
            created_at: r.get("created_at"),
        }))
    }

    /// List all organizations
    pub async fn list_organizations(&self) -> Result<Vec<OrganizationInfo>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, name, display_name, description, created_at FROM organizations ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| OrganizationInfo {
                id: r.get("id"),
                name: r.get("name"),
                display_name: r.get("display_name"),
                description: r.get("description"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// Update an organization
    pub async fn update_organization(
        &self,
        name: &str,
        display_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Option<OrganizationInfo>, sqlx::Error> {
        let mut updates = Vec::new();
        if display_name.is_some() {
            updates.push("display_name = ?");
        }
        if description.is_some() {
            updates.push("description = ?");
        }
        
        if updates.is_empty() {
            return self.get_organization(name).await;
        }

        let query = format!(
            "UPDATE organizations SET {} WHERE name = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        if let Some(dn) = display_name {
            q = q.bind(dn);
        }
        if let Some(d) = description {
            q = q.bind(d);
        }
        q = q.bind(name);
        q.execute(&self.pool).await?;

        self.get_organization(name).await
    }

    // ============ Repository Methods ============

    /// Create a new repository entry
    pub async fn create_repository(&self, org_name: &str, name: &str, path: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO repositories (org_name, name, path) VALUES (?, ?, ?)")
            .bind(org_name)
            .bind(name)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    /// Create a new repository entry with fork info
    pub async fn create_repository_with_fork(&self, org_name: &str, name: &str, path: &str, forked_from: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO repositories (org_name, name, path, forked_from) VALUES (?, ?, ?, ?)")
            .bind(org_name)
            .bind(name)
            .bind(path)
            .bind(forked_from)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get repository by org and name
    pub async fn get_repository(&self, org_name: &str, name: &str) -> Result<Option<Repository>, sqlx::Error> {
        let row = sqlx::query("SELECT id, org_name, name, path, forked_from FROM repositories WHERE org_name = ? AND name = ?")
            .bind(org_name)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Repository {
            id: r.get("id"),
            org_name: r.get("org_name"),
            name: r.get("name"),
            path: r.get("path"),
            forked_from: r.get("forked_from"),
        }))
    }

    /// List all repositories for an organization
    pub async fn list_repositories(&self, org_name: &str) -> Result<Vec<Repository>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, org_name, name, path, forked_from FROM repositories WHERE org_name = ? ORDER BY name")
            .bind(org_name)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| Repository {
                id: r.get("id"),
                org_name: r.get("org_name"),
                name: r.get("name"),
                path: r.get("path"),
                forked_from: r.get("forked_from"),
            })
            .collect())
    }

    /// List all repositories across all organizations
    #[allow(dead_code)]
    pub async fn list_all_repositories(&self) -> Result<Vec<Repository>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, org_name, name, path, forked_from FROM repositories ORDER BY org_name, name")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| Repository {
                id: r.get("id"),
                org_name: r.get("org_name"),
                name: r.get("name"),
                path: r.get("path"),
                forked_from: r.get("forked_from"),
            })
            .collect())
    }

    // ============ Issue Methods ============

    /// Get next issue number for a repository
    async fn next_issue_number(&self, repo_name: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COALESCE(MAX(number), 0) + 1 as next FROM issues WHERE repo_name = ?")
            .bind(repo_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("next"))
    }

    /// Create a new issue
    pub async fn create_issue(&self, repo_name: &str, title: &str, body: &str, author: &str) -> Result<IssueInfo, sqlx::Error> {
        let number = self.next_issue_number(repo_name).await?;
        
        let result = sqlx::query(
            "INSERT INTO issues (repo_name, number, title, body, author) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(repo_name)
        .bind(number)
        .bind(title)
        .bind(body)
        .bind(author)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        
        Ok(IssueInfo {
            id,
            repo_name: repo_name.to_string(),
            number,
            title: title.to_string(),
            body: body.to_string(),
            state: "open".to_string(),
            author: author.to_string(),
            created_at: chrono_now(),
            updated_at: chrono_now(),
        })
    }

    /// Get issue by repo and number
    pub async fn get_issue(&self, repo_name: &str, number: i64) -> Result<Option<IssueInfo>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, repo_name, number, title, body, state, author, created_at, updated_at FROM issues WHERE repo_name = ? AND number = ?"
        )
        .bind(repo_name)
        .bind(number)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| IssueInfo {
            id: r.get("id"),
            repo_name: r.get("repo_name"),
            number: r.get("number"),
            title: r.get("title"),
            body: r.get("body"),
            state: r.get("state"),
            author: r.get("author"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// List issues for a repository
    pub async fn list_issues(&self, repo_name: &str) -> Result<Vec<IssueInfo>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, repo_name, number, title, body, state, author, created_at, updated_at FROM issues WHERE repo_name = ? ORDER BY number DESC"
        )
        .bind(repo_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| IssueInfo {
                id: r.get("id"),
                repo_name: r.get("repo_name"),
                number: r.get("number"),
                title: r.get("title"),
                body: r.get("body"),
                state: r.get("state"),
                author: r.get("author"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Update an issue
    pub async fn update_issue(
        &self,
        repo_name: &str,
        number: i64,
        title: Option<&str>,
        body: Option<&str>,
        state: Option<&str>,
    ) -> Result<Option<IssueInfo>, sqlx::Error> {
        // Build dynamic update query
        let mut updates = Vec::new();
        if title.is_some() {
            updates.push("title = ?");
        }
        if body.is_some() {
            updates.push("body = ?");
        }
        if state.is_some() {
            updates.push("state = ?");
        }
        
        if updates.is_empty() {
            return self.get_issue(repo_name, number).await;
        }

        updates.push("updated_at = CURRENT_TIMESTAMP");
        let query = format!(
            "UPDATE issues SET {} WHERE repo_name = ? AND number = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        if let Some(t) = title {
            q = q.bind(t);
        }
        if let Some(b) = body {
            q = q.bind(b);
        }
        if let Some(s) = state {
            q = q.bind(s);
        }
        q = q.bind(repo_name).bind(number);
        q.execute(&self.pool).await?;

        self.get_issue(repo_name, number).await
    }

    /// List comments for an issue
    pub async fn list_issue_comments(&self, issue_id: i64) -> Result<Vec<IssueCommentInfo>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, issue_id, body, author, created_at FROM issue_comments WHERE issue_id = ? ORDER BY created_at ASC"
        )
        .bind(issue_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| IssueCommentInfo {
                id: r.get("id"),
                issue_id: r.get("issue_id"),
                body: r.get("body"),
                author: r.get("author"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// Create a comment on an issue
    pub async fn create_issue_comment(&self, issue_id: i64, body: &str, author: &str) -> Result<IssueCommentInfo, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO issue_comments (issue_id, body, author) VALUES (?, ?, ?)"
        )
        .bind(issue_id)
        .bind(body)
        .bind(author)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        
        Ok(IssueCommentInfo {
            id,
            issue_id,
            body: body.to_string(),
            author: author.to_string(),
            created_at: chrono_now(),
        })
    }

    // ============ Pull Request Methods ============

    /// Get next PR number for a repository
    async fn next_pr_number(&self, repo_name: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COALESCE(MAX(number), 0) + 1 as next FROM pull_requests WHERE repo_name = ?")
            .bind(repo_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("next"))
    }

    /// Create a new pull request
    pub async fn create_pull_request(
        &self,
        repo_name: &str,
        title: &str,
        body: &str,
        source_repo: &str,
        source_branch: &str,
        target_branch: &str,
        author: &str,
    ) -> Result<PullRequestInfo, sqlx::Error> {
        let number = self.next_pr_number(repo_name).await?;
        
        let result = sqlx::query(
            "INSERT INTO pull_requests (repo_name, number, title, body, source_repo, source_branch, target_branch, author) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(repo_name)
        .bind(number)
        .bind(title)
        .bind(body)
        .bind(source_repo)
        .bind(source_branch)
        .bind(target_branch)
        .bind(author)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        
        Ok(PullRequestInfo {
            id,
            repo_name: repo_name.to_string(),
            number,
            title: title.to_string(),
            body: body.to_string(),
            state: "open".to_string(),
            source_repo: source_repo.to_string(),
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            author: author.to_string(),
            created_at: chrono_now(),
            updated_at: chrono_now(),
        })
    }

    /// Get pull request by repo and number
    pub async fn get_pull_request(&self, repo_name: &str, number: i64) -> Result<Option<PullRequestInfo>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, repo_name, number, title, body, state, source_repo, source_branch, target_branch, author, created_at, updated_at FROM pull_requests WHERE repo_name = ? AND number = ?"
        )
        .bind(repo_name)
        .bind(number)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| PullRequestInfo {
            id: r.get("id"),
            repo_name: r.get("repo_name"),
            number: r.get("number"),
            title: r.get("title"),
            body: r.get("body"),
            state: r.get("state"),
            source_repo: r.get("source_repo"),
            source_branch: r.get("source_branch"),
            target_branch: r.get("target_branch"),
            author: r.get("author"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// List pull requests for a repository
    pub async fn list_pull_requests(&self, repo_name: &str) -> Result<Vec<PullRequestInfo>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, repo_name, number, title, body, state, source_repo, source_branch, target_branch, author, created_at, updated_at FROM pull_requests WHERE repo_name = ? ORDER BY number DESC"
        )
        .bind(repo_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PullRequestInfo {
                id: r.get("id"),
                repo_name: r.get("repo_name"),
                number: r.get("number"),
                title: r.get("title"),
                body: r.get("body"),
                state: r.get("state"),
                source_repo: r.get("source_repo"),
                source_branch: r.get("source_branch"),
                target_branch: r.get("target_branch"),
                author: r.get("author"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Update a pull request
    pub async fn update_pull_request(
        &self,
        repo_name: &str,
        number: i64,
        title: Option<&str>,
        body: Option<&str>,
        state: Option<&str>,
    ) -> Result<Option<PullRequestInfo>, sqlx::Error> {
        let mut updates = Vec::new();
        if title.is_some() {
            updates.push("title = ?");
        }
        if body.is_some() {
            updates.push("body = ?");
        }
        if state.is_some() {
            updates.push("state = ?");
        }
        
        if updates.is_empty() {
            return self.get_pull_request(repo_name, number).await;
        }

        updates.push("updated_at = CURRENT_TIMESTAMP");
        let query = format!(
            "UPDATE pull_requests SET {} WHERE repo_name = ? AND number = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        if let Some(t) = title {
            q = q.bind(t);
        }
        if let Some(b) = body {
            q = q.bind(b);
        }
        if let Some(s) = state {
            q = q.bind(s);
        }
        q = q.bind(repo_name).bind(number);
        q.execute(&self.pool).await?;

        self.get_pull_request(repo_name, number).await
    }

    /// List comments for a pull request
    pub async fn list_pr_comments(&self, pr_id: i64) -> Result<Vec<PullRequestCommentInfo>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, pr_id, body, author, created_at FROM pr_comments WHERE pr_id = ? ORDER BY created_at ASC"
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PullRequestCommentInfo {
                id: r.get("id"),
                pr_id: r.get("pr_id"),
                body: r.get("body"),
                author: r.get("author"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// Create a comment on a pull request
    pub async fn create_pr_comment(&self, pr_id: i64, body: &str, author: &str) -> Result<PullRequestCommentInfo, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO pr_comments (pr_id, body, author) VALUES (?, ?, ?)"
        )
        .bind(pr_id)
        .bind(body)
        .bind(author)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        
        Ok(PullRequestCommentInfo {
            id,
            pr_id,
            body: body.to_string(),
            author: author.to_string(),
            created_at: chrono_now(),
        })
    }
}

/// Get current timestamp as string
fn chrono_now() -> String {
    // Simple timestamp without external dependency
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string())
}

/// Repository model
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Repository {
    pub id: i64,
    pub org_name: String,
    pub name: String,
    pub path: String,
    pub forked_from: Option<String>,
}

/// Initialize a bare git repository on disk
pub async fn init_bare_repo(path: &Path) -> Result<(), std::io::Error> {
    let status = tokio::process::Command::new("git")
        .args(["init", "--bare"])
        .arg(path)
        .status()
        .await?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to initialize bare git repository",
        ));
    }

    Ok(())
}
