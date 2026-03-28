use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use rand::RngCore;
use std::path::PathBuf;
use std::sync::Mutex;

/// Encrypted key prefix to identify encrypted values
const ENC_PREFIX: &str = "ENC:";

/// Key file name
const KEY_FILE: &str = ".key";

/// Nonce length in bytes (12 bytes for AES-GCM)
const NONCE_LEN: usize = 12;

/// Global encryption key storage
static ENCRYPTION_KEY: Lazy<Mutex<Option<[u8; 32]>>> = Lazy::new(|| Mutex::new(None));

fn get_key_path() -> PathBuf {
    crate::get_app_data_dir().join(KEY_FILE)
}

/// Generate a new random 32-byte encryption key
fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    key
}

/// Set file permissions to 600 (owner read/write only) on Unix
#[cfg(unix)]
fn set_secure_permissions(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("Failed to set key file permissions: {}", e))
}

#[cfg(windows)]
fn set_secure_permissions(_path: &std::path::Path) -> Result<(), String> {
    // Windows uses ACLs, simplified handling
    Ok(())
}

/// Get or create the encryption key
///
/// The key is stored in a file with 600 permissions.
/// If the file doesn't exist, a new key is generated.
pub fn get_or_create_encryption_key() -> Result<[u8; 32], String> {
    // Check if key is already loaded in memory
    {
        let key_lock = ENCRYPTION_KEY.lock().map_err(|e| e.to_string())?;
        if let Some(key) = key_lock.as_ref() {
            return Ok(*key);
        }
    }

    let key_path = get_key_path();
    let key_dir = key_path.parent().ok_or("Invalid key path")?;

    // Ensure directory exists
    std::fs::create_dir_all(key_dir)
        .map_err(|e| format!("Failed to create key directory: {}", e))?;

    let key = if key_path.exists() {
        // Load existing key
        let key_bytes =
            std::fs::read(&key_path).map_err(|e| format!("Failed to read key file: {}", e))?;

        if key_bytes.len() != 32 {
            return Err("Invalid key file: expected 32 bytes".to_string());
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        key
    } else {
        // Generate new key
        let key = generate_key();
        std::fs::write(&key_path, key).map_err(|e| format!("Failed to write key file: {}", e))?;

        set_secure_permissions(&key_path)?;
        tracing::info!("Generated new encryption key at {:?}", key_path);
        key
    };

    // Cache in memory
    {
        let mut key_lock = ENCRYPTION_KEY.lock().map_err(|e| e.to_string())?;
        *key_lock = Some(key);
    }

    Ok(key)
}

/// Check if a value is already encrypted
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENC_PREFIX)
}

/// Encrypt an API key
///
/// Returns a Base64-encoded string prefixed with "ENC:"
pub fn encrypt_api_key(plain: &str) -> Result<String, String> {
    if plain.is_empty() {
        return Ok(String::new());
    }

    let key = get_or_create_encryption_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine nonce + ciphertext and encode
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(format!("{}{}", ENC_PREFIX, BASE64.encode(&combined)))
}

/// Decrypt an API key
///
/// Takes a Base64-encoded string prefixed with "ENC:" and returns the plain text
pub fn decrypt_api_key(encrypted: &str) -> Result<String, String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    if !is_encrypted(encrypted) {
        // Not encrypted, return as-is (for migration compatibility)
        return Ok(encrypted.to_string());
    }

    let encrypted_part = encrypted
        .strip_prefix(ENC_PREFIX)
        .ok_or("Invalid encrypted format")?;

    let key = get_or_create_encryption_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    // Decode Base64
    let combined = BASE64
        .decode(encrypted_part)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if combined.len() < NONCE_LEN {
        return Err("Invalid encrypted data: too short".to_string());
    }

    // Split nonce and ciphertext
    let nonce = Nonce::from_slice(&combined[..NONCE_LEN]);
    let ciphertext = &combined[NONCE_LEN..];

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Migrate a plain text API key to encrypted storage
///
/// Returns true if migration was performed
pub fn migrate_plain_api_key(plain: &str) -> Result<Option<String>, String> {
    if plain.is_empty() {
        return Ok(None);
    }

    // Check if already encrypted
    if is_encrypted(plain) {
        return Ok(None);
    }

    // Encrypt the plain key
    let encrypted = encrypt_api_key(plain)?;
    tracing::info!("Migrated plain API key to encrypted storage");
    Ok(Some(encrypted))
}

