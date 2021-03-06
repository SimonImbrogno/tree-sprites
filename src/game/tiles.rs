use crate::render::{SpriteId, SpriteSheetLayout, SpriteSheetEntry, SpriteSetIdentifier};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TileType {
    None,
    TestPattern,
    DirtBrick,
    Dirt,
    GridLine,
    TreeShadow,

    Grass,
    GrassTBR,
    GrassTBL,
    GrassBTL,
    GrassBTR,
    GrassT,
    GrassB,
    GrassL,
    GrassR,
    GrassBR,
    GrassBL,
    GrassTR,
    GrassTL,
    GrassDiagUp,
    GrassDiagDown,

    Stone,
    StoneTBR,
    StoneTBL,
    StoneBTL,
    StoneBTR,
    StoneT,
    StoneB,
    StoneL,
    StoneR,
    StoneBR,
    StoneBL,
    StoneTR,
    StoneTL,
    StoneDiagUp,
    StoneDiagDown,

    AshTreeSprout,
    AshTreeSeedling,
    AshTreeSapling,
    AshTreeMature,
    AshTreeOld,
    AshTreeDecline,
    AshTreeSnag,
    AshTreeStump,

    PineTreeSprout,
    PineTreeSeedling,
    PineTreeSapling,
    PineTreeMature,
    PineTreeOld,
    PineTreeDecline,
    PineTreeSnag,
    PineTreeStump,

    CottonWoodTreeSprout,
    CottonWoodTreeSeedling,
    CottonWoodTreeSapling,
    CottonWoodTreeMature,
    CottonWoodTreeOld,
    CottonWoodTreeDecline,
    CottonWoodTreeSnag,
    CottonWoodTreeStump,
}

impl Default for TileType {
    fn default() -> Self {
        Self::None
    }
}

impl From<TileType> for SpriteId {
    fn from(src: TileType) -> Self {
        SpriteId(src as usize)
    }
}

// SAFETY: TileType is an enum, fool!
unsafe impl SpriteSetIdentifier for TileType {}

macro_rules! sprite {
    ($x:expr, $y:expr, $t:expr) => {
        SpriteSheetEntry { id: $t, pos: ($x, $y) }
    };
}

pub fn get_sprite_sheet_layout() -> SpriteSheetLayout<TileType> {
    use TileType::*;

    SpriteSheetLayout {
        label: "main".into(),
        tile_dim: (32, 32),
        entries: vec![
            sprite!(0, 0, None),
            sprite!(1, 0, TestPattern),
            sprite!(3, 0, DirtBrick),
            sprite!(4, 0, GridLine),
            sprite!(6, 0, TreeShadow),

            sprite!(2, 0, Dirt),

            sprite!(2, 6, Grass),
            sprite!(0, 4, GrassTBL),
            sprite!(1, 4, GrassTBR),
            sprite!(0, 5, GrassBTL),
            sprite!(1, 5, GrassBTR),
            sprite!(2, 4, GrassT),
            sprite!(2, 5, GrassB),
            sprite!(0, 6, GrassL),
            sprite!(1, 6, GrassR),
            sprite!(3, 4, GrassBR),
            sprite!(4, 4, GrassBL),
            sprite!(3, 5, GrassTR),
            sprite!(4, 5, GrassTL),
            sprite!(3, 6, GrassDiagDown),
            sprite!(4, 6, GrassDiagUp),

            sprite!(2, 9, Stone),
            sprite!(0, 7, StoneTBL),
            sprite!(1, 7, StoneTBR),
            sprite!(0, 8, StoneBTL),
            sprite!(1, 8, StoneBTR),
            sprite!(2, 7, StoneT),
            sprite!(2, 8, StoneB),
            sprite!(0, 9, StoneL),
            sprite!(1, 9, StoneR),
            sprite!(3, 7, StoneBR),
            sprite!(4, 7, StoneBL),
            sprite!(3, 8, StoneTR),
            sprite!(4, 8, StoneTL),
            sprite!(3, 9, StoneDiagDown),
            sprite!(4, 9, StoneDiagUp),

            sprite!(0, 1, AshTreeSprout),
            sprite!(1, 1, AshTreeSeedling),
            sprite!(2, 1, AshTreeSapling),
            sprite!(3, 1, AshTreeMature),
            sprite!(4, 1, AshTreeOld),
            sprite!(5, 1, AshTreeDecline),
            sprite!(6, 1, AshTreeSnag),
            sprite!(7, 1, AshTreeStump),

            sprite!(0, 2, PineTreeSprout),
            sprite!(1, 2, PineTreeSeedling),
            sprite!(2, 2, PineTreeSapling),
            sprite!(3, 2, PineTreeMature),
            sprite!(4, 2, PineTreeOld),
            sprite!(5, 2, PineTreeDecline),
            sprite!(6, 2, PineTreeSnag),
            sprite!(7, 2, PineTreeStump),

            sprite!(0, 3, CottonWoodTreeSprout),
            sprite!(1, 3, CottonWoodTreeSeedling),
            sprite!(2, 3, CottonWoodTreeSapling),
            sprite!(3, 3, CottonWoodTreeMature),
            sprite!(4, 3, CottonWoodTreeOld),
            sprite!(5, 3, CottonWoodTreeDecline),
            sprite!(6, 3, CottonWoodTreeSnag),
            sprite!(7, 3, CottonWoodTreeStump),
        ],
    }
}
