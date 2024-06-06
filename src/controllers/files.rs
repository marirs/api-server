use crate::secure::guards::ApiKeyGuard;
use crate::server::config::Settings;
use chrono::Datelike;
use chrono::Utc;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{fs::NamedFile, http::Status, Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use rocket_okapi::JsonSchema;
use std::fs;
use std::path::Path;

#[derive(Serialize, JsonSchema)]
pub struct UploadResponse {
    status: String,
    path: String,
}

#[openapi(tag = "Files")]
#[post("/upload", data = "<data>")]
pub async fn upload_file(
    content_type: &rocket::http::ContentType,
    data: Data<'_>,
    config: &State<Settings>,
    _api_guard: ApiKeyGuard,
) -> Result<Json<UploadResponse>, Status> {
    let now = Utc::now();
    let storage_dir = Path::new(&config.server.storage_path)
        .join(now.year().to_string())
        .join(format!("{:02}", now.month()));

    if let Err(e) = fs::create_dir_all(&storage_dir) {
        eprintln!("Failed to create directory: {}", e);
        return Err(Status::InternalServerError);
    }

    let mut options = MultipartFormDataOptions::new();
    options.allowed_fields.push(
        MultipartFormDataField::file("file")
            .size_limit(rocket::data::ByteUnit::max_value().as_u64()),
    );

    let form_data = match MultipartFormData::parse(content_type, data, options).await {
        Ok(form_data) => form_data,
        Err(e) => {
            eprintln!("Failed to parse form data: {:?}", e);
            return Err(Status::BadRequest);
        }
    };

    if let Some(file_field) = form_data.files.get("file").and_then(|files| files.get(0)) {
        let file_name = file_field
            .file_name
            .clone()
            .unwrap_or("uploaded_file".to_string());
        let file_path = storage_dir.join(file_name);

        if let Err(e) = fs::copy(&file_field.path, &file_path) {
            eprintln!("Failed to save file: {}", e);
            return Err(Status::InternalServerError);
        }

        Ok(Json(UploadResponse {
            status: "success".into(),
            path: file_path.to_string_lossy().into(),
        }))
    } else {
        Err(Status::BadRequest)
    }
}

#[openapi(tag = "Users")]
#[get("/download/<year>/<month>/<file>")]
pub async fn download_file(
    year: i32,
    month: u32,
    file: String,
    config: &State<Settings>,
    _api_guard: ApiKeyGuard,
) -> Result<NamedFile, Status> {
    let file_path = Path::new(&config.server.storage_path)
        .join(year.to_string())
        .join(format!("{:02}", month))
        .join(file);

    NamedFile::open(&file_path)
        .await
        .map_err(|_| Status::NotFound)
}
