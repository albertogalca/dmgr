pub mod github;
pub mod s3;

/// Result of uploading to a distribution target
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// The download URL for the uploaded file
    pub download_url: String,
    /// Optional metadata about the upload
    #[allow(dead_code)]
    pub metadata: Option<String>,
}
