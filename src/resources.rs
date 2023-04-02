use std::path::Path;

use anyhow::{Result, Context};
use image::DynamicImage;

pub struct Resources {
    pub blocks_image: DynamicImage,
}

fn load_image(path: &Path, name: &str) -> Result<DynamicImage> {
    let full_name = path.join("textures").join(name);
    let data = std::fs::read(full_name)
        .with_context(|| format!("Failed to load image {}", name))?;
    let image = image::load_from_memory(&data)
        .with_context(|| format!("Failed to decode image {}", name))?;
    Ok(image)
}

impl Resources {
    pub fn try_load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        Ok(Resources {
            blocks_image: load_image(path, "blocks.png")?,
        })  
    }
}
