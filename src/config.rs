use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{env, fmt, fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobConfig {
    pub active_profile_id: String,
    pub profiles: Vec<ProviderProfile>,
    #[serde(default)]
    pub tool_approval: ApprovalPolicy,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    /// Run allowlisted tools automatically.
    Auto,
    /// Ask before running tools in interactive chat; deny tools in non-interactive ask.
    OnRequest,
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
        }
    }
}

impl Default for ApprovalPolicy {
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
}
