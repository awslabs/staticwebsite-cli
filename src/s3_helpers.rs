use crate::Error;
use aws_sdk_s3::Client;
use mime_guess;
use std::fs;
use std::path::Path;
use aws_smithy_http::byte_stream::ByteStream;
use tracing::info;

#[derive(Clone)]
struct UploadTask {
    source: Box<Path>,
    destination_bucket: String,
    destination_path: String,
}

///
/// Uploads the contents of the given directory to the given bucket in S3.
///
pub async fn upload_directory(
    directory: &Path,
    destination_bucket: &String,
    s3_client: &Client,
) -> Result<(), Error> {
    let tasks = directory_to_tasks("".to_string(), directory, destination_bucket)?;

    // Upload sequentially for now
    for task in tasks {
        upload_file(&task, s3_client).await?;
    }

    Ok(())
}

fn directory_to_tasks(
    base: String,
    directory: &Path,
    destination_bucket: &String,
) -> Result<Vec<UploadTask>, Error> {
    let mut ret: Vec<UploadTask> = vec![];

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let subdir_name = format!(
                "{}",
                path.file_name()
                    .expect("subdirs have a name")
                    .to_str()
                    .expect("names contain a string")
            );
            let next_dir = if base == "" {
                subdir_name
            } else {
                format!("{}/{}", base, subdir_name)
            };
            let subdir_results = directory_to_tasks(next_dir, path.as_path(), destination_bucket)?;
            ret = [ret.as_slice(), subdir_results.as_slice()].concat()
        } else {
            let file_name = format!(
                "{}",
                path.file_name()
                    .expect("files have a name")
                    .to_str()
                    .expect("names contain a string")
            );
            let destination_path = if base == "" {
                file_name
            } else {
                format!("{}/{}", base, file_name)
            };
            ret.push(UploadTask {
                destination_path,
                source: path.into_boxed_path(),
                destination_bucket: destination_bucket.clone(),
            })
        }
    }

    return Ok(ret);
}

async fn upload_file(task: &UploadTask, s3_client: &Client) -> Result<(), Error> {
    info!(file = task.destination_path, "Uploading");
    let body = ByteStream::from_path(&task.source).await;
    let mime_type = mime_guess::from_path(&task.source).first().unwrap();
    let body_contents = body?;

    let _ = s3_client
        .put_object()
        .bucket(&task.destination_bucket)
        .key(&task.destination_path)
        .body(body_contents)
        .content_type(mime_type.to_string())
        .send()
        .await?;

    return Ok(());
}
