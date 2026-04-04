use std::{
    collections::HashMap,
    fs, io,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use axum::http::StatusCode;
use parking_lot::RwLock;
use s3::{AddressingStyle, Auth, Client, Credentials};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{
    config::Config,
    storage::process_image::{ProcessImageError, process_image},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRef {
    pub path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbhash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UploadedFileRecord {
    pub account_id: String,
    pub path: String,
    pub reuse_key: String,
    pub width: u32,
    pub height: u32,
    pub thumbhash: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl UploadedFileRecord {
    fn image_ref(&self) -> ImageRef {
        ImageRef {
            path: self.path.clone(),
            width: Some(self.width),
            height: Some(self.height),
            thumbhash: Some(self.thumbhash.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct UploadedFileIndex {
    by_reuse_key: HashMap<String, UploadedFileRecord>,
}

impl UploadedFileIndex {
    fn load(path: &Path) -> Result<Self, FileStorageError> {
        match fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(FileStorageError::Io(error)),
        }
    }
}

#[derive(Clone)]
enum FileBackend {
    Local { files_root: PathBuf },
    S3Compatible { client: Client, bucket: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicUrlMode {
    ProxyThroughServer,
    DirectBaseUrl,
}

#[derive(Debug, Error)]
pub enum FileStorageError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ProcessImage(#[from] ProcessImageError),
    #[error(transparent)]
    Metadata(#[from] serde_json::Error),
    #[error(transparent)]
    S3(#[from] s3::Error),
    #[error("invalid file path")]
    InvalidPath,
    #[error("file not found")]
    NotFound,
    #[error("invalid file storage configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Clone)]
pub struct FileStorage {
    data_dir: Arc<PathBuf>,
    public_base_url: Arc<String>,
    public_url_mode: PublicUrlMode,
    backend: Arc<FileBackend>,
    metadata_path: Arc<PathBuf>,
    metadata_index: Arc<RwLock<UploadedFileIndex>>,
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new_for_tests(PathBuf::from("./data"), "http://127.0.0.1:3005".into())
    }
}

impl FileStorage {
    pub fn new(config: &Config) -> Self {
        Self::try_new(config)
            .unwrap_or_else(|error| panic!("failed to initialize file storage: {error}"))
    }

    pub fn new_for_tests(data_dir: PathBuf, public_base_url: String) -> Self {
        Self::with_backend(
            data_dir,
            public_base_url,
            PublicUrlMode::ProxyThroughServer,
            FileBackend::Local {
                files_root: PathBuf::from("./unused"),
            },
        )
        .with_local_files_root()
    }

    #[cfg(test)]
    fn new_s3_for_tests(
        data_dir: PathBuf,
        endpoint: String,
        bucket: String,
        public_base_url: String,
    ) -> Self {
        let client = Client::builder(endpoint)
            .expect("valid mock s3 endpoint")
            .region("us-east-1")
            .auth(Auth::Anonymous)
            .addressing_style(AddressingStyle::Path)
            .build()
            .expect("mock s3 client should build");
        Self::with_backend(
            data_dir,
            public_base_url,
            PublicUrlMode::DirectBaseUrl,
            FileBackend::S3Compatible { client, bucket },
        )
    }

    fn try_new(config: &Config) -> Result<Self, FileStorageError> {
        let data_dir = env_value(&["VIBE_DATA_DIR", "DATA_DIR"])
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./data"));

        if let Some(endpoint) = s3_endpoint_from_env() {
            let bucket = env_value(&["VIBE_S3_BUCKET", "S3_BUCKET"])
                .ok_or_else(|| FileStorageError::InvalidConfig("missing S3 bucket".into()))?;
            let region =
                env_value(&["VIBE_S3_REGION", "S3_REGION"]).unwrap_or_else(|| "us-east-1".into());
            let auth = s3_auth_from_env()?;
            let client = Client::builder(endpoint.clone())?
                .region(region)
                .auth(auth)
                .addressing_style(AddressingStyle::Path)
                .build()?;
            let public_base_url = env_value(&["VIBE_S3_PUBLIC_URL", "S3_PUBLIC_URL"])
                .unwrap_or_else(|| format!("{}/{}", endpoint.trim_end_matches('/'), bucket));

            return Ok(Self::with_backend(
                data_dir,
                public_base_url,
                PublicUrlMode::DirectBaseUrl,
                FileBackend::S3Compatible { client, bucket },
            ));
        }

        let public_base_url = env_value(&["VIBE_PUBLIC_URL", "PUBLIC_URL"])
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", config.port));
        Ok(Self::with_backend(
            data_dir,
            public_base_url,
            PublicUrlMode::ProxyThroughServer,
            FileBackend::Local {
                files_root: PathBuf::from("./unused"),
            },
        )
        .with_local_files_root())
    }

    fn with_backend(
        data_dir: PathBuf,
        public_base_url: String,
        public_url_mode: PublicUrlMode,
        backend: FileBackend,
    ) -> Self {
        let metadata_path = data_dir.join("uploaded-files.json");
        let metadata_index = UploadedFileIndex::load(&metadata_path).unwrap_or_default();
        Self {
            data_dir: Arc::new(data_dir),
            public_base_url: Arc::new(public_base_url.trim_end_matches('/').to_string()),
            public_url_mode,
            backend: Arc::new(backend),
            metadata_path: Arc::new(metadata_path),
            metadata_index: Arc::new(RwLock::new(metadata_index)),
        }
    }

    fn with_local_files_root(mut self) -> Self {
        let files_root = self.data_dir.join("files");
        self.backend = Arc::new(FileBackend::Local { files_root });
        self
    }

    pub fn root_dir(&self) -> &Path {
        match self.backend.as_ref() {
            FileBackend::Local { files_root } => files_root.as_path(),
            FileBackend::S3Compatible { .. } => self.data_dir.as_ref(),
        }
    }

    pub fn public_url(&self, path: &str) -> String {
        match self.public_url_mode {
            PublicUrlMode::ProxyThroughServer => {
                format!("{}/files/{}", self.public_base_url, path)
            }
            PublicUrlMode::DirectBaseUrl => {
                format!("{}/{}", self.public_base_url, path)
            }
        }
    }

    pub fn resolve_image_json(&self, value: Option<&Value>) -> Option<Value> {
        let mut value = value?.clone();
        let Some(path) = value
            .get("path")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
        else {
            return Some(value);
        };
        if let Value::Object(ref mut object) = value {
            object.insert("url".into(), Value::String(self.public_url(&path)));
        }
        Some(value)
    }

    pub async fn store_image(
        &self,
        user_id: &str,
        directory: &str,
        prefix: &str,
        reuse_key: Option<&str>,
        bytes: &[u8],
    ) -> Result<ImageRef, FileStorageError> {
        if let Some(reuse_key) = reuse_key
            && let Some(existing) = self.lookup_reuse_key(reuse_key)
        {
            if self.exists(&existing.path).await? {
                return Ok(existing.image_ref());
            }
            self.remove_reuse_key(reuse_key)?;
        }

        let processed = process_image(bytes)?;
        let filename = format!(
            "{}_{}.{}",
            prefix,
            uuid::Uuid::now_v7(),
            processed.format_extension
        );
        let path = format!("public/users/{user_id}/{directory}/{filename}");
        self.put(&path, bytes).await?;
        let image_ref = ImageRef {
            path: path.clone(),
            width: Some(processed.width),
            height: Some(processed.height),
            thumbhash: Some(processed.thumbhash.clone()),
        };

        if let Some(reuse_key) = reuse_key {
            self.persist_uploaded_file(UploadedFileRecord {
                account_id: user_id.to_string(),
                path,
                reuse_key: reuse_key.to_string(),
                width: processed.width,
                height: processed.height,
                thumbhash: processed.thumbhash,
                created_at: crate::storage::db::now_ms(),
                updated_at: crate::storage::db::now_ms(),
            })?;
        }

        Ok(image_ref)
    }

    pub async fn put(&self, path: &str, bytes: &[u8]) -> Result<(), FileStorageError> {
        self.validate_path(path)?;
        match self.backend.as_ref() {
            FileBackend::Local { files_root } => {
                let full_path = files_root.join(path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(full_path, bytes)?;
                Ok(())
            }
            FileBackend::S3Compatible { client, bucket } => {
                client
                    .objects()
                    .put(bucket, path)
                    .content_type(content_type_for_path(path))
                    .body_bytes(bytes.to_vec())
                    .send()
                    .await?;
                Ok(())
            }
        }
    }

    pub async fn get(&self, path: &str) -> Result<Vec<u8>, FileStorageError> {
        self.validate_path(path)?;
        match self.backend.as_ref() {
            FileBackend::Local { files_root } => {
                let full_path = files_root.join(path);
                match fs::read(full_path) {
                    Ok(bytes) => Ok(bytes),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        Err(FileStorageError::NotFound)
                    }
                    Err(error) => Err(FileStorageError::Io(error)),
                }
            }
            FileBackend::S3Compatible { client, bucket } => {
                let object = client
                    .objects()
                    .get(bucket, path)
                    .send()
                    .await
                    .map_err(|error| {
                        if error.status() == Some(StatusCode::NOT_FOUND) {
                            FileStorageError::NotFound
                        } else {
                            FileStorageError::S3(error)
                        }
                    })?;
                Ok(object.bytes().await?.to_vec())
            }
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, FileStorageError> {
        self.validate_path(path)?;
        match self.backend.as_ref() {
            FileBackend::Local { files_root } => Ok(files_root.join(path).exists()),
            FileBackend::S3Compatible { client, bucket } => {
                match client.objects().head(bucket, path).send().await {
                    Ok(_) => Ok(true),
                    Err(error) if error.status() == Some(StatusCode::NOT_FOUND) => Ok(false),
                    Err(error) => Err(FileStorageError::S3(error)),
                }
            }
        }
    }

    fn lookup_reuse_key(&self, reuse_key: &str) -> Option<UploadedFileRecord> {
        self.metadata_index
            .read()
            .by_reuse_key
            .get(reuse_key)
            .cloned()
    }

    fn remove_reuse_key(&self, reuse_key: &str) -> Result<(), FileStorageError> {
        self.metadata_index.write().by_reuse_key.remove(reuse_key);
        self.flush_metadata_index()
    }

    fn persist_uploaded_file(&self, record: UploadedFileRecord) -> Result<(), FileStorageError> {
        self.metadata_index
            .write()
            .by_reuse_key
            .insert(record.reuse_key.clone(), record);
        self.flush_metadata_index()
    }

    fn flush_metadata_index(&self) -> Result<(), FileStorageError> {
        if let Some(parent) = self.metadata_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&*self.metadata_index.read())?;
        let temp_path = self.metadata_path.with_extension("json.tmp");
        fs::write(&temp_path, bytes)?;
        fs::rename(temp_path, self.metadata_path.as_ref())?;
        Ok(())
    }

    fn validate_path(&self, path: &str) -> Result<(), FileStorageError> {
        let relative = Path::new(path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(FileStorageError::InvalidPath);
        }
        Ok(())
    }
}

fn content_type_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn env_value(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn s3_endpoint_from_env() -> Option<String> {
    if let Some(endpoint) = env_value(&["VIBE_S3_ENDPOINT"]) {
        return Some(endpoint);
    }
    let host = env_value(&["S3_HOST"])?;
    let use_ssl = env_value(&["S3_USE_SSL"])
        .map(|value| value == "true")
        .unwrap_or(true);
    let scheme = if use_ssl { "https" } else { "http" };
    let port = env_value(&["S3_PORT"]);
    Some(match port {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    })
}

fn s3_auth_from_env() -> Result<Auth, FileStorageError> {
    let access_key = env_value(&["VIBE_S3_ACCESS_KEY", "S3_ACCESS_KEY", "AWS_ACCESS_KEY_ID"]);
    let secret_key = env_value(&[
        "VIBE_S3_SECRET_KEY",
        "S3_SECRET_KEY",
        "AWS_SECRET_ACCESS_KEY",
    ]);
    match (access_key, secret_key) {
        (Some(access_key), Some(secret_key)) => {
            let mut credentials = Credentials::new(access_key, secret_key)?;
            if let Some(session_token) = env_value(&["VIBE_S3_SESSION_TOKEN", "AWS_SESSION_TOKEN"])
            {
                credentials = credentials.with_session_token(session_token)?;
            }
            Ok(Auth::Static(credentials))
        }
        (None, None) => Ok(Auth::Anonymous),
        _ => Err(FileStorageError::InvalidConfig(
            "S3 access key and secret key must both be set".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    use axum::{
        Router,
        body::{Body, Bytes},
        extract::{Path as AxumPath, State},
        http::StatusCode,
        response::IntoResponse,
        routing::put,
    };
    use image::{ImageBuffer, ImageFormat, Rgba};
    use parking_lot::RwLock;

    use super::FileStorage;

    #[tokio::test]
    async fn file_storage_round_trip_works() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::new_for_tests(
            PathBuf::from(temp_dir.path()),
            "http://localhost:3005".into(),
        );
        storage.put("img/test.png", &[1, 2, 3]).await.unwrap();
        assert_eq!(storage.get("img/test.png").await.unwrap(), vec![1, 2, 3]);
        assert_eq!(
            storage.public_url("img/test.png"),
            "http://localhost:3005/files/img/test.png"
        );
    }

    #[tokio::test]
    async fn store_image_persists_file_and_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::new_for_tests(
            PathBuf::from(temp_dir.path()),
            "http://localhost:3005".into(),
        );
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();

        let image_ref = storage
            .store_image(
                "acct",
                "avatars",
                "github",
                Some("image-url:test"),
                &cursor.into_inner(),
            )
            .await
            .unwrap();
        assert!(image_ref.path.contains("public/users/acct/avatars/github_"));
        assert_eq!(image_ref.width, Some(2));
        assert!(storage.get(&image_ref.path).await.is_ok());
    }

    #[tokio::test]
    async fn store_image_reuses_reference_for_same_reuse_key_across_instances() {
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = PathBuf::from(temp_dir.path());
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        let bytes = cursor.into_inner();

        let first_storage =
            FileStorage::new_for_tests(data_dir.clone(), "http://localhost:3005".into());
        let first = first_storage
            .store_image("acct", "avatars", "github", Some("image-url:test"), &bytes)
            .await
            .unwrap();

        let second_storage = FileStorage::new_for_tests(data_dir, "http://localhost:3005".into());
        let second = second_storage
            .store_image("acct", "avatars", "github", Some("image-url:test"), &bytes)
            .await
            .unwrap();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn s3_compatible_backend_round_trips_objects() {
        let temp_dir = tempfile::tempdir().unwrap();
        let server = MockS3Server::spawn().await;
        let storage = FileStorage::new_s3_for_tests(
            PathBuf::from(temp_dir.path()),
            server.endpoint(),
            "bucket".into(),
            format!("{}/bucket", server.endpoint()),
        );

        storage.put("img/test.txt", b"hello s3").await.unwrap();
        let bytes = storage.get("img/test.txt").await.unwrap();

        assert_eq!(bytes, b"hello s3");
        assert_eq!(
            storage.public_url("img/test.txt"),
            format!("{}/bucket/img/test.txt", server.endpoint())
        );
    }

    type ObjectMap = Arc<RwLock<HashMap<String, Vec<u8>>>>;

    struct MockS3Server {
        endpoint: String,
    }

    impl MockS3Server {
        async fn spawn() -> Self {
            async fn put_object(
                State(objects): State<ObjectMap>,
                AxumPath((bucket, key)): AxumPath<(String, String)>,
                body: Bytes,
            ) -> impl IntoResponse {
                objects
                    .write()
                    .insert(format!("{bucket}/{key}"), body.to_vec());
                StatusCode::OK
            }

            async fn head_object(
                State(objects): State<ObjectMap>,
                AxumPath((bucket, key)): AxumPath<(String, String)>,
            ) -> impl IntoResponse {
                if objects.read().contains_key(&format!("{bucket}/{key}")) {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            }

            async fn get_object(
                State(objects): State<ObjectMap>,
                AxumPath((bucket, key)): AxumPath<(String, String)>,
            ) -> impl IntoResponse {
                match objects.read().get(&format!("{bucket}/{key}")).cloned() {
                    Some(bytes) => (StatusCode::OK, Body::from(bytes)).into_response(),
                    None => StatusCode::NOT_FOUND.into_response(),
                }
            }

            let objects: ObjectMap = Arc::new(RwLock::new(HashMap::new()));
            let app = Router::new()
                .route(
                    "/{bucket}/{*key}",
                    put(put_object).get(get_object).head(head_object),
                )
                .with_state(objects);
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            Self {
                endpoint: format!("http://{address}"),
            }
        }

        fn endpoint(&self) -> String {
            self.endpoint.clone()
        }
    }
}
