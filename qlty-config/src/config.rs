mod builder;
mod coverage;
mod download;
mod file_type;
mod ignore;
pub mod issue_transformer;
mod language;
mod overrides;
mod plugin;
mod release;
pub mod smells;
mod source;

pub use self::ignore::{Ignore, ALL_WILDCARD};
pub use self::overrides::Override;
use self::smells::Smells;
pub use builder::Builder;
pub use coverage::Coverage;
pub use download::{Cpu, DownloadDef, DownloadFileType, OperatingSystem, System};
pub use file_type::FileType;
pub use language::Language;
pub use plugin::{
    CheckTrigger, DriverBatchBy, DriverDef, DriverType, EnabledPlugin, ExtraPackage,
    InvocationDirectoryDef, InvocationDirectoryType, IssueMode, OutputDestination, OutputFormat,
    PackageFileCandidate, PluginDef, PluginEnvironment, PluginFetch, Runtime, SuggestionMode,
    TargetDef, TargetType,
};
pub use release::ReleaseDef;
pub use source::SourceDef;

use crate::config::plugin::EnabledRuntimes;
pub use crate::config::plugin::PluginsConfig;
use crate::sources::{Source, SourcesList};
use crate::version::QLTY_VERSION;
use crate::Library;
use anyhow::{anyhow, bail, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct QltyConfig {
    pub config_version: Option<String>,
    pub cli_version: Option<String>,

    pub project_id: Option<String>,

    #[serde(default)]
    pub ignore: Vec<Ignore>,

    #[serde(default)]
    #[serde(rename = "override")] // Since `override` is a reserved keyword
    pub overrides: Vec<Override>,

    #[serde(default)]
    pub file_types: HashMap<String, FileType>,

    #[serde(default)]
    pub test_patterns: Vec<String>,

    #[serde(default)]
    pub coverage: Coverage,

    #[serde(default)]
    pub runtimes: EnabledRuntimes,

    #[serde(default)]
    pub plugins: PluginsConfig,

    #[serde(default)]
    pub sources: HashMap<String, SourceDef>,

    #[serde(default)]
    pub language: HashMap<String, Language>,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(default, skip_serializing)]
    pub ignore_patterns: Vec<String>,

    #[serde(default)]
    pub plugin: Vec<EnabledPlugin>,

    pub smells: Option<Smells>,

    #[serde(default)]
    pub source: Vec<SourceDef>,
}

impl QltyConfig {
    pub fn validate_cli_version(&self) -> Result<()> {
        if self.cli_version.is_none() {
            return Ok(());
        }

        let expected_version = Version::parse(self.cli_version.as_ref().unwrap())?;
        let actual_version = Version::parse(QLTY_VERSION)?;

        if !self.is_version_compatible(&expected_version, &actual_version) {
            if cfg!(debug_assertions) {
                debug!("qlty v{} is incompatible with the cli_version from qlty.toml ({}). Proceeding because qlty is a debug build.", actual_version, expected_version);
            } else {
                bail!("qlty v{} is incompatible with the cli_version from qlty.toml ({}). Please update qlty.", actual_version, expected_version);
            }
        }

        Ok(())
    }

    fn is_version_compatible(&self, expected: &Version, actual: &Version) -> bool {
        // Major version differences are always incompatible
        if expected.major != actual.major {
            return false;
        }

        // Prior to v1, consider minor changes to be incompatible if the expected version is greater than the actual version
        if expected.major == 0 && expected.minor > actual.minor {
            return false;
        }

        true
    }

    pub fn default_source(&self, library: &Library) -> Result<Box<dyn Source>> {
        if let Some(source) = self
            .source
            .iter()
            .find(|source_def| source_def.name.as_deref() == Some("default"))
        {
            return source.source(library);
        }

        let source_def = self.sources.get("default").ok_or_else(|| {
            anyhow!(
                "Could not find `sources.default` key in project config: {:#?}",
                self
            )
        })?;

        source_def.source(library)
    }

    pub fn sources_list(&self, library: &Library) -> Result<SourcesList> {
        let mut sources_list = SourcesList::new();

        for source_def in self.source.iter() {
            sources_list.sources.push(source_def.source(library)?);
        }

        for source_def in self.sources.values() {
            sources_list.sources.push(source_def.source(library)?);
        }

        Ok(sources_list)
    }

    pub fn language_map<T>(&self, f: impl Fn(&Language) -> T) -> HashMap<String, T> {
        self.language
            .iter()
            .map(|(name, settings)| (name.clone(), f(settings)))
            .collect::<HashMap<_, _>>()
    }
}

#[cfg(test)]
mod test {
    use crate::Workspace;

    #[test]
    #[ignore] // always requires network connection
    fn default() {
        let workspace = Workspace::new().unwrap();
        workspace.fetch_sources().unwrap();
        workspace.config().unwrap();
    }
}
