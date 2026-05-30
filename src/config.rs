use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupConfig {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub check: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub install: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub update: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub uninstall: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub reboot_on: HashMap<String, bool>,
}

impl AppConfig {
    pub fn effective_id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub apps: Vec<AppConfig>,
    #[serde(default)]
    pub groups: Vec<GroupConfig>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    apps: Vec<serde_yaml::Value>,
    #[serde(default)]
    groups: Vec<GroupConfig>,
}

pub fn load_config(path: &Path) -> Result<PackageConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let raw: RawConfig = match ext {
        "json" => serde_json::from_str(&content).context("Failed to parse JSON config")?,
        _ => serde_yaml::from_str(&content).context("Failed to parse YAML config")?,
    };

    let mut apps = Vec::new();
    for (i, val) in raw.apps.into_iter().enumerate() {
        let app = parse_app(val, i).with_context(|| format!("Failed to parse app #{}", i))?;
        apps.push(app);
    }

    Ok(PackageConfig {
        version: raw.version,
        apps,
        groups: raw.groups,
    })
}

fn parse_app(val: serde_yaml::Value, index: usize) -> Result<AppConfig> {
    let map = match val {
        serde_yaml::Value::Mapping(m) => m,
        _ => bail!("App must be a mapping"),
    };

    let get_str = |key: &str| -> String {
        map.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let name = get_str("name");
    if name.is_empty() {
        bail!("App #{} is missing required 'name' field", index);
    }

    let raw_id = get_str("id");
    let id = if raw_id.is_empty() {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
            .collect()
    } else {
        raw_id
    };

    let description = get_str("description");
    let group = get_str("group");

    let depends_on = map
        .get("depends_on")
        .map(|v| parse_string_list(v))
        .unwrap_or_default();

    let reboot_on = map
        .get("reboot_on")
        .and_then(|v| v.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| {
                    let key = k.as_str()?.to_string();
                    let val = v.as_bool()?;
                    Some((key, val))
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse actions - support both top-level and nested under "actions" key
    let check = parse_action_map(map.get("check"));
    let install = parse_action_map(
        map.get("install")
            .or_else(|| map.get("actions").and_then(|a| a.get("install"))),
    );
    let update = parse_action_map(
        map.get("update")
            .or_else(|| map.get("actions").and_then(|a| a.get("update"))),
    );
    let uninstall = parse_action_map(
        map.get("uninstall")
            .or_else(|| map.get("actions").and_then(|a| a.get("uninstall"))),
    );

    Ok(AppConfig {
        id,
        name,
        description,
        group,
        check,
        install,
        update,
        uninstall,
        depends_on,
        reboot_on,
    })
}

fn parse_action_map(val: Option<&serde_yaml::Value>) -> HashMap<String, Vec<String>> {
    let Some(val) = val else {
        return HashMap::new();
    };

    match val {
        serde_yaml::Value::String(s) => {
            let mut m = HashMap::new();
            m.insert("default".to_string(), vec![s.clone()]);
            m
        }
        serde_yaml::Value::Sequence(seq) => {
            let cmds = seq.iter().filter_map(|v| v.as_str()).map(String::from).collect();
            let mut m = HashMap::new();
            m.insert("default".to_string(), cmds);
            m
        }
        serde_yaml::Value::Mapping(mapping) => {
            let mut result = HashMap::new();
            for (k, v) in mapping {
                let Some(key) = k.as_str() else { continue };
                // Handle legacy "installed" key
                let key = if key == "installed" { "default" } else { key };
                let cmds = parse_string_list(v);
                if !cmds.is_empty() {
                    result.insert(key.to_string(), cmds);
                }
            }
            result
        }
        _ => HashMap::new(),
    }
}

fn parse_string_list(val: &serde_yaml::Value) -> Vec<String> {
    match val {
        serde_yaml::Value::String(s) => vec![s.clone()],
        serde_yaml::Value::Sequence(seq) => {
            seq.iter().filter_map(|v| v.as_str()).map(String::from).collect()
        }
        _ => vec![],
    }
}

pub fn find_default_config() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let base = config_dir.join("settupper");

    let names = ["packages.yaml", "packages.yml", "packages.json"];
    for name in &names {
        let path = base.join(name);
        if path.exists() {
            return Some(path);
        }
    }

    for name in &names {
        let path = PathBuf::from(name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}
