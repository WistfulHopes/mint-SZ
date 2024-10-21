use std::collections::{HashSet};
use std::io;
use std::io::{ErrorKind};
use std::path::{Path, PathBuf};

use fs_err as fs;

use snafu::{prelude::*, Whatever};
use tracing::info;

use crate::mod_lints::LintError;
use crate::providers::{ModInfo, ProviderError};
use mint_lib::mod_info::{ModType};
use mint_lib::DBSZInstallation;

use crate::integrate::IntegrationError::IoError;

#[tracing::instrument(level = "debug", skip(path_pak))]
pub fn uninstall<P: AsRef<Path>>(path_pak: P, modio_mods: HashSet<u32>) -> Result<(), Whatever> {
    let installation = DBSZInstallation::from_game_path(path_pak)
        .whatever_context("failed to get DBSZ installation")?;
    let path_mods = installation.mods_path();
    match fs::remove_dir_all(&path_mods) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
    .with_whatever_context(|_| format!("failed to remove {}", path_mods.display()))?;
    let path_mods_paks = installation.paks_path().join("~mods");
    match fs::remove_dir_all(&path_mods_paks) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
    .with_whatever_context(|_| format!("failed to remove {}", path_mods_paks.display()))?;
    /* #[cfg(feature = "hook")]
    {
        let path_hook_dll = installation
            .binaries_directory()
            .join(installation.installation_type.hook_dll_name());
        match fs::remove_file(&path_hook_dll) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
        .with_whatever_context(|_| format!("failed to remove {}", path_hook_dll.display()))?;
    } */
    Ok(())
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum IntegrationError {
    #[snafu(display("unable to determine DBSZ installation at provided path {}", path.display()))]
    DrgInstallationNotFound { path: PathBuf },
    #[snafu(transparent)]
    IoError { source: std::io::Error },
    #[snafu(transparent)]
    RepakError { source: repak::Error },
    #[snafu(transparent)]
    UnrealAssetError { source: unreal_asset::Error },
    #[snafu(display("mod {:?}: I/O error encountered during its processing", mod_info.name))]
    CtxtIoError {
        source: std::io::Error,
        mod_info: ModInfo,
    },
    #[snafu(display("mod {:?}: repak error encountered during its processing", mod_info.name))]
    CtxtRepakError {
        source: repak::Error,
        mod_info: ModInfo,
    },
    #[snafu(display(
        "mod {:?}: modfile {} contains unexpected prefix",
        mod_info.name,
        modfile_path
    ))]
    ModfileInvalidPrefix {
        mod_info: ModInfo,
        modfile_path: String,
    },
    #[snafu(display(
        "mod {:?}: failed to integrate: {source}",
        mod_info.name,
    ))]
    CtxtGenericError {
        source: Box<dyn std::error::Error + Send + Sync>,
        mod_info: ModInfo,
    },
    #[snafu(transparent)]
    ProviderError { source: ProviderError },
    #[snafu(display("integration error: {msg}"))]
    GenericError { msg: String },
    #[snafu(transparent)]
    JoinError { source: tokio::task::JoinError },
    #[snafu(transparent)]
    LintError { source: LintError },
    #[snafu(display("self update failed: {source:?}"))]
    SelfUpdateFailed {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn integrate<P: AsRef<Path>>(
    path_project: P,
    mods: Vec<(ModInfo, PathBuf)>,
) -> Result<(), IntegrationError> {
    let Ok(installation) = DBSZInstallation::from_game_path(&path_project) else {
        return Err(IntegrationError::DrgInstallationNotFound {
            path: path_project.as_ref().to_path_buf(),
        });
    };

    /*
    #[cfg(feature = "hook")]
    {
        let path_hook_dll = installation
            .binaries_directory()
            .join(installation.installation_type.hook_dll_name());
        let hook_dll = include_bytes!(env!("CARGO_CDYLIB_FILE_HOOK_hook"));
        if path_hook_dll
            .metadata()
            .map(|m| m.len() != hook_dll.len() as u64)
            .unwrap_or(true)
        {
            fs::write(&path_hook_dll, hook_dll)?;
        }
    }*/

    for (mod_info, path) in &mods {
        match mod_info.mod_type
        {
            ModType::ModPlugin => {
                let result = copy_dir_all(path, &installation.mods_path().join(&mod_info.name));
                match result {
                    Err(e) => return Err(IoError {source: e }),
                    _ => {}
                }
            }
            ModType::Pak => {
                let result = copy_dir_all(path, &installation.paks_path().join("~mods").join(&mod_info.name));
                match result {
                    Err(e) => return Err(IoError {source: e }),
                    _ => {}
                }
            }
        }
    }

    info!(
        "{} mods installed",
        mods.len(),
    );

    Ok(())
}