/// Securely zero out a string in memory (best effort)
pub fn secure_zero_string(s: &mut String) {
    // This is a best-effort approach; Rust's String doesn't guarantee
    // the memory won't be moved or copied elsewhere
    unsafe {
        std::ptr::write_volatile(s.as_mut_ptr(), 0);
        for i in 1..s.len() {
            std::ptr::write_volatile(s.as_mut_ptr().add(i), 0);
        }
    }
    s.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_encrypt_decrypt_roundtrip() {
        let plain = "sk-test-api-key-12345";
        let encrypted = encrypt_api_key(plain).unwrap();

        assert!(is_encrypted(&encrypted));
        assert_ne!(encrypted, plain);

        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, plain);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let encrypted = encrypt_api_key("").unwrap();
        assert_eq!(encrypted, "");
    }

    #[test]
    fn test_decrypt_empty_string() {
        let decrypted = decrypt_api_key("").unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("ENC:something"));
        assert!(!is_encrypted("sk-test"));
        assert!(!is_encrypted(""));
    }

    #[test]
    #[serial]
    fn test_migrate_plain_key() {
        let plain = "sk-plain-key";
        let result = migrate_plain_api_key(plain).unwrap();

        assert!(result.is_some());
        let encrypted = result.unwrap();
        assert!(is_encrypted(&encrypted));

        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, plain);
    }

    #[test]
    #[serial]
    fn test_migrate_already_encrypted() {
        let plain = "sk-test";
        let encrypted = encrypt_api_key(plain).unwrap();

        // Already encrypted, should return None
        let result = migrate_plain_api_key(&encrypted).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_migrate_empty_key() {
        let result = migrate_plain_api_key("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_different_keys_produce_different_ciphertext() {
        let plain = "sk-test-key";

        let encrypted1 = encrypt_api_key(plain).unwrap();
        let encrypted2 = encrypt_api_key(plain).unwrap();

        // Due to random nonce, same plaintext should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(decrypt_api_key(&encrypted1).unwrap(), plain);
        assert_eq!(decrypt_api_key(&encrypted2).unwrap(), plain);
    }

    #[test]
    fn test_decrypt_unencrypted_returns_as_is() {
        // For migration compatibility, decrypting an unencrypted value returns it as-is
        let plain = "sk-unencrypted-key";
        let result = decrypt_api_key(plain).unwrap();
        assert_eq!(result, plain);
    }

    #[test]
    fn test_secure_zero_string() {
        let mut s = String::from("sensitive-data-12345");
        let _len = s.len();

        secure_zero_string(&mut s);

        assert!(s.is_empty());
        // Note: We can't easily verify the memory was zeroed due to Rust's safety model
    }

    #[test]
    #[serial]
    fn test_special_characters() {
        let plain = "sk-!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let encrypted = encrypt_api_key(plain).unwrap();
        let decrypted = decrypt_api_key(&encrypted).unwrap();

        assert_eq!(decrypted, plain);
    }

    #[test]
    #[serial]
    fn test_long_key() {
        let plain = "sk-".to_string() + &"x".repeat(1000);
        let encrypted = encrypt_api_key(&plain).unwrap();
        let decrypted = decrypt_api_key(&encrypted).unwrap();

        assert_eq!(decrypted, plain);
    }

    // ── Platform-specific file permission tests (CORE-008 Task 2.3) ──

    #[test]
    #[cfg(unix)]
    fn test_unix_set_secure_permissions_sets_600() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::NamedTempFile::new().unwrap();
        let result = set_secure_permissions(temp.path());
        assert!(
            result.is_ok(),
            "set_secure_permissions should succeed on Unix"
        );

        let perms = std::fs::metadata(temp.path()).unwrap().permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "File permissions should be 600 (owner read/write only)"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_unix_set_secure_permissions_fails_on_invalid_path() {
        let result = set_secure_permissions(std::path::Path::new("/nonexistent/path/.key"));
        assert!(
            result.is_err(),
            "set_secure_permissions should fail for non-existent path"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_set_secure_permissions_is_noop() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let result = set_secure_permissions(temp.path());
        assert!(
            result.is_ok(),
            "Windows set_secure_permissions should always succeed (no-op)"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_set_secure_permissions_noop_on_nonexistent_path() {
        // Windows no-op should succeed even for non-existent paths
        let result = set_secure_permissions(std::path::Path::new("C:\\nonexistent\\path\\.key"));
        assert!(
            result.is_ok(),
            "Windows no-op should succeed regardless of path validity"
        );
    }

    #[test]
    #[serial]
    fn test_encryption_key_roundtrip_across_platforms() {
        // Verify encryption works consistently regardless of platform
        let long_key = "a".repeat(256);
        let test_cases = vec![
            "simple-key",
            "key-with-unicode-密钥",
            "key with spaces and !@#$%",
            &long_key,
        ];

        for plain in test_cases {
            let encrypted = encrypt_api_key(plain).unwrap();
            assert!(is_encrypted(&encrypted), "Should be marked encrypted");
            let decrypted = decrypt_api_key(&encrypted).unwrap();
            assert_eq!(
                decrypted,
                plain,
                "Roundtrip failed for: {}",
                &plain[..plain.len().min(20)]
            );
        }
    }

    #[test]
    fn test_key_path_uses_platform_appropriate_separator() {
        let key_path = get_key_path();
        let path_str = key_path.to_string_lossy();

        // Verify the path uses the correct separator for this platform
        assert!(
            path_str.contains("DailyLogger"),
            "Key path should contain app name"
        );
        assert!(path_str.ends_with(".key"), "Key path should end with .key");

        // PathBuf should use platform-native separators
        #[cfg(target_os = "windows")]
        assert!(
            path_str.contains('\\'),
            "Windows paths should use backslash"
        );
    }
}
