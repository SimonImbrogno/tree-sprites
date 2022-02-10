use std::collections::HashMap;

use anyhow::Result;

use super::texture::Texture;

// SAFETY:
//  All values returned from Into<SpriteId> MUST be >= 0 and < std::mem::variant_count::<T>()
//  Only implement this on enums you fool!
pub unsafe trait SpriteSetIdentifier: Into<SpriteId> + Copy {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpriteId(pub usize);

pub struct SpriteSheetLayout<T>
where
    T: SpriteSetIdentifier,
    [(); std::mem::variant_count::<T>()]: Sized,
{
    pub label: String,
    pub tile_dim: (usize, usize),
    pub entries: Vec<SpriteSheetEntry<T>>,
}

pub struct SpriteSheetEntry<T>
where
    T: SpriteSetIdentifier,
    [(); std::mem::variant_count::<T>()]: Sized,
{
    pub id: T,
    pub pos: (usize, usize),
}

pub struct SpriteSheet<T>
where
    T: SpriteSetIdentifier,
    [(); std::mem::variant_count::<T>()]: Sized,
{
    index_map: [usize; std::mem::variant_count::<T>()],
    pub texture: Texture,
}

impl<T> SpriteSheet<T>
where
    T: SpriteSetIdentifier,
    [(); std::mem::variant_count::<T>()]: Sized,
{
    pub fn try_load_from_bytes(bytes: &[u8], layout: &SpriteSheetLayout<T>, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let mut sprite_sheet = image::load_from_memory(bytes)?;

        let mut images = Vec::new();
        let mut index_map = [0; std::mem::variant_count::<T>()];

        for (index, entry) in layout.entries.iter().enumerate() {
            let min_x = (entry.pos.0 * layout.tile_dim.0) as u32;
            let min_y = (entry.pos.1 * layout.tile_dim.1) as u32;
            let dim_x = layout.tile_dim.0 as u32;
            let dim_y = layout.tile_dim.1 as u32;

            let sprite = sprite_sheet.crop(min_x, min_y, dim_x, dim_y);
            images.push(sprite);

            let id: SpriteId = entry.id.into();
            *index_map.get_mut(id.0).unwrap() = index;
        }

        let texture = Texture::try_create_array_texture_from_images(&images, device, queue)?;
        Ok(SpriteSheet { index_map, texture })
    }

    pub fn get_texture_index(&self, identifier: T) -> usize {
        let id: SpriteId = identifier.into();
        *self.index_map.get(id.0).unwrap_or(&0)
    }
}
