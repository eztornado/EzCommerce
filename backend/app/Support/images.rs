//! Product image processing: EXIF orientation, responsive resizing and
//! modern encoding (AVIF + JPEG/PNG fallback) with SEO-friendly file names.
//!
//! Uploaded files are stored under `public/storage/products` as
//! `<slug>-<hash>-w<width>.<ext>` plus `<slug>-<hash>-w<width>.avif`. A small
//! JSON sidecar (`<slug>-<hash>.json`) records dimensions and variants so the
//! product API can build `srcset` without hitting the database.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder, ImageReader};
use larust_support::serde_json::{json, Value};

use super::response::{ApiError, ApiResult};
use super::slug;

const PRODUCT_TARGET_WIDTHS: [u32; 4] = [160, 360, 720, 1200];
const HERO_TARGET_WIDTHS: [u32; 4] = [640, 960, 1280, 1600];
const MAX_WIDTH: u32 = 1600;
const JPEG_QUALITY: u8 = 82;
const AVIF_SPEED: u8 = 10;
const AVIF_QUALITY: u8 = 60;

static METADATA_CACHE: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, Value>> {
    METADATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Folders whose uploads go through the responsive image pipeline.
pub fn is_responsive_folder(folder: &str) -> bool {
    matches!(folder, "product" | "products" | "hero")
}

fn encode_jpeg(image: &DynamicImage) -> ApiResult<Vec<u8>> {
    let rgb = image.to_rgb8();
    let mut output = Vec::new();
    JpegEncoder::new_with_quality(&mut output, JPEG_QUALITY)
        .encode(&rgb, rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
        .map_err(|error| ApiError::internal(format!("JPEG encoding failed: {error}")))?;
    Ok(output)
}

fn encode_png(image: &DynamicImage) -> ApiResult<Vec<u8>> {
    let rgba = image.to_rgba8();
    let mut output = Vec::new();
    PngEncoder::new_with_quality(
        &mut output,
        CompressionType::Fast,
        PngFilter::Adaptive,
    )
    .write_image(
        &rgba,
        rgba.width(),
        rgba.height(),
        ExtendedColorType::Rgba8,
    )
    .map_err(|error| ApiError::internal(format!("PNG encoding failed: {error}")))?;
    Ok(output)
}

fn encode_avif(image: &DynamicImage) -> ApiResult<Vec<u8>> {
    let rgba = image.to_rgba8();
    let mut output = Vec::new();
    AvifEncoder::new_with_speed_quality(&mut output, AVIF_SPEED, AVIF_QUALITY)
        .write_image(
            &rgba,
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|error| ApiError::internal(format!("AVIF encoding failed: {error}")))?;
    Ok(output)
}

fn write_file(path: &std::path::Path, bytes: &[u8]) -> ApiResult<()> {
    std::fs::write(path, bytes)
        .map_err(|error| ApiError::internal(format!("Unable to store the image: {error}")))
}

fn apply_exif_orientation(image: DynamicImage, bytes: &[u8]) -> DynamicImage {
    let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) else {
        return image;
    };
    let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) else {
        return image;
    };
    match field.value.get_uint(0).unwrap_or(1) {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.rotate90().fliph(),
        6 => image.rotate90(),
        7 => image.rotate270().fliph(),
        8 => image.rotate270(),
        _ => image,
    }
}

fn target_widths(source_width: u32, targets: &[u32]) -> Vec<u32> {
    let mut widths: Vec<u32> = targets
        .iter()
        .copied()
        .filter(|width| *width < source_width)
        .collect();
    widths.push(source_width.min(MAX_WIDTH));
    widths.sort_unstable();
    widths.dedup();
    widths
}

fn looks_like_svg(bytes: &[u8], filename: &str) -> bool {
    if filename.to_ascii_lowercase().ends_with(".svg") {
        return true;
    }
    let prefix = String::from_utf8_lossy(&bytes[..bytes.len().min(256)]);
    prefix.trim_start().starts_with("<svg")
}

fn base_stem(filename: &str) -> String {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("product");
    let mut slugged = slug::slug(stem);
    if slugged.is_empty() {
        slugged = "product".to_string();
    }
    slugged.truncate(60);
    while slugged.ends_with('-') {
        slugged.pop();
    }
    slugged
}

/// Processes a product upload synchronously (call it inside
/// `spawn_blocking`). Returns the main path plus responsive variants.
pub fn process_product_image(bytes: &[u8], original_filename: &str) -> ApiResult<Value> {
    process_responsive_image(
        bytes,
        original_filename,
        "products",
        &PRODUCT_TARGET_WIDTHS,
    )
}

/// Responsive hero/banner upload (same pipeline, wider sizes).
pub fn process_hero_image(bytes: &[u8], original_filename: &str) -> ApiResult<Value> {
    process_responsive_image(bytes, original_filename, "hero", &HERO_TARGET_WIDTHS)
}

