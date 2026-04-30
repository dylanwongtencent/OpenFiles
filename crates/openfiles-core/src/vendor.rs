//! Vendor adapter construction.
//!
//! OpenFiles intentionally keeps the object-store contract small: stat, list,
//! ranged read, write, delete, and copy. OpenDAL is used for vendor coverage,
//! while local filesystem storage is available for tests and demos.

use crate::backend::ObjectBackend;
use crate::config::{BackendConfig, ProviderKind};
use crate::{LocalFsBackend, Result};
use std::sync::Arc;

pub fn build_backend(config: &BackendConfig) -> Result<Arc<dyn ObjectBackend>> {
    match config.provider {
        ProviderKind::LocalFs => Ok(Arc::new(LocalFsBackend::new(&config.root))),
        #[cfg(feature = "opendal-backend")]
        _ => build_opendal_backend(config),
        #[cfg(not(feature = "opendal-backend"))]
        _ => Err(crate::OpenFilesError::Unsupported(
            "build with feature `opendal-backend` for cloud providers".to_string(),
        )),
    }
}

#[cfg(feature = "opendal-backend")]
fn build_opendal_backend(config: &BackendConfig) -> Result<Arc<dyn ObjectBackend>> {
    use crate::backend::OpendalBackend;
    use opendal::services;
    use opendal::Operator;

    fn apply_s3_common(builder: &mut services::S3, config: &BackendConfig) {
        if !config.bucket.is_empty() {
            builder.bucket(&config.bucket);
        }
        if !config.root.is_empty() {
            builder.root(&config.root);
        }
        if let Some(endpoint) = &config.endpoint {
            builder.endpoint(endpoint);
        }
        if let Some(region) = &config.region {
            builder.region(region);
        }
        if let Some(access_key_id) = &config.access_key_id {
            builder.access_key_id(access_key_id);
        }
        if let Some(secret_access_key) = &config.secret_access_key {
            builder.secret_access_key(secret_access_key);
        }
        if let Some(session_token) = &config.session_token {
            builder.session_token(session_token);
        }
    }

    let operator = match config.provider {
        ProviderKind::AwsS3 | ProviderKind::S3Compatible | ProviderKind::Storj | ProviderKind::Minio | ProviderKind::NetappStorageGrid => {
            let mut builder = services::S3::default();
            apply_s3_common(&mut builder, config);
            Operator::new(builder)?.finish()
        }
        ProviderKind::GcpGcs => {
            let mut builder = services::Gcs::default();
            if !config.bucket.is_empty() {
                builder.bucket(&config.bucket);
            }
            if !config.root.is_empty() {
                builder.root(&config.root);
            }
            if let Some(endpoint) = &config.endpoint {
                builder.endpoint(endpoint);
            }
            if let Some(credential) = &config.credential {
                builder.credential(credential);
            }
            if let Some(credential_path) = &config.credential_path {
                builder.credential_path(credential_path);
            }
            Operator::new(builder)?.finish()
        }
        ProviderKind::AzureBlob => {
            let mut builder = services::Azblob::default();
            if !config.root.is_empty() {
                builder.root(&config.root);
            }
            if !config.container.is_empty() {
                builder.container(&config.container);
            }
            if let Some(endpoint) = &config.endpoint {
                builder.endpoint(endpoint);
            }
            if let Some(account_name) = &config.account_name {
                builder.account_name(account_name);
            }
            if let Some(account_key) = &config.account_key {
                builder.account_key(account_key);
            }
            if let Some(sas_token) = &config.sas_token {
                builder.sas_token(sas_token);
            }
            Operator::new(builder)?.finish()
        }
        ProviderKind::VercelBlob => {
            let mut builder = services::VercelBlob::default();
            if !config.root.is_empty() {
                builder.root(&config.root);
            }
            if let Some(token) = &config.token {
                builder.token(token);
            }
            Operator::new(builder)?.finish()
        }
        ProviderKind::LocalFs => unreachable!("handled by build_backend"),
    };

    Ok(Arc::new(OpendalBackend::new(operator)))
}
