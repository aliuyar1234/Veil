//! Policy executor for applying policies to content.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use veil_crypto::{InMemoryVault, TokenVault};
use veil_detect::Finding;

use crate::apply::apply_policy_to_findings;
use crate::error::PolicyError;
use crate::protect::{protect_value, ProtectedValue, ProtectionContext};
use crate::rules::ProtectionAction;
use crate::schema::Policy;

/// Executor for applying policies to content.
pub struct PolicyExecutor {
    /// Resolved encryption key.
    encryption_key: Option<Vec<u8>>,
    /// Token vault for tokenization.
    vault: Arc<dyn TokenVault>,
    /// Cache for consistent pseudonymization (original -> pseudonym).
    pseudonym_cache: HashMap<String, String>,
    /// Locale for pseudonymization.
    locale: String,
}

impl PolicyExecutor {
    /// Create a new executor with default settings.
    pub fn new() -> Self {
        Self {
            encryption_key: None,
            vault: Arc::new(InMemoryVault::new()),
            pseudonym_cache: HashMap::new(),
            locale: "en_US".to_string(),
        }
    }

    /// Create executor from a policy.
    ///
    /// Resolves key references and configures locale.
    pub fn from_policy(policy: &Policy) -> Result<Self, PolicyError> {
        let mut executor = Self::new();

        // Set locale from policy
        if let Some(ref locale) = policy.locale {
            executor.locale = locale.to_string();
        }

        // Try to resolve encryption key from first protection rule with key_ref
        for rule in &policy.protection {
            if let Some(ref key_ref) = rule.key_ref {
                let key = key_ref.resolve().map_err(PolicyError::KeyRefError)?;
                executor.encryption_key = Some(key);
                break;
            }
        }

        Ok(executor)
    }

    /// Set encryption key directly.
    pub fn with_key(mut self, key: Vec<u8>) -> Self {
        self.encryption_key = Some(key);
        self
    }

    /// Set token vault.
    pub fn with_vault(mut self, vault: Arc<dyn TokenVault>) -> Self {
        self.vault = vault;
        self
    }

    /// Clear the pseudonym cache.
    ///
    /// Call this when starting a new document or session if you don't want
    /// consistent pseudonymization across documents.
    pub fn clear_cache(&mut self) {
        self.pseudonym_cache.clear();
    }

