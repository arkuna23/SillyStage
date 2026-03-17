use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use handler::HandlerError;
use protocol::ErrorPayload;

use crate::ServerState;

const FILE_NAME_HEADER: &str = "x-file-name";

pub async fn upload(
    State(state): State<ServerState>,
    Path((resource_id, file_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let file_name = read_file_name(&headers);
    let content_type = read_content_type(&headers);

    match state
        .handler()
        .upload_resource_file(
            &resource_id,
            &file_id,
            file_name,
            content_type,
            body.to_vec(),
        )
        .await
    {
        Ok(payload) => (StatusCode::OK, Json(payload)).into_response(),
        Err(error) => binary_error_response(error),
    }
}

pub async fn download(
    State(state): State<ServerState>,
    Path((resource_id, file_id)): Path<(String, String)>,
) -> Response {
    match state
        .handler()
        .download_resource_file(&resource_id, &file_id)
        .await
    {
        Ok(asset) => binary_asset_response(asset),
        Err(error) => binary_error_response(error),
    }
}

fn binary_asset_response(asset: handler::BinaryAsset) -> Response {
    let mut response = (
        StatusCode::OK,
        [(CONTENT_TYPE, asset.content_type)],
        asset.bytes,
    )
        .into_response();

    if let Some(file_name) = asset.file_name {
        if let Ok(value) = HeaderValue::from_str(&content_disposition_value(&file_name)) {
            response.headers_mut().insert(CONTENT_DISPOSITION, value);
        }
    }

    response
}

fn content_disposition_value(file_name: &str) -> String {
    let safe_file_name = file_name.replace('"', "_").replace('\\', "_");
    format!("attachment; filename=\"{safe_file_name}\"")
}

fn read_content_type(headers: &HeaderMap) -> String {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "application/octet-stream".to_owned())
}

fn read_file_name(headers: &HeaderMap) -> Option<String> {
    headers
        .get(FILE_NAME_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn binary_error_response(error: HandlerError) -> Response {
    let payload = error.to_error_payload();
    (status_from_error(&payload), Json(payload)).into_response()
}

fn status_from_error(error: &ErrorPayload) -> StatusCode {
    match error.code {
        -32_700 | -32_600 | -32_602 => StatusCode::BAD_REQUEST,
        40_404 => StatusCode::NOT_FOUND,
        40_909 => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
