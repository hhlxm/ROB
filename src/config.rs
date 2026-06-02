use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{env, fmt, fs, path::PathBuf};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobConfig {
    pub active_profile_id: String,
    pub profiles: Vec<ProviderProfile>,
    #[serde(default)]
    pub tool_approval: ApprovalPolicy,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub reasoning: ReasoningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key_env: Option<String>,
    pub api_key: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub token_threshold: usize,
    pub recent_messages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub effort: ReasoningEffort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    /// Run allowlisted tools automatically.
    Auto,
    /// Ask before running tools in interactive chat; deny tools in non-interactive ask.
    OnRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ReasoningEffort {
    /// Do not send provider-specific reasoning controls.
    Auto,
    /// Explicitly disable thinking when the provider supports it.
    No,
    /// Request low reasoning effort.
    Low,
    /// Request medium reasoning effort.
    Medium,
    /// Request high reasoning effort.
    High,
}

impl RobConfig {
    pub fn load_or_default() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir {}", parent.display()))?;
        }
        let raw = toml::to_string_pretty(self)?;
        fs::write(&path, raw).with_context(|| format!("failed to write config {}", path.display()))
    }

    pub fn active_profile(&self) -> Result<&ProviderProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == self.active_profile_id)
            .or_else(|| self.profiles.first())
            .ok_or_else(|| {
                anyhow!(
                    "no provider configured; run `rob config set --base-url ... --model ... --api-key-env ...`"
                )
            })
    }

    pub fn set_active_profile(&mut self, profile: ProviderProfile) {
        self.active_profile_id = profile.id.clone();
        self.profiles
            .retain(|candidate| candidate.name != profile.name);
        self.profiles.push(profile);
    }

    pub fn add_active_profile(&mut self, profile: ProviderProfile) -> Result<()> {
        if self
            .profiles
            .iter()
            .any(|candidate| candidate.name == profile.name)
        {
            return Err(anyhow!(
                "provider profile `{}` already exists; choose a different --name or pass --replace",
                profile.name
            ));
        }

        self.active_profile_id = profile.id.clone();
        self.profiles.push(profile);
        Ok(())
    }

    pub fn replace_active_profile(&mut self, mut profile: ProviderProfile) {
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|candidate| candidate.name == profile.name)
        {
            profile.id = existing.id.clone();
            self.active_profile_id = profile.id.clone();
            *existing = profile;
            return;
        }

        self.active_profile_id = profile.id.clone();
        self.profiles.push(profile);
    }

    pub fn next_profile_name(&self, base_url: &str, model: &str) -> String {
        let base = if self.profiles.is_empty() {
            "default".to_string()
        } else {
            suggested_profile_name(base_url, model)
        };
        let mut candidate = base.clone();
        let mut suffix = 2;

        while self
            .profiles
            .iter()
            .any(|profile| profile.name == candidate)
        {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }

        candidate
    }

    pub fn set_active_profile_by_ref(&mut self, profile_ref: &str) -> Result<()> {
        let profile = self
            .profiles
            .iter()
            .find(|candidate| candidate.name == profile_ref || candidate.id == profile_ref)
            .ok_or_else(|| anyhow!("provider profile `{profile_ref}` was not found"))?;
        self.active_profile_id = profile.id.clone();
        Ok(())
    }
}

impl Default for RobConfig {
    fn default() -> Self {
        Self {
            active_profile_id: String::new(),
            profiles: Vec::new(),
            tool_approval: ApprovalPolicy::Auto,
            context: ContextConfig::default(),
            reasoning: ReasoningConfig::default(),
        }
    }
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            token_threshold: 32_000,
            recent_messages: 12,
        }
    }
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            effort: ReasoningEffort::Auto,
        }
    }
}

impl Default for ReasoningEffort {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for ApprovalPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(formatter, "auto"),
            Self::OnRequest => write!(formatter, "on-request"),
        }
    }
}

impl fmt::Display for ReasoningEffort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(formatter, "auto"),
            Self::No => write!(formatter, "no"),
            Self::Low => write!(formatter, "low"),
            Self::Medium => write!(formatter, "medium"),
            Self::High => write!(formatter, "high"),
        }
    }
}

impl ProviderProfile {
    pub fn new(
        name: String,
        base_url: String,
        model: String,
        api_key_env: Option<String>,
        api_key: Option<String>,
        protocol: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            base_url: normalize_base_url(&base_url),
            model,
            api_key_env,
            api_key,
            protocol,
        }
    }

    pub fn resolve_api_key(&self) -> Result<String> {
        if let Some(env_name) = &self.api_key_env {
            let value = env::var(env_name)
                .with_context(|| format!("environment variable {env_name} is not set"))?;
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }

        if let Some(api_key) = &self.api_key {
            if !api_key.trim().is_empty() {
                return Ok(api_key.clone());
            }
        }

        Err(anyhow!(
            "provider `{}` has no API key; set api_key_env or api_key",
            self.name
        ))
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("ROB_CONFIG") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    let config_dir = dirs::config_dir().ok_or_else(|| anyhow!("failed to locate config dir"))?;
    Ok(config_dir.join("rob").join("config.toml"))
}

