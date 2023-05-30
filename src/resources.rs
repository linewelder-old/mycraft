use std::path::Path;

use anyhow::Result;
use image::DynamicImage;

use crate::{context::Context, rendering::texture::Texture};

pub struct Resources {
    pub blocks_texture: Texture,
    pub sky_texture: Texture,
}

fn load_image(path: &Path, name: &str) -> Result<DynamicImage> {
    use anyhow::Context;

    let full_name = path.join("textures").join(name);
    let data =
        std::fs::read(full_name).with_context(|| format!("Failed to load image {}", name))?;
    let image = image::load_from_memory(&data)
        .with_context(|| format!("Failed to decode image {}", name))?;
    Ok(image)
}

fn load_texture(context: &Context, path: &Path, name: &str) -> Result<Texture> {
    let image = load_image(path, name)?;
    Ok(Texture::new(context, name, image))
}

impl Resources {
    pub fn try_load(context: &Context, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        Ok(Resources {
            blocks_texture: load_texture(context, path, "blocks.png")?,
            sky_texture: load_texture(context, path, "sky.png")?,
        })
    }
}
