use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequiredStatus {
    RequiredByAll,
    Optional,
}

/// Whether a mod can be resolved by clients or not
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum ResolvableStatus {
    Unresolvable(String),
    Resolvable,
}

/// Points to a mod, optionally a specific version
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ModType {
    ModPlugin,
    Pak,
}

/// Returned from ModStore
#[derive(Debug, Clone)]
pub struct ModInfo {
    pub provider: &'static str,
    pub name: String,
    pub spec: ModSpecification,          // unpinned version
    pub versions: Vec<ModSpecification>, // pinned versions TODO make this a different type
    pub resolution: ModResolution,
    pub suggested_require: bool,
    pub suggested_dependencies: Vec<ModSpecification>, // ModResponse
    pub mod_type: ModType,
}

/// Returned from ModProvider
#[derive(Debug, Clone)]
pub enum ModResponse {
    Redirect(ModSpecification),
    Resolve(ModInfo),
}

/// Points to a mod, optionally a specific version
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ModSpecification {
    pub url: String,
}

impl ModSpecification {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub fn satisfies_dependency(&self, other: &ModSpecification) -> bool {
        // TODO this hack works surprisingly well but is still a complete hack and should be replaced
        self.url.starts_with(&other.url) || other.url.starts_with(&self.url)
    }
}

/// Points to a specific version of a specific mod
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ModResolution {
    pub url: ModIdentifier,
    pub status: ResolvableStatus,
}

impl ModResolution {
    pub fn resolvable(url: ModIdentifier) -> Self {
        Self {
            url,
            status: ResolvableStatus::Resolvable,
        }
    }
    pub fn unresolvable(url: ModIdentifier, name: String) -> Self {
        Self {
            url,
            status: ResolvableStatus::Unresolvable(name),
        }
    }
    /// Used to get the URL if resolvable or just return the mod name if not
    pub fn get_resolvable_url_or_name(&self) -> &str {
        match &self.status {
            ResolvableStatus::Resolvable => &self.url.0,
            ResolvableStatus::Unresolvable(name) => name,
        }
    }
}

/// Mod identifier used for tracking gameplay affecting status.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ModIdentifier(pub String);

impl ModIdentifier {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}
impl From<String> for ModIdentifier {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for ModIdentifier {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

/// Stripped down mod info stored in the mod pak to be used in game
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub version: SemverVersion,
    pub mods: Vec<MetaMod>,
    pub config: MetaConfig,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaConfig {
    pub disable_fix_exploding_gas: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SemverVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
impl Display for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaMod {
    pub name: String,
    pub version: String,
    pub url: String,
    pub author: String,
    pub required: bool,
}