fn process_responsive_image(
    bytes: &[u8],
    original_filename: &str,
    folder: &str,
    target_widths_list: &[u32],
) -> ApiResult<Value> {
    let stem = format!(
        "{}-{}",
        base_stem(original_filename),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let dir = crate::paths().join("public/storage").join(folder);
    std::fs::create_dir_all(&dir)
        .map_err(|error| ApiError::internal(format!("Unable to create the image folder: {error}")))?;

    // SVG is kept as-is (vector, no raster variants).
    if looks_like_svg(bytes, original_filename) {
        let filename = format!("{stem}.svg");
        write_file(&dir.join(&filename), bytes)?;
        let path = format!("{folder}/{filename}");
        return Ok(json!({
            "path": path,
            "url": format!("/storage/{path}"),
            "mime": "image/svg+xml",
            "width": Value::Null,
            "height": Value::Null,
            "variants": []
        }));
    }

    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| ApiError::unprocessable(format!("Unable to detect the image format: {error}")))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8_000);
    limits.max_image_height = Some(8_000);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| {
        ApiError::unprocessable(format!(
            "Unable to read the image. Use JPG, PNG, WebP, AVIF, GIF, BMP or TIFF: {error}"
        ))
    })?;
    let image = apply_exif_orientation(image, bytes);
    let (source_width, source_height) = image.dimensions();
    if source_width == 0 || source_height == 0 {
        return Err(ApiError::unprocessable("The image has no valid dimensions."));
    }

    let has_alpha = image.color().has_alpha();
    let fallback_extension = if has_alpha { "png" } else { "jpg" };
    let fallback_mime = if has_alpha { "image/png" } else { "image/jpeg" };

    let mut variants = Vec::new();
    let mut main_path = String::new();
    let mut main_width = source_width;
    let mut main_height = source_height;

    for width in target_widths(source_width, target_widths_list) {
        let height = ((source_height as f64 * width as f64) / source_width as f64)
            .round()
            .max(1.0) as u32;
        let resized = if width == source_width {
            image.clone()
        } else {
            image.resize_exact(width, height, FilterType::Lanczos3)
        };

        let fallback_name = format!("{stem}-w{width}.{fallback_extension}");
        let avif_name = format!("{stem}-w{width}.avif");
        let fallback_bytes = if has_alpha {
            encode_png(&resized)?
        } else {
            encode_jpeg(&resized)?
        };
        let avif_bytes = encode_avif(&resized)?;
        write_file(&dir.join(&fallback_name), &fallback_bytes)?;
        write_file(&dir.join(&avif_name), &avif_bytes)?;

        variants.push(json!({
            "width": width,
            "height": height,
            "src": format!("{folder}/{fallback_name}"),
            "avif": format!("{folder}/{avif_name}")
        }));
        main_path = format!("{folder}/{fallback_name}");
        main_width = width;
        main_height = height;
    }

    let metadata = json!({
        "width": main_width,
        "height": main_height,
        "format": fallback_extension,
        "mime": fallback_mime,
        "variants": variants
    });
    write_file(
        &dir.join(format!("{stem}.json")),
        metadata.to_string().as_bytes(),
    )?;

    Ok(json!({
        "path": main_path,
        "url": format!("/storage/{main_path}"),
        "mime": fallback_mime,
        "width": main_width,
        "height": main_height,
        "variants": variants
    }))
}

fn sidecar_for(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let (without_extension, _) = filename.rsplit_once('.')?;
    let (stem, width) = without_extension.rsplit_once("-w")?;
    if width.parse::<u32>().is_err() {
        return None;
    }
    let directory = &path[..path.len() - filename.len()];
    Some(format!("{directory}{stem}.json"))
}

/// Reads the cached sidecar metadata for a product image (if it exists).
pub fn metadata_for(path: &str) -> Option<Value> {
    let sidecar = sidecar_for(path)?;
    if let Ok(cache) = cache().lock() {
        if let Some(metadata) = cache.get(&sidecar) {
            return Some(metadata.clone());
        }
    }
    let full = crate::paths().join("public/storage").join(&sidecar);
    let raw = std::fs::read_to_string(full).ok()?;
    let metadata: Value = larust_support::serde_json::from_str(&raw).ok()?;
    if let Ok(mut cache) = cache().lock() {
        cache.insert(sidecar, metadata.clone());
    }
    Some(metadata)
}

/// Builds the image object consumed by the storefront (`url`, `srcset`,
/// dimensions, alt text and AVIF sources).
pub fn image_resource(path: &str, alt: &str) -> Value {
    let url = format!("/storage/{path}");
    let Some(metadata) = metadata_for(path) else {
        return json!({ "url": url, "alt": alt });
    };
    let variants = metadata["variants"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut fallback_srcset = Vec::new();
    let mut avif_srcset = Vec::new();
    for variant in &variants {
        if let (Some(src), Some(width)) = (variant["src"].as_str(), variant["width"].as_i64()) {
            fallback_srcset.push(format!("/storage/{src} {width}w"));
        }
        if let (Some(src), Some(width)) = (variant["avif"].as_str(), variant["width"].as_i64()) {
            avif_srcset.push(format!("/storage/{src} {width}w"));
        }
    }
    let fallback_srcset = fallback_srcset.join(", ");
    let avif_srcset = avif_srcset.join(", ");
    json!({
        "url": url,
        "alt": alt,
        "width": metadata["width"],
        "height": metadata["height"],
        "mime": metadata["mime"],
        "srcset": if fallback_srcset.is_empty() { Value::Null } else { json!(fallback_srcset) },
        "avif_srcset": if avif_srcset.is_empty() { Value::Null } else { json!(avif_srcset) },
        "variants": variants
    })
}
