use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::memory_storage::DB_CONNECTION;

/// User account for team collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// User registration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

/// User login parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

/// Result of user authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub user: Option<User>,
    pub message: String,
}

/// Session info stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub username: String,
    pub logged_in_at: DateTime<Utc>,
}

/// Create the sessions table for session persistence
pub fn create_sessions_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            user_id TEXT NOT NULL,
            username TEXT NOT NULL,
            logged_in_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create sessions table: {}", e))?;

    Ok(())
}

/// Save session to local storage (single session model)
pub fn save_session(user: &User) -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let logged_in_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO sessions (id, user_id, username, logged_in_at) VALUES (1, ?1, ?2, ?3)",
        params![user.id, user.username, logged_in_at],
    )
    .map_err(|e| format!("Failed to save session: {}", e))?;

    Ok(())
}

/// Get current session from local storage
#[tauri::command]
pub fn get_current_session() -> Result<Option<Session>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let result: Option<Session> = conn
        .query_row(
            "SELECT user_id, username, logged_in_at FROM sessions WHERE id = 1",
            [],
            |row| {
                Ok(Session {
                    user_id: row.get(0)?,
                    username: row.get(1)?,
                    logged_in_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result)
}

/// Logout - clear session from local storage
#[tauri::command]
pub fn logout() -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    conn.execute("DELETE FROM sessions WHERE id = 1", [])
        .map_err(|e| format!("Failed to logout: {}", e))?;

    Ok(())
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Password hashing failed: {}", e))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| format!("Invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a unique user ID
fn generate_user_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!(
        "user_{:016x}",
        rng.sample(rand::distributions::Uniform::new(0u64, u64::MAX))
    )
}

/// Create the users table
pub fn create_users_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create users table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)",
        [],
    )
    .map_err(|e| format!("Failed to create users index: {}", e))?;

    Ok(())
}

/// Register a new user
#[tauri::command]
pub fn register_user(params: RegisterParams) -> Result<User, String> {
    // Validate username
    if params.username.len() < 3 || params.username.len() > 32 {
        return Err("Username must be 3-32 characters".to_string());
    }

    if !params
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        );
    }

    // Validate password
    if params.password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    // Hash password
    let password_hash = hash_password(&params.password)?;

    // Generate user ID
    let user_id = generate_user_id();
    let created_at = Utc::now().to_rfc3339();

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    conn.execute(
        "INSERT INTO users (id, username, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user_id, params.username, params.email, password_hash, created_at],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed: users.username") {
            "Username already exists".to_string()
        } else if e.to_string().contains("UNIQUE constraint failed: users.email") {
            "Email already registered".to_string()
        } else {
            format!("Failed to create user: {}", e)
        }
    })?;

    Ok(User {
        id: user_id,
        username: params.username,
        email: params.email,
        created_at: Utc::now(),
    })
}

/// Login a user
#[tauri::command]
pub fn login_user(params: LoginParams) -> Result<User, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let result: Option<(String, String, Option<String>, String)> = conn
        .query_row(
            "SELECT id, username, email, password_hash FROM users WHERE username = ?1",
            params![params.username],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    let (user_id, username, email, password_hash) = result.ok_or("Invalid username or password")?;

    if !verify_password(&params.password, &password_hash)? {
        return Err("Invalid username or password".to_string());
    }

    let user = User {
        id: user_id,
        username,
        email,
        created_at: Utc::now(), // We don't store this in the return, just use current time
    };

    // Save session after successful login
    drop(db); // Release lock before save_session
    save_session(&user)?;

    Ok(user)
}

/// Get user by ID
#[tauri::command]
pub fn get_user_by_id(user_id: &str) -> Result<Option<User>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let result: Option<User> = conn
        .query_row(
            "SELECT id, username, email, created_at FROM users WHERE id = ?1",
            params![user_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result)
}

/// Get all users (for admin purposes)
#[tauri::command]
pub fn get_all_users() -> Result<Vec<User>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT id, username, email, created_at FROM users ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| format!("Failed to query users: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect users: {}", e))?;

    Ok(users)
}

/// Delete a user
#[tauri::command]
pub fn delete_user(user_id: &str) -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute("DELETE FROM users WHERE id = ?1", params![user_id])
        .map_err(|e| format!("Failed to delete user: {}", e))?;

    Ok(rows_affected > 0)
}

