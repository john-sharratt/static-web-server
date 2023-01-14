use std::path::PathBuf;

use globset::{Glob, GlobMatcher};
use headers::HeaderMap;
use structopt::StructOpt;

use crate::{Context, Result};

mod cli;
pub mod file;

#[cfg(windows)]
pub use cli::Commands;

use cli::General;

/// The `headers` file options.
pub struct Headers {
    /// Source pattern glob matcher
    pub source: GlobMatcher,
    /// Map of custom HTTP headers
    pub headers: HeaderMap,
}

/// The `advanced` file options.
pub struct Advanced {
    pub headers: Option<Vec<Headers>>,
}

/// The full server CLI and File options.
pub struct Settings {
    /// General server options
    pub general: General,
    /// Advanced server options
    pub advanced: Option<Advanced>,
}

impl Settings {
    /// Handles CLI and config file options and converging them into one.
    pub fn get() -> Result<Settings> {
        let mut opts = General::from_args();

        // If no config file was supplied then attempt to use the default path
        if opts.config_file.is_none() {
            let default_config_file: PathBuf = "./cfg/config.toml".into();
            if default_config_file.exists() {
                opts.config_file = Some(default_config_file);
            }
        }

        // Define the general CLI/file options
        let mut host = opts.host;
        let mut port = opts.port;
        let mut root = opts.root;
        let mut log_level = opts.log_level;
        let mut config_file = opts.config_file.clone();
        let mut cache_control_headers = opts.cache_control_headers;
        let mut compression = opts.compression;
        let mut page404 = opts.page404;
        let mut page50x = opts.page50x;
        #[cfg(feature = "http2")]
        let mut http2 = opts.http2;
        #[cfg(feature = "http2")]
        let mut http2_tls_cert = opts.http2_tls_cert;
        #[cfg(feature = "http2")]
        let mut http2_tls_key = opts.http2_tls_key;
        let mut security_headers = opts.security_headers;
        let mut cors_allow_origins = opts.cors_allow_origins;
        let mut cors_allow_headers = opts.cors_allow_headers;
        let mut directory_listing = opts.directory_listing;
        let mut directory_listing_order = opts.directory_listing_order;
        let mut basic_auth = opts.basic_auth;
        let mut fd = opts.fd;
        let mut threads_multiplier = opts.threads_multiplier;
        let mut grace_period = opts.grace_period;
        let mut page_fallback = opts.page_fallback;
        let mut log_remote_address = opts.log_remote_address;

        // Define the advanced file options
        let mut settings_advanced: Option<Advanced> = None;

        // Handle "config file options" and set them when available
        // NOTE: All config file based options shouldn't be mandatory, therefore `Some()` wrapped
        if let Some(ref p) = opts.config_file {
            if p.is_file() {
                let path_resolved = p
                    .canonicalize()
                    .unwrap_or(p.clone());

                let settings = file::Settings::read(&path_resolved)
                    .with_context(|| {
                        "can not read toml config file because has invalid or unsupported format/options"
                    })?;

                config_file = Some(path_resolved);

                // Assign the corresponding file option values
                if let Some(general) = settings.general {
                    if let Some(v) = general.host {
                        host = v
                    }
                    if let Some(v) = general.port {
                        port = v
                    }
                    if let Some(v) = general.root {
                        root = v
                    }
                    if let Some(ref v) = general.log_level {
                        log_level = v.name().to_lowercase();
                    }
                    if let Some(v) = general.cache_control_headers {
                        cache_control_headers = v
                    }
                    if let Some(v) = general.compression {
                        compression = v
                    }
                    if let Some(v) = general.page404 {
                        page404 = v
                    }
                    if let Some(v) = general.page50x {
                        page50x = v
                    }
                    #[cfg(feature = "http2")]
                    if let Some(v) = general.http2 {
                        http2 = v
                    }
                    #[cfg(feature = "http2")]
                    if let Some(v) = general.http2_tls_cert {
                        http2_tls_cert = Some(v)
                    }
                    #[cfg(feature = "http2")]
                    if let Some(v) = general.http2_tls_key {
                        http2_tls_key = Some(v)
                    }
                    if let Some(v) = general.security_headers {
                        security_headers = v
                    }
                    if let Some(ref v) = general.cors_allow_origins {
                        cors_allow_origins = v.to_owned()
                    }
                    if let Some(ref v) = general.cors_allow_headers {
                        cors_allow_headers = v.to_owned()
                    }
                    if let Some(v) = general.directory_listing {
                        directory_listing = v
                    }
                    if let Some(v) = general.directory_listing_order {
                        directory_listing_order = v
                    }
                    if let Some(ref v) = general.basic_auth {
                        basic_auth = v.to_owned()
                    }
                    if let Some(v) = general.fd {
                        fd = Some(v)
                    }
                    if let Some(v) = general.threads_multiplier {
                        threads_multiplier = v
                    }
                    if let Some(v) = general.grace_period {
                        grace_period = v
                    }
                    if let Some(v) = general.page_fallback {
                        page_fallback = Some(v)
                    }
                    if let Some(v) = general.log_remote_address {
                        log_remote_address = v
                    }
                }

                // Prepare the "advanced" options
                if let Some(advanced) = settings.advanced {
                    // 1. Custom HTTP headers assignment
                    let headers_entries = match advanced.headers {
                        Some(headers_entries) => {
                            let mut headers_vec: Vec<Headers> = Vec::new();

                            // Compile a glob pattern for each header sources entry
                            for headers_entry in headers_entries.iter() {
                                let source = Glob::new(&headers_entry.source)
                                    .with_context(|| {
                                        format!(
                                            "can not compile glob pattern for header source: {}",
                                            &headers_entry.source
                                        )
                                    })?
                                    .compile_matcher();

                                headers_vec.push(Headers {
                                    source,
                                    headers: headers_entry.headers.to_owned(),
                                });
                            }
                            Some(headers_vec)
                        }
                        _ => None,
                    };

                    settings_advanced = Some(Advanced {
                        headers: headers_entries,
                    });
                }
            }
        }

        Ok(Settings {
            general: General {
                host,
                port,
                root,
                log_level,
                config_file,
                cache_control_headers,
                compression,
                page404,
                page50x,
                #[cfg(feature = "http2")]
                http2,
                #[cfg(feature = "http2")]
                http2_tls_cert,
                #[cfg(feature = "http2")]
                http2_tls_key,
                security_headers,
                cors_allow_origins,
                cors_allow_headers,
                directory_listing,
                directory_listing_order,
                basic_auth,
                fd,
                threads_multiplier,
                grace_period,
                page_fallback,
                log_remote_address,

                // NOTE:
                // Windows-only options and commands
                #[cfg(windows)]
                windows_service: opts.windows_service,
                #[cfg(windows)]
                commands: opts.commands,
            },
            advanced: settings_advanced,
        })
    }
}
