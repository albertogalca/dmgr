use crate::output;
use crate::runner;
use std::path::Path;

use super::UploadResult;

/// Upload a file to S3
pub fn upload_file(
    bucket: &str,
    prefix: Option<&str>,
    file_path: &Path,
    region: Option<&str>,
    dry_run: bool,
) -> Result<UploadResult, String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;

    let s3_key = match prefix {
        Some(p) => format!("{}/{}", p.trim_matches('/'), file_name),
        None => file_name.to_string(),
    };

    let s3_uri = format!("s3://{}/{}", bucket, s3_key);
    let file_str = file_path.to_string_lossy();

    let mut args = vec!["s3", "cp", file_str.as_ref(), s3_uri.as_str()];

    let region_owned: String;
    if let Some(r) = region {
        region_owned = r.to_string();
        args.push("--region");
        args.push(&region_owned);
    }

    runner::run("aws", &args, dry_run).map_err(|e| format!("Failed to upload to S3: {}", e))?;

    // Construct the download URL
    let download_url = format!("https://{}.s3.amazonaws.com/{}", bucket, s3_key);

    output::success(&format!("Uploaded to S3: {}", download_url));

    Ok(UploadResult {
        download_url,
        metadata: Some(format!("Bucket: {}, Key: {}", bucket, s3_key)),
    })
}

/// Upload appcast.xml to S3
pub fn upload_appcast(
    bucket: &str,
    prefix: Option<&str>,
    appcast_path: &Path,
    region: Option<&str>,
    dry_run: bool,
) -> Result<String, String> {
    let s3_key = match prefix {
        Some(p) => format!("{}/appcast.xml", p.trim_matches('/')),
        None => "appcast.xml".to_string(),
    };

    let s3_uri = format!("s3://{}/{}", bucket, s3_key);
    let file_str = appcast_path.to_string_lossy();

    let mut args = vec![
        "s3",
        "cp",
        file_str.as_ref(),
        s3_uri.as_str(),
        "--content-type",
        "application/xml",
    ];

    let region_owned: String;
    if let Some(r) = region {
        region_owned = r.to_string();
        args.push("--region");
        args.push(&region_owned);
    }

    runner::run("aws", &args, dry_run)
        .map_err(|e| format!("Failed to upload appcast to S3: {}", e))?;

    let url = format!("https://{}.s3.amazonaws.com/{}", bucket, s3_key);
    output::success(&format!("Uploaded appcast to: {}", url));

    Ok(url)
}

/// Check if AWS CLI is available and configured
#[allow(dead_code)]
pub fn check_available() -> Result<(), String> {
    // Check if aws is installed
    which::which("aws")
        .map_err(|_| "aws CLI not found. Install with: brew install awscli".to_string())?;

    // Check if credentials are configured
    let args = vec!["sts", "get-caller-identity"];
    runner::run_capture("aws", &args).map_err(|_| {
        "AWS CLI not configured. Run: aws configure".to_string()
    })?;

    Ok(())
}
