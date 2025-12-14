#![allow(unused_imports)]
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[cfg(feature = "embed_elements")]
use rust_embed::RustEmbed;

#[cfg(feature = "embed_elements")]
#[derive(RustEmbed)]
#[folder = "assets/elements/"]
pub struct ElementsAssets;

#[cfg(feature = "embed_elements")]
pub async fn serve_elements_asset(
    axum::extract::Path(file): axum::extract::Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(content) = ElementsAssets::get(&file) {
        let mime_type = content_type_for(&file);
        let body = content.data.into_owned();
        Ok(([(axum::http::header::CONTENT_TYPE, mime_type)], body))
    } else {
        tracing::warn!("Elements asset not found: {}", file);
        Err(StatusCode::NOT_FOUND)
    }
}

#[cfg(feature = "embed_elements")]
fn content_type_for(file: &str) -> &'static str {
    match file.rsplit('.').next().unwrap_or("") {
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}
