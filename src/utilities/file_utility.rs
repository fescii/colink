use actix_files as fs;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{Error, HttpRequest};
use actix_multipart::form::{
	tempfile::TempFile,
	MultipartForm,
};
use std::fmt;
use std::path::PathBuf;

// Custom file uploading error
#[derive(Debug)]
pub struct UploadError {
	pub message: String
}

impl fmt::Display for UploadError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,"{}", self.message)
	}
}

impl std::error::Error for UploadError{}


#[derive(Debug, MultipartForm)]
pub struct UploadForm {
  #[multipart(rename = "file")]
  pub file: TempFile,
}


pub async fn upload_file(
	payload: MultipartForm<UploadForm>,
	name: &str,
	static_root: &str,
	path_to: &str,
) -> Result<String, UploadError> {

	const MAX_FILE_SIZE: u64 = 1024 * 1024 * 10; // 10 MB

	// reject malformed requests - If size is zero or less
	if payload.file.size <= 0 {
		return Err(UploadError{
			message: "The uploaded file cannot be of size zero bytes.".to_string()
		})
	}
	// reject malformed requests - If size is greater  than 10Mbs
	if payload.file.size > (1024 * 1024 * 10) {
		return Err(UploadError{
			message: format!("The uploaded file is too large. Maximum size should be {} bytes.", MAX_FILE_SIZE).to_string()
		})
	}


	let temp_file_path = payload.file.file.path();

	// let dir = std::env::temp_dir();
  // println!("Temporary directory is: {:?}", payload.file);

	let original_filename = payload.file.file_name.clone().unwrap();

  let extension = original_filename.rsplit('.').next().unwrap();

  // Generate a new unique filename with the same extension
  let new_filename = format!("{}.{}", name, extension);

	// Create path to save file
	let mut path = PathBuf::from(static_root);
	path.push(path_to);
	path.push(&new_filename);

	// Create string path
	let path_str = format!("{}/{}", path_to, new_filename);


	match std::fs::rename(temp_file_path, path.clone()) {
    Ok(_) => Ok(path_str),
    Err(_) => Err(UploadError{
			message: "Could not update the logo, Error occurred while uploading!".to_string()
		})
	}
}

// Serving static file starter path
pub async fn index(req: HttpRequest) -> Result<fs::NamedFile, Error> {
	let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
	let file = fs::NamedFile::open(path)?;
	Ok(file
		.use_last_modified(true)
		.set_content_disposition(ContentDisposition {
			disposition: DispositionType::Attachment,
			parameters: vec![],
		}))
}
