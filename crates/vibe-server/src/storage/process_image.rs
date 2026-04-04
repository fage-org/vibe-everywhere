use image::{GenericImageView, ImageFormat, ImageReader, imageops::FilterType};
use thiserror::Error;

use crate::storage::thumbhash::compute_thumbhash;

#[derive(Debug, Error)]
pub enum ProcessImageError {
    #[error("Unsupported image input")]
    Decode,
    #[error("Unsupported image format")]
    UnsupportedFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedImage {
    pub width: u32,
    pub height: u32,
    pub thumbhash: String,
    pub format_extension: &'static str,
}

pub fn process_image(bytes: &[u8]) -> Result<ProcessedImage, ProcessImageError> {
    let reader = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| ProcessImageError::Decode)?;
    let format = reader.format().ok_or(ProcessImageError::Decode)?;
    let format_extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        _ => return Err(ProcessImageError::UnsupportedFormat),
    };

    let decoded = reader.decode().map_err(|_| ProcessImageError::Decode)?;
    let (width, height) = decoded.dimensions();

    let (target_width, target_height) = if width > height {
        (
            100,
            ((height as f32 * 100.0) / width as f32).round().max(1.0) as u32,
        )
    } else if height > width {
        (
            (((width as f32 * 100.0) / height as f32).round().max(1.0) as u32),
            100,
        )
    } else {
        (100, 100)
    };
    let thumb = decoded.resize(target_width, target_height, FilterType::Triangle);
    let rgba = thumb.to_rgba8();

    Ok(ProcessedImage {
        width,
        height,
        thumbhash: compute_thumbhash(rgba.width(), rgba.height(), rgba.as_raw()),
        format_extension,
    })
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, ImageFormat, Rgba};

    use super::{ProcessImageError, process_image};

    #[test]
    fn process_image_extracts_dimensions_and_thumbhash() {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        let processed = process_image(&cursor.into_inner()).unwrap();
        assert_eq!(processed.width, 1);
        assert_eq!(processed.height, 1);
        assert_eq!(processed.format_extension, "png");
        assert!(!processed.thumbhash.is_empty());
    }

    #[test]
    fn process_image_rejects_non_image_input() {
        let error = process_image(b"not-an-image").unwrap_err();
        assert!(matches!(error, ProcessImageError::Decode));
    }
}