/// Check if any user exists (for first-run detection)
#[tauri::command]
pub fn has_any_user() -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use serial_test::serial;

    fn setup_test_db() {
        let conn = Connection::open_in_memory().unwrap();
        create_users_table(&conn).unwrap();
        create_sessions_table(&conn).unwrap();

        // Initialize global DB_CONNECTION for tests
        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert_ne!(hash, password);
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_hash_password_different_salts() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify the same password
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    #[serial]
    fn test_register_user_success() {
        setup_test_db();

        let params = RegisterParams {
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
        };

        let user = register_user(params).unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.id.starts_with("user_"));
    }

    #[test]
    #[serial]
    fn test_register_user_short_username() {
        setup_test_db();

        let params = RegisterParams {
            username: "ab".to_string(),
            email: None,
            password: "password123".to_string(),
        };

        let result = register_user(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("3-32 characters"));
    }

    #[test]
    #[serial]
    fn test_register_user_invalid_username_chars() {
        setup_test_db();

        let params = RegisterParams {
            username: "test@user".to_string(),
            email: None,
            password: "password123".to_string(),
        };

        let result = register_user(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters, numbers"));
    }

    #[test]
    #[serial]
    fn test_register_user_short_password() {
        setup_test_db();

        let params = RegisterParams {
            username: "testuser".to_string(),
            email: None,
            password: "short".to_string(),
        };

        let result = register_user(params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("8 characters"));
    }

    #[test]
    #[serial]
    fn test_register_user_duplicate_username() {
        setup_test_db();

        let params1 = RegisterParams {
            username: "testuser".to_string(),
            email: Some("test1@example.com".to_string()),
            password: "password123".to_string(),
        };
        register_user(params1).unwrap();

        let params2 = RegisterParams {
            username: "testuser".to_string(),
            email: Some("test2@example.com".to_string()),
            password: "password456".to_string(),
        };

        let result = register_user(params2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    #[serial]
    fn test_login_user_success() {
        setup_test_db();

        let register_params = RegisterParams {
            username: "loginuser".to_string(),
            email: None,
            password: "correctpassword".to_string(),
        };
        register_user(register_params).unwrap();

        let login_params = LoginParams {
            username: "loginuser".to_string(),
            password: "correctpassword".to_string(),
        };

        let user = login_user(login_params).unwrap();
        assert_eq!(user.username, "loginuser");
    }

    #[test]
    #[serial]
    fn test_login_user_wrong_password() {
        setup_test_db();

        let register_params = RegisterParams {
            username: "loginuser".to_string(),
            email: None,
            password: "correctpassword".to_string(),
        };
        register_user(register_params).unwrap();

        let login_params = LoginParams {
            username: "loginuser".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = login_user(login_params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid"));
    }

    #[test]
    #[serial]
    fn test_login_user_nonexistent() {
        setup_test_db();

        let login_params = LoginParams {
            username: "nonexistent".to_string(),
            password: "anypassword".to_string(),
        };

        let result = login_user(login_params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid"));
    }

    #[test]
    #[serial]
    fn test_get_user_by_id() {
        setup_test_db();

        let params = RegisterParams {
            username: "finduser".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        let created = register_user(params).unwrap();

        let found = get_user_by_id(&created.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "finduser");
    }

    #[test]
    #[serial]
    fn test_get_user_by_id_not_found() {
        setup_test_db();

        let found = get_user_by_id("user_nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    #[serial]
    fn test_has_any_user() {
        setup_test_db();

        assert!(!has_any_user().unwrap());

        let params = RegisterParams {
            username: "firstuser".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        register_user(params).unwrap();

        assert!(has_any_user().unwrap());
    }

    #[test]
    #[serial]
    fn test_delete_user() {
        setup_test_db();

        let params = RegisterParams {
            username: "deleteuser".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        let user = register_user(params).unwrap();

        assert!(delete_user(&user.id).unwrap());
        assert!(!delete_user(&user.id).unwrap()); // Already deleted
    }

    #[test]
    #[serial]
    fn test_get_all_users() {
        setup_test_db();

        let params1 = RegisterParams {
            username: "user1".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        register_user(params1).unwrap();

        let params2 = RegisterParams {
            username: "user2".to_string(),
            email: None,
            password: "password456".to_string(),
        };
        register_user(params2).unwrap();

        let users = get_all_users().unwrap();
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_generate_user_id_format() {
        let id = generate_user_id();
        assert!(id.starts_with("user_"));
        assert_eq!(id.len(), 21); // "user_" + 16 hex chars
    }

    #[test]
    #[serial]
    fn test_save_and_get_session() {
        setup_test_db();

        let params = RegisterParams {
            username: "sessionuser".to_string(),
            email: Some("session@example.com".to_string()),
            password: "password123".to_string(),
        };
        let user = register_user(params).unwrap();

        // Initially no session
        assert!(get_current_session().unwrap().is_none());

        // Save session
        save_session(&user).unwrap();

        // Get session
        let session = get_current_session().unwrap();
        assert!(session.is_some());
        let session = session.unwrap();
        assert_eq!(session.user_id, user.id);
        assert_eq!(session.username, "sessionuser");
    }

    #[test]
    #[serial]
    fn test_logout() {
        setup_test_db();

        let params = RegisterParams {
            username: "logoutuser".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        let user = register_user(params).unwrap();

        save_session(&user).unwrap();
        assert!(get_current_session().unwrap().is_some());

        logout().unwrap();
        assert!(get_current_session().unwrap().is_none());
    }

    #[test]
    #[serial]
    fn test_login_creates_session() {
        setup_test_db();

        let register_params = RegisterParams {
            username: "loginsession".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        register_user(register_params).unwrap();

        // No session initially
        assert!(get_current_session().unwrap().is_none());

        // Login creates session
        let login_params = LoginParams {
            username: "loginsession".to_string(),
            password: "password123".to_string(),
        };
        login_user(login_params).unwrap();

        // Session should now exist
        let session = get_current_session().unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().username, "loginsession");
    }

    #[test]
    #[serial]
    fn test_session_overwrites_on_new_login() {
        setup_test_db();

        // Create two users
        let params1 = RegisterParams {
            username: "user1".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        register_user(params1).unwrap();

        let params2 = RegisterParams {
            username: "user2".to_string(),
            email: None,
            password: "password456".to_string(),
        };
        register_user(params2).unwrap();

        // Login as user1
        let login1 = LoginParams {
            username: "user1".to_string(),
            password: "password123".to_string(),
        };
        login_user(login1).unwrap();
        assert_eq!(get_current_session().unwrap().unwrap().username, "user1");

        // Login as user2 (should overwrite)
        let login2 = LoginParams {
            username: "user2".to_string(),
            password: "password456".to_string(),
        };
        login_user(login2).unwrap();
        assert_eq!(get_current_session().unwrap().unwrap().username, "user2");
    }
}
