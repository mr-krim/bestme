use crate::ai::{Result, AIError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::{
    core::PWSTR,
    Win32::Security::Credentials::{
        CredReadW, CredWriteW, CredDeleteW, CREDENTIALW, CRED_TYPE_GENERIC,
        CRED_PERSIST_LOCAL_MACHINE,
    },
};

#[cfg(target_os = "macos")]
use security_framework::passwords::{get_generic_password, set_generic_password, delete_generic_password};

#[cfg(target_os = "linux")]
use secret_service::{EncryptionType, SecretService};

const SERVICE_NAME: &str = "BestMe";
const API_KEY_PREFIX: &str = "api_key_";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureApiKey {
    pub provider: String,
    pub key_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ApiKeyManager {
    keys_metadata: HashMap<String, SecureApiKey>,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            keys_metadata: HashMap::new(),
        }
    }

    pub async fn store_api_key(
        &mut self,
        provider: &str,
        api_key: &str,
    ) -> Result<()> {
        let key_id = format!("{}{}", API_KEY_PREFIX, provider);
        
        // Store in OS keychain
        #[cfg(target_os = "windows")]
        self.store_windows(&key_id, api_key)?;
        
        #[cfg(target_os = "macos")]
        self.store_macos(&key_id, api_key)?;
        
        #[cfg(target_os = "linux")]
        self.store_linux(&key_id, api_key).await?;
        
        // Store metadata
        let metadata = SecureApiKey {
            provider: provider.to_string(),
            key_id: key_id.clone(),
            created_at: chrono::Utc::now(),
            last_used: None,
        };
        
        self.keys_metadata.insert(provider.to_string(), metadata);
        self.save_metadata()?;
        
        Ok(())
    }

    pub async fn get_api_key(&mut self, provider: &str) -> Result<String> {
        let key_id = format!("{}{}", API_KEY_PREFIX, provider);
        
        // Try OS keychain first
        let api_key = {
            #[cfg(target_os = "windows")]
            { self.get_windows(&key_id)? }
            
            #[cfg(target_os = "macos")]
            { self.get_macos(&key_id)? }
            
            #[cfg(target_os = "linux")]
            { self.get_linux(&key_id).await? }
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            { return Err(AIError::ConfigError("Unsupported platform for keychain".to_string())); }
        };
        
        // Update last used
        if let Some(metadata) = self.keys_metadata.get_mut(provider) {
            metadata.last_used = Some(chrono::Utc::now());
            self.save_metadata()?;
        }
        
        Ok(api_key)
    }

    pub async fn delete_api_key(&mut self, provider: &str) -> Result<()> {
        let key_id = format!("{}{}", API_KEY_PREFIX, provider);
        
        #[cfg(target_os = "windows")]
        self.delete_windows(&key_id)?;
        
        #[cfg(target_os = "macos")]
        self.delete_macos(&key_id)?;
        
        #[cfg(target_os = "linux")]
        self.delete_linux(&key_id).await?;
        
        self.keys_metadata.remove(provider);
        self.save_metadata()?;
        
        Ok(())
    }

    pub fn list_stored_keys(&self) -> Vec<&SecureApiKey> {
        self.keys_metadata.values().collect()
    }

    // Platform-specific implementations
    #[cfg(target_os = "windows")]
    fn store_windows(&self, key_id: &str, api_key: &str) -> Result<()> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        let target_name: Vec<u16> = OsStr::new(key_id).encode_wide().chain(Some(0)).collect();
        let credential_blob = api_key.as_bytes();
        
        let mut credential = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR(target_name.as_ptr() as *mut u16),
            CredentialBlobSize: credential_blob.len() as u32,
            CredentialBlob: credential_blob.as_ptr() as *mut u8,
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            UserName: PWSTR::null(),
            ..Default::default()
        };
        
        unsafe {
            CredWriteW(&mut credential, 0)
                .map_err(|e| AIError::ConfigError(format!("Failed to store key: {:?}", e)))?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn store_macos(&self, key_id: &str, api_key: &str) -> Result<()> {
        set_generic_password(SERVICE_NAME, key_id, api_key.as_bytes())
            .map_err(|e| AIError::ConfigError(format!("Failed to store key: {}", e)))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn store_linux(&self, key_id: &str, api_key: &str) -> Result<()> {
        let ss = SecretService::connect(EncryptionType::Dh)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to connect to secret service: {}", e)))?;
        
        let collection = ss.get_default_collection()
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to get default collection: {}", e)))?;
        
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("application", SERVICE_NAME);
        attributes.insert("service", key_id);
        
        collection.create_item(
            &format!("{} - {}", SERVICE_NAME, key_id),
            attributes,
            api_key.as_bytes(),
            true,
            "text/plain",
        )
        .await
        .map_err(|e| AIError::ConfigError(format!("Failed to store key: {}", e)))?;
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn get_windows(&self, key_id: &str) -> Result<String> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        let target_name: Vec<u16> = OsStr::new(key_id).encode_wide().chain(Some(0)).collect();
        let mut credential_ptr = std::ptr::null_mut();
        
        unsafe {
            CredReadW(windows::core::PCWSTR::from_raw(target_name.as_ptr()), CRED_TYPE_GENERIC, Some(0), &mut credential_ptr)
                .map_err(|e| AIError::ConfigError(format!("Failed to read key: {:?}", e)))?;
            
            let credential = &*credential_ptr;
            let api_key_bytes = std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            );
            
            let api_key = String::from_utf8_lossy(api_key_bytes).to_string();
            
            // Free the credential
            windows::Win32::Security::Credentials::CredFree(credential_ptr as *mut _);
            
            Ok(api_key)
        }
    }

    #[cfg(target_os = "macos")]
    fn get_macos(&self, key_id: &str) -> Result<String> {
        let password = get_generic_password(SERVICE_NAME, key_id)
            .map_err(|e| AIError::ConfigError(format!("Failed to read key: {}", e)))?;
        
        String::from_utf8(password)
            .map_err(|e| AIError::ConfigError(format!("Invalid UTF-8 in stored key: {}", e)))
    }

    #[cfg(target_os = "linux")]
    async fn get_linux(&self, key_id: &str) -> Result<String> {
        let ss = SecretService::connect(EncryptionType::Dh)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to connect to secret service: {}", e)))?;
        
        let mut search_attributes = std::collections::HashMap::new();
        search_attributes.insert("application", SERVICE_NAME);
        search_attributes.insert("service", key_id);
        
        let search_result = ss.search_items(search_attributes)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to search for key: {}", e)))?;
        
        let item = search_result.unlocked.first()
            .ok_or_else(|| AIError::ConfigError("API key not found".to_string()))?;
        
        let secret = item.get_secret()
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to get secret: {}", e)))?;
        
        String::from_utf8(secret)
            .map_err(|e| AIError::ConfigError(format!("Invalid UTF-8 in stored key: {}", e)))
    }

    #[cfg(target_os = "windows")]
    fn delete_windows(&self, key_id: &str) -> Result<()> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        let target_name: Vec<u16> = OsStr::new(key_id).encode_wide().chain(Some(0)).collect();
        
        unsafe {
            CredDeleteW(windows::core::PCWSTR::from_raw(target_name.as_ptr()), CRED_TYPE_GENERIC, Some(0))
                .map_err(|e| AIError::ConfigError(format!("Failed to delete key: {:?}", e)))?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn delete_macos(&self, key_id: &str) -> Result<()> {
        delete_generic_password(SERVICE_NAME, key_id)
            .map_err(|e| AIError::ConfigError(format!("Failed to delete key: {}", e)))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn delete_linux(&self, key_id: &str) -> Result<()> {
        let ss = SecretService::connect(EncryptionType::Dh)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to connect to secret service: {}", e)))?;
        
        let mut search_attributes = std::collections::HashMap::new();
        search_attributes.insert("application", SERVICE_NAME);
        search_attributes.insert("service", key_id);
        
        let search_result = ss.search_items(search_attributes)
            .await
            .map_err(|e| AIError::ConfigError(format!("Failed to search for key: {}", e)))?;
        
        for item in search_result.unlocked {
            item.delete()
                .await
                .map_err(|e| AIError::ConfigError(format!("Failed to delete key: {}", e)))?;
        }
        
        Ok(())
    }

    fn save_metadata(&self) -> Result<()> {
        let metadata_path = crate::config::get_app_data_dir().join("ai_keys_metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.keys_metadata)
            .map_err(|e| AIError::ConfigError(format!("Failed to serialize metadata: {}", e)))?;
        
        std::fs::write(metadata_path, metadata_json)
            .map_err(|e| AIError::ConfigError(format!("Failed to save metadata: {}", e)))?;
        
        Ok(())
    }

    pub fn load_metadata(&mut self) -> Result<()> {
        let metadata_path = crate::config::get_app_data_dir().join("ai_keys_metadata.json");
        
        if metadata_path.exists() {
            let metadata_json = std::fs::read_to_string(metadata_path)
                .map_err(|e| AIError::ConfigError(format!("Failed to read metadata: {}", e)))?;
            
            self.keys_metadata = serde_json::from_str(&metadata_json)
                .map_err(|e| AIError::ConfigError(format!("Failed to parse metadata: {}", e)))?;
        }
        
        Ok(())
    }
}

// Environment variable fallback
pub fn get_api_key_from_env(provider: &str) -> Option<String> {
    let env_var_name = format!("BESTME_{}_API_KEY", provider.to_uppercase());
    std::env::var(env_var_name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_env_fallback() {
        std::env::set_var("BESTME_OPENROUTER_API_KEY", "test_key_123");
        let key = get_api_key_from_env("openrouter");
        assert_eq!(key, Some("test_key_123".to_string()));
        std::env::remove_var("BESTME_OPENROUTER_API_KEY");
    }
}