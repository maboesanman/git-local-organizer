use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub local_src_dir: String,
    hosts: HashMap<String, BaseUrlConfig>,
}

#[derive(Serialize, Deserialize)]
struct BaseUrlConfig {
    #[serde(default)]
    fork_resolution: ForkResolutionMethod,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ForkResolutionMethod {
    Github { api_token: String },
    None,
}

impl Default for ForkResolutionMethod {
    fn default() -> Self {
        Self::None
    }
}

impl Config {
    pub fn get_github_api_token(&self, host: &str) -> Result<String, &str> {
        match self.hosts.get(host) {
            Some(base_url_config) => match &base_url_config.fork_resolution {
                ForkResolutionMethod::Github { api_token } => Ok(api_token.to_string()),
                _ => Err("not github url"),
            },
            None => Err("not github url"),
        }
    }
}
