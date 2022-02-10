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
}

impl TreeSpecies {
    pub fn seed_radius(&self) -> (f32, f32) {
        match self {
            Self::Ash => (self.crowd_radius(), 3.0),
            Self::Fir => (self.crowd_radius(), 1.0),
        }
    }

    pub fn crowd_radius(&self) -> f32 {
        match self {
            Self::Ash => 0.45,
            Self::Fir => 0.3,
        }
    }

    pub fn seed_rate(&self) -> SeedRate {
        match self {
            Self::Ash => SeedRate { average: 4.0, variation: 1.0},
            Self::Fir => SeedRate { average: 3.0, variation: 1.0},
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    // pub seed_fatigue: f32,
    pub seed_timer: f32,
    pub to_delete: bool,
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
            to_delete: false,
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

            (_, Stump)    => None,
        }
    }
}
