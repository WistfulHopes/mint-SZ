pub mod error;
pub mod mod_info;
pub mod update;

use std::{
    io::BufWriter,
    path::{Path, PathBuf},
};

use anyhow::{Result};
use fs_err as fs;
use tracing::*;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug)]
pub struct DBSZInstallation {
    pub root: PathBuf,
}

impl DBSZInstallation {
    /// Returns first DBSZ installation found.
    pub fn find() -> Option<Self> {
        steamlocate::SteamDir::locate()
            .ok()
            .and_then(|steamdir| {
                steamdir
                    .find_app(1790600)
                    .ok()
                    .flatten()
                    .map(|(app, library)| {
                        library
                            .resolve_app_dir(&app)
                            .join("SparkingZERO")
                    })
            })
            .and_then(|path| Self::from_game_path(path).ok())
    }
    pub fn from_game_path<P: AsRef<Path>>(game: P) -> Result<Self> {
        let root = game
            .as_ref()
            .parent()
            .unwrap()
            .to_path_buf();
        Ok(Self {
            root,
        })
    }
    pub fn binaries_directory(&self) -> PathBuf {
        self.root
            .join("Binaries")
            .join("Win64")
    }
    pub fn paks_path(&self) -> PathBuf {
        self.root.join("Content").join("Paks")
    }
    pub fn mods_path(&self) -> PathBuf {
        self.root.join("Mods")
    }
}

pub fn setup_logging<P: AsRef<Path>>(
    log_path: P,
    target: &str,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing::metadata::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{
        field::RecordFields,
        filter,
        fmt::{
            self,
            format::{Pretty, Writer},
            FormatFields,
        },
        EnvFilter,
    };

    /// Workaround for <https://github.com/tokio-rs/tracing/issues/1817>.
    struct NewType(Pretty);

    impl<'writer> FormatFields<'writer> for NewType {
        fn format_fields<R: RecordFields>(
            &self,
            writer: Writer<'writer>,
            fields: R,
        ) -> core::fmt::Result {
            self.0.format_fields(writer, fields)
        }
    }

    let f = fs::File::create(log_path.as_ref())?;
    let writer = BufWriter::new(f);
    let (log_file_appender, guard) = tracing_appender::non_blocking(writer);
    let debug_file_log = fmt::layer()
        .with_writer(log_file_appender)
        .fmt_fields(NewType(Pretty::default()))
        .with_ansi(false)
        .with_filter(filter::Targets::new().with_target(target, Level::DEBUG));
    let stderr_log = fmt::layer()
        .with_writer(std::io::stderr)
        .event_format(tracing_subscriber::fmt::format().without_time())
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        );
    let subscriber = tracing_subscriber::registry()
        .with(stderr_log)
        .with(debug_file_log);

    tracing::subscriber::set_global_default(subscriber)?;

    debug!("tracing subscriber setup");
    info!("writing logs to {:?}", log_path.as_ref().display());

    Ok(guard)
}