fn normalize_base_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn suggested_profile_name(base_url: &str, model: &str) -> String {
    let raw_name = Url::parse(&normalize_base_url(base_url))
        .ok()
        .and_then(|url| url.host_str().map(host_profile_name))
        .unwrap_or_else(|| model.to_string());

    sanitize_profile_name(&raw_name)
}

fn host_profile_name(host: &str) -> String {
    if host
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        return host.to_string();
    }

    let labels: Vec<&str> = host.split('.').filter(|label| !label.is_empty()).collect();
    if labels.len() >= 3 && matches!(labels[0], "api" | "www") {
        return labels[1].to_string();
    }
    if labels.len() >= 2 {
        return labels[labels.len() - 2].to_string();
    }

    host.to_string()
}

fn sanitize_profile_name(raw: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;

    for character in raw.trim().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !output.is_empty() && !last_was_separator {
            output.push('-');
            last_was_separator = true;
        }
    }

    let output = output.trim_matches('-');
    if output.is_empty() {
        "provider".to_string()
    } else {
        output.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_profile_normalizes_base_url() {
        let profile = ProviderProfile::new(
            "test".to_string(),
            " https://example.com/v1/// ".to_string(),
            "model".to_string(),
            Some("ROB_TEST_KEY".to_string()),
            None,
            "openai-compatible".to_string(),
        );

        assert_eq!(profile.base_url, "https://example.com/v1");
    }

    #[test]
    fn config_switches_active_profile_by_name() {
        let profile = ProviderProfile::new(
            "primary".to_string(),
            "https://example.com/v1".to_string(),
            "model".to_string(),
            None,
            Some("key".to_string()),
            "openai-compatible".to_string(),
        );
        let expected_id = profile.id.clone();
        let mut config = RobConfig::default();
        config.set_active_profile(profile);
        config.active_profile_id.clear();

        config.set_active_profile_by_ref("primary").unwrap();

        assert_eq!(config.active_profile_id, expected_id);
    }

    #[test]
    fn config_add_active_profile_rejects_duplicate_names() {
        let mut config = RobConfig::default();
        config
            .add_active_profile(ProviderProfile::new(
                "primary".to_string(),
                "https://example.com/v1".to_string(),
                "model-a".to_string(),
                Some("KEY_A".to_string()),
                None,
                "openai-compatible".to_string(),
            ))
            .unwrap();

        let error = config
            .add_active_profile(ProviderProfile::new(
                "primary".to_string(),
                "https://api.example.com/v1".to_string(),
                "model-b".to_string(),
                Some("KEY_B".to_string()),
                None,
                "openai-compatible".to_string(),
            ))
            .unwrap_err();

        assert!(error.to_string().contains("already exists"));
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].api_key_env.as_deref(), Some("KEY_A"));
    }

    #[test]
    fn config_replace_active_profile_preserves_profile_id() {
        let original = ProviderProfile::new(
            "primary".to_string(),
            "https://example.com/v1".to_string(),
            "model-a".to_string(),
            Some("KEY_A".to_string()),
            None,
            "openai-compatible".to_string(),
        );
        let expected_id = original.id.clone();
        let mut config = RobConfig::default();
        config.add_active_profile(original).unwrap();

        config.replace_active_profile(ProviderProfile::new(
            "primary".to_string(),
            "https://api.example.com/v1".to_string(),
            "model-b".to_string(),
            Some("KEY_B".to_string()),
            None,
            "openai-compatible".to_string(),
        ));

        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].id, expected_id);
        assert_eq!(config.profiles[0].model, "model-b");
        assert_eq!(config.profiles[0].api_key_env.as_deref(), Some("KEY_B"));
        assert_eq!(config.active_profile_id, expected_id);
    }

    #[test]
    fn config_generates_unique_names_for_implicit_profiles() {
        let mut config = RobConfig::default();

        assert_eq!(
            config.next_profile_name("https://api.openai.com/v1", "gpt"),
            "default"
        );

        config
            .add_active_profile(ProviderProfile::new(
                "default".to_string(),
                "https://api.openai.com/v1".to_string(),
                "gpt".to_string(),
                Some("OPENAI_API_KEY".to_string()),
                None,
                "openai-compatible".to_string(),
            ))
            .unwrap();

        assert_eq!(
            config.next_profile_name("https://api.deepseek.com/v1", "deepseek-chat"),
            "deepseek"
        );

        config
            .add_active_profile(ProviderProfile::new(
                "deepseek".to_string(),
                "https://api.deepseek.com/v1".to_string(),
                "deepseek-chat".to_string(),
                Some("DEEPSEEK_API_KEY".to_string()),
                None,
                "openai-compatible".to_string(),
            ))
            .unwrap();

        assert_eq!(
            config.next_profile_name("https://api.deepseek.com/v1", "deepseek-reasoner"),
            "deepseek-2"
        );
    }
}