    /// Process content with a policy.
    ///
    /// This is the main entry point for policy-based protection.
    pub fn process(
        &mut self,
        content: &str,
        findings: Vec<Finding>,
        policy: &Policy,
    ) -> Result<ProcessResult, PolicyError> {
        let start = Instant::now();

        // Filter findings by detection rules
        let filtered_findings = apply_policy_to_findings(policy, findings.clone());
        let findings_filtered = findings.len() - filtered_findings.len();

        // Build result content with replacements
        let mut result_content = content.to_string();
        let mut actions = Vec::new();

        // Sort findings by start position (descending) to replace from end
        let mut sorted_findings: Vec<_> = filtered_findings.iter().enumerate().collect();
        sorted_findings.sort_by(|a, b| b.1.start.cmp(&a.1.start));

        for (original_index, finding) in sorted_findings {
            // Find matching protection rule
            let rule = policy
                .protection
                .iter()
                .find(|r| r.types.is_empty() || r.types.contains(&finding.category));

            if let Some(rule) = rule {
                // Protect the value
                let protected = self.protect_finding(finding, rule)?;

                // Apply to content
                let original = &content[finding.start..finding.end];
                result_content = result_content.replacen(original, &protected.value, 1);

                actions.push(AppliedAction {
                    finding_index: original_index,
                    action: protected.action,
                    original: original.to_string(),
                    protected: protected.value,
                    cached: protected.cached,
                });
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let findings_protected = actions.len();

        Ok(ProcessResult {
            content: result_content,
            findings: filtered_findings,
            actions,
            stats: ProcessStats {
                findings_detected: findings.len(),
                findings_filtered,
                findings_protected,
                duration_ms,
            },
        })
    }

    /// Protect a single finding according to a protection rule.
    pub fn protect_finding(
        &mut self,
        finding: &Finding,
        rule: &crate::rules::ProtectionRule,
    ) -> Result<ProtectedValue, PolicyError> {
        let value = finding.matched_text.as_str();

        // Check cache for consistent pseudonymization
        if rule.action == ProtectionAction::Pseudonymize && rule.consistent {
            if let Some(cached) = self.pseudonym_cache.get(value) {
                return Ok(ProtectedValue::cached(
                    cached.clone(),
                    ProtectionAction::Pseudonymize,
                ));
            }
        }

        // Build protection context
        let mut ctx = ProtectionContext::default()
            .with_optional_key(self.encryption_key.clone())
            .with_vault(self.vault.clone())
            .with_locale(&self.locale);

        if rule.consistent {
            // Derive seed from value for consistency
            ctx = ctx.consistent(Some(hash_to_seed(value)));
        }

        // Protect
        let protected = protect_value(
            value,
            &rule.action,
            rule.style.as_deref(),
            rule.key_ref.as_ref(),
            &ctx,
        )?;

        // Cache for consistent pseudonymization
        if rule.action == ProtectionAction::Pseudonymize && rule.consistent {
            self.pseudonym_cache
                .insert(value.to_string(), protected.value.clone());
        }

        Ok(protected)
    }
}

impl Default for PolicyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of processing content with a policy.
#[derive(Debug)]
pub struct ProcessResult {
    /// Protected content.
    pub content: String,
    /// Findings that were processed (after filtering).
    pub findings: Vec<Finding>,
    /// Protection actions applied.
    pub actions: Vec<AppliedAction>,
    /// Processing statistics.
    pub stats: ProcessStats,
}

/// A protection action that was applied.
#[derive(Debug)]
pub struct AppliedAction {
    /// Index of the finding that was protected.
    pub finding_index: usize,
    /// Action that was applied.
    pub action: ProtectionAction,
    /// Original value.
    pub original: String,
    /// Protected value.
    pub protected: String,
    /// Whether this was from cache (consistent mode).
    pub cached: bool,
}

/// Processing statistics.
#[derive(Debug, Default)]
pub struct ProcessStats {
    /// Total findings detected.
    pub findings_detected: usize,
    /// Findings filtered out by policy.
    pub findings_filtered: usize,
    /// Findings protected.
    pub findings_protected: usize,
    /// Processing time in milliseconds.
    pub duration_ms: u64,
}

/// Hash a string to a u64 seed for consistent pseudonymization.
fn hash_to_seed(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_detect::{PiiCategory, ValidationStatus};

    fn make_finding(text: &str, category: PiiCategory, start: usize, end: usize) -> Finding {
        Finding::new(text, category, start, end, 0.9, ValidationStatus::Valid, 0)
    }

    #[test]
    fn test_executor_new() {
        let executor = PolicyExecutor::new();
        assert!(executor.encryption_key.is_none());
        assert!(executor.pseudonym_cache.is_empty());
    }

    #[test]
    fn test_executor_from_policy() {
        let policy = Policy::default();
        let executor = PolicyExecutor::from_policy(&policy).unwrap();
        assert!(executor.encryption_key.is_none());
    }

    #[test]
    fn test_executor_clear_cache() {
        let mut executor = PolicyExecutor::new();
        executor
            .pseudonym_cache
            .insert("key".to_string(), "value".to_string());
        executor.clear_cache();
        assert!(executor.pseudonym_cache.is_empty());
    }

    #[test]
    fn test_executor_process_redact() {
        let mut executor = PolicyExecutor::new();
        let content = "Contact: test@example.com";
        let findings = vec![make_finding("test@example.com", PiiCategory::Email, 9, 25)];

        let policy = Policy {
            protection: vec![crate::rules::ProtectionRule {
                types: vec![PiiCategory::Email],
                action: ProtectionAction::Redact,
                style: Some("label".to_string()),
                consistent: false,
                key_ref: None,
            }],
            ..Default::default()
        };

        let result = executor.process(content, findings, &policy).unwrap();

        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("test@example.com"));
        assert_eq!(result.stats.findings_protected, 1);
    }

    #[test]
    fn test_executor_consistent_pseudonymization() {
        let mut executor = PolicyExecutor::new();

        let rule = crate::rules::ProtectionRule {
            types: vec![],
            action: ProtectionAction::Pseudonymize,
            style: None,
            consistent: true,
            key_ref: None,
        };

        let finding1 = make_finding("john@test.com", PiiCategory::Email, 0, 13);
        let finding2 = make_finding("john@test.com", PiiCategory::Email, 20, 33);

        let protected1 = executor.protect_finding(&finding1, &rule).unwrap();
        let protected2 = executor.protect_finding(&finding2, &rule).unwrap();

        // Same name should get same pseudonym
        assert_eq!(protected1.value, protected2.value);
        // Second one should be cached
        assert!(protected2.cached);
    }

    #[test]
    fn test_process_stats() {
        let mut executor = PolicyExecutor::new();
        let content = "Email: test@example.com";
        let findings = vec![make_finding("test@example.com", PiiCategory::Email, 7, 23)];

        let policy = Policy::default();
        let result = executor.process(content, findings, &policy).unwrap();

        assert_eq!(result.stats.findings_detected, 1);
        assert!(result.stats.duration_ms < 1000); // Should be fast
    }
}
