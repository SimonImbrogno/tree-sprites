use super::game_state::SoilType;
use super::tiles::TileType;
use super::position::WorldPosition;

#[derive(Clone, Copy, Debug)]
pub struct SeedRate {
    pub average: f32,
    pub variation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeSpecies {
    Ash,
    Fir,
    CottonWood,
}

impl TreeSpecies {
    pub fn seed_radius(&self) -> (f32, f32) {
        match self {
            Self::Ash        => (0.4, 4.5),
            Self::Fir        => (0.3, 1.5),
            Self::CottonWood => (0.6, 6.0),
        }
    }

    pub fn seed_success_rate(&self) -> SeedRate {
        match self {
            Self::Ash        => SeedRate { average: 4.0, variation: 1.0},
            Self::Fir        => SeedRate { average: 3.0, variation: 1.0},
            Self::CottonWood => SeedRate { average: 16.0, variation: 3.0},
        }
    }

    pub fn soil_preference(&self) -> SoilType {
        match self {
            Self::Ash        => SoilType::Normal,
            Self::Fir        => SoilType::Stony,
            Self::CottonWood => SoilType::Normal,
        }
    }

    pub fn shadow_radius(&self, growth_stage: TreeGrowthStage) -> f32 {
        match self {
            Self::Ash => match growth_stage {
                TreeGrowthStage::Sapling => 0.55,
                TreeGrowthStage::Mature => 0.9,
                TreeGrowthStage::Old => 0.8,
                TreeGrowthStage::Decline => 0.55,
                _ => 0.0,
            },
            Self::Fir => match growth_stage {
                TreeGrowthStage::Seedling => 0.25,
                TreeGrowthStage::Sapling => 0.55,
                TreeGrowthStage::Mature => 0.7,
                TreeGrowthStage::Old => 0.65,
                TreeGrowthStage::Decline => 0.55,
                _ => 0.0,
            },
            Self::CottonWood => match growth_stage {
                TreeGrowthStage::Seedling => 0.25,
                TreeGrowthStage::Sapling => 0.7,
                TreeGrowthStage::Mature => 0.9,
                TreeGrowthStage::Old => 0.9,
                TreeGrowthStage::Decline => 0.8,
                _ => 0.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum TreeGrowthStage {
    Sprout,
    Seedling,
    Sapling,
    Mature,
    Old,
    Decline,
    Snag,
    Stump,
}

impl TreeGrowthStage {
    pub fn next(&self) -> Self {
        match self {
            Self::Sprout   => Self::Seedling,
            Self::Seedling => Self::Sapling,
            Self::Sapling  => Self::Mature,
            Self::Mature   => Self::Old,
            Self::Old      => Self::Decline,
            Self::Decline  => Self::Snag,
            Self::Snag     => Self::Stump,
            Self::Stump    => Self::Stump,
        }
    }
}

impl From<&Tree> for TileType {
    fn from(src: &Tree) -> Self {
        use TreeGrowthStage::*;
        use TreeSpecies::*;
        match (src.species, src.stage) {
            (Ash, Sprout)   => Self::AshTreeSprout,
            (Ash, Seedling) => Self::AshTreeSeedling,
            (Ash, Sapling)  => Self::AshTreeSapling,
            (Ash, Mature)   => Self::AshTreeMature,
            (Ash, Old)      => Self::AshTreeOld,
            (Ash, Decline)  => Self::AshTreeDecline,
            (Ash, Snag)     => Self::AshTreeSnag,
            (Ash, Stump)    => Self::AshTreeStump,

            (Fir, Sprout)   => Self::PineTreeSprout,
            (Fir, Seedling) => Self::PineTreeSeedling,
            (Fir, Sapling)  => Self::PineTreeSapling,
            (Fir, Mature)   => Self::PineTreeMature,
            (Fir, Old)      => Self::PineTreeOld,
            (Fir, Decline)  => Self::PineTreeDecline,
            (Fir, Snag)     => Self::PineTreeSnag,
            (Fir, Stump)    => Self::PineTreeStump,

            (CottonWood, Sprout)   => Self::CottonWoodTreeSprout,
            (CottonWood, Seedling) => Self::CottonWoodTreeSeedling,
            (CottonWood, Sapling)  => Self::CottonWoodTreeSapling,
            (CottonWood, Mature)   => Self::CottonWoodTreeMature,
            (CottonWood, Old)      => Self::CottonWoodTreeOld,
            (CottonWood, Decline)  => Self::CottonWoodTreeDecline,
            (CottonWood, Snag)     => Self::CottonWoodTreeSnag,
            (CottonWood, Stump)    => Self::CottonWoodTreeStump,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Tree {
    pub position: WorldPosition,

    pub species: TreeSpecies,
    pub stage: TreeGrowthStage,

    pub growth: f32,
    pub base_growth_speed: f32,
    pub growth_target: Option<f32>,

    pub seed_timer: f32,
    pub shade_factor: f32,
}

impl Tree {
    pub fn new(species: TreeSpecies, position: WorldPosition) -> Self {
        use TreeGrowthStage::*;

        let mut result = Self {
            position,

            species,
            stage: Sprout,

            growth: 0.0,
            base_growth_speed: 1.0,
            growth_target: None,

            seed_timer: 1.0,

            shade_factor: 1.0,
        };

        result.growth_target = result.growth_required_for_next_stage();
        result
    }

    pub fn grow(&mut self, dt_s: f32) -> TreeGrowthStage {
        if let Some(target) = self.growth_target {
            let growth_amt = self.base_growth_speed * dt_s;
            self.growth += growth_amt;

            if self.growth > target {
                self.stage = self.stage.next();
                self.growth_target = {
                    if let Some(new_required_growth) = self.growth_required_for_next_stage() {
                        Some(target + new_required_growth)
                    } else {
                        None
                    }
                };
            }
        }

        use TreeGrowthStage::*;
        if let Mature | Old | Decline = self.stage {
            self.seed_timer -= dt_s;
        }

        self.stage
    }

    pub fn growth_required_for_next_stage(&self) -> Option<f32> {
        use TreeGrowthStage::*;
        use TreeSpecies::*;

        match (self.species, self.stage) {
            (Ash, Sprout)   => Some(1.0),
            (Ash, Seedling) => Some(5.0),
            (Ash, Sapling)  => Some(20.0),
            (Ash, Mature)   => Some(45.0),
            (Ash, Old)      => Some(40.0),
            (Ash, Decline)  => Some(20.0),
            (Ash, Snag)     => Some(10.0),

            (Fir, Sprout)   => Some(5.0),
            (Fir, Seedling) => Some(15.0),
            (Fir, Sapling)  => Some(20.0),
            (Fir, Mature)   => Some(60.0),
            (Fir, Old)      => Some(60.0),
            (Fir, Decline)  => Some(20.0),
            (Fir, Snag)     => Some(10.0),

            (CottonWood, Sprout)   => Some(2.0),
            (CottonWood, Seedling) => Some(3.0),
            (CottonWood, Sapling)  => Some(10.0),
            (CottonWood, Mature)   => Some(80.0),
            (CottonWood, Old)      => Some(75.0),
            (CottonWood, Decline)  => Some(30.0),
            (CottonWood, Snag)     => Some(15.0),

            (_, Stump)    => None,
        }
    }

    // A little counter intuitive "dead, decaying" trees still "grow", they're just unaffected by modifiers.
    pub fn is_alive(&self) -> bool {
        self.stage != TreeGrowthStage::Snag && self.stage != TreeGrowthStage::Stump
    }
}
