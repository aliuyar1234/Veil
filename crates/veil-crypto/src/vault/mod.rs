//! Token vault storage abstraction.

mod file;
mod memory;

pub use file::FileVault;
pub use memory::InMemoryVault;

use crate::error::VaultError;

/// Trait for token vault storage backends.
///
/// Implementations must be thread-safe (Send + Sync).
pub trait TokenVault: Send + Sync {
    /// Store a token-to-original mapping.
    ///
    /// # Arguments
    /// * `token` - The generated token
    /// * `original` - The original value
    ///
    /// # Returns
    /// * `Ok(())` - If stored successfully
    /// * `Err(VaultError::Duplicate)` - If token already exists
    fn store(&self, token: &str, original: &str) -> Result<(), VaultError>;

    /// Look up the original value for a token.
    ///
    /// # Arguments
    /// * `token` - The token to look up
    ///
    /// # Returns
    /// * `Ok(Some(original))` - If found
    /// * `Ok(None)` - If token doesn't exist
    fn lookup(&self, token: &str) -> Result<Option<String>, VaultError>;

    /// Delete a token mapping.
    ///
    /// # Arguments
    /// * `token` - The token to delete
    ///
    /// # Returns
    /// * `Ok(true)` - If deleted
    /// * `Ok(false)` - If token didn't exist
    fn delete(&self, token: &str) -> Result<bool, VaultError>;

    /// Find an existing token for an original value.
    ///
    /// Used for consistent tokenization (same input → same token).
    ///
    /// # Arguments
    /// * `original` - The original value to find a token for
    ///
    /// # Returns
    /// * `Ok(Some(token))` - If original was already tokenized
    /// * `Ok(None)` - If not found
    fn find_by_original(&self, original: &str) -> Result<Option<String>, VaultError>;
}
