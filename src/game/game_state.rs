#![macro_use]

use std::ops::{Index, IndexMut};
use std::panic::AssertUnwindSafe;
use std::time::Duration;
use std::mem::MaybeUninit;

use bumpalo::Bump;
use cgmath::Relative;
use log::debug;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::timer::{AverageDurationTimer, TargetTimer};
use crate::timer::measure;

use super::tiles::TileType;
use super::position::{WorldPosition, TileOffset, TileCoordinate};
use super::trees::{Tree, TreeGrowthStage, TreeSpecies};
use super::tree_region_iterator::{TreeRegionIterator, TreeRegionIteratorMut};

pub const GRID_DIM: usize = 30;
pub const GRID_SIZE: usize = GRID_DIM * GRID_DIM;

pub const TILE_DIM: f32 = 1.0;
pub const TILE_RAD: f32 = TILE_DIM * 0.5;

pub const NUM_TREES_PER_TILE: usize = 10;
const MAX_NUM_TREES: usize = GRID_SIZE * NUM_TREES_PER_TILE;

macro_rules! tile_index {
    ($x:expr, $y:expr) => {
        $x as usize + ($y as usize * GRID_DIM)
    }
}

macro_rules! tree_slot_index {
    ($tile:expr, $t:expr) => {
        ($tile * NUM_TREES_PER_TILE) + $t
    }
}

macro_rules! tree_slot_index_xyt {
    ($x:expr, $y:expr, $t:expr) => {

        tree_slot_index!(tile_index!($x, $y), $t)
    }
}

pub(crate) use tile_index;
pub(crate) use tree_slot_index;
pub(crate) use tree_slot_index_xyt;

fn smoothstep(min: f32, max: f32, mut x: f32) -> f32 {
    x = f32::clamp((x - min) / (max - min), 0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

pub struct GameCamera {
    pub position: cgmath::Point3<f32>,
    pub zoom_level: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GroundCover {
    Grass,
    Dirt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoilType {
    Stony,
    Normal,
}

pub struct DebugFlags {
    pub show_grid: bool,
    pub show_dual: bool,
    pub show_trees: bool,
}

pub struct GameState {
    pub camera: GameCamera,
    pub tiles: [(GroundCover, SoilType); GRID_SIZE],
    pub tile_light_amt: [f32; GRID_SIZE],

    pub count_trees: usize,
    pub per_tile_tree_count: [u8; GRID_SIZE],
    pub trees: [Option<Tree>; MAX_NUM_TREES],

    paused: bool,
    pub debug: DebugFlags,

    //Timers...
    debug_log_timer: TargetTimer,
    perf_timer: AverageDurationTimer<20>,

    rng: ThreadRng,
    speed: f32,
    zoom_factor: f32,
    pub one_sec_sin: f32,
}

impl GameState {
    pub fn new() -> Self {
        let rng = rand::thread_rng();

        let camera = {
            let x = (GRID_DIM as f32 * TILE_DIM) * 0.5;
            let y = (GRID_DIM as f32 * TILE_DIM) * 0.5;

            GameCamera {
                position: cgmath::Point3::new(x, y, -0.5),
                zoom_level: 20.0,
            }
        };

        let mut result = Self {
            tiles: [(GroundCover::Grass, SoilType::Normal); GRID_SIZE],
            tile_light_amt: [0.0; GRID_SIZE],
            count_trees: 0,
            per_tile_tree_count: [0; GRID_SIZE],
            trees: [None; MAX_NUM_TREES],

            paused: false,
            debug: DebugFlags {
                show_dual: false,
                show_grid: false,
                show_trees: true,
            },

            debug_log_timer: TargetTimer::new(Duration::from_secs(1)),
            perf_timer: AverageDurationTimer::new(),

            rng,
            speed: 0.005,
            zoom_factor: 0.01, // percent of current zoom level
            one_sec_sin: 0.0,
            camera,
        };

        for (index, tile) in result.tiles.iter_mut().enumerate() {
            if
                index / GRID_DIM == 0 ||
                index % GRID_DIM == 0 ||
                index / GRID_DIM == GRID_DIM -1 ||
                index % GRID_DIM == GRID_DIM -1
            {
                tile.0 = GroundCover::Grass;
            }
        }


        for tile in result.tiles.iter_mut().take(GRID_SIZE / 2) {
            tile.1 = SoilType::Stony;
        }

        // const NUM_INITIAL_TREES: usize = 30;
        // let mut plant_locations = Vec::with_capacity(NUM_INITIAL_TREES);
        // for _ in 0..NUM_INITIAL_TREES {
        //     plant_locations.push(
        //         WorldPosition {
        //             coord: TileCoordinate {
        //                 x: result.rng.gen_range(0..GRID_DIM) as i32,
        //                 y: result.rng.gen_range(0..GRID_DIM) as i32,
        //             },
        //             offset: TileOffset {
        //                 x: result.rng.gen_range(0.0..1.0),
        //                 y: result.rng.gen_range(0.0..1.0),
        //             },
        //         }
        //     );
        // }

        const NUM_INITIAL_TREES: usize = 10;
        let mut plant_locations = Vec::with_capacity(NUM_INITIAL_TREES * 2);
        for _ in 0..NUM_INITIAL_TREES {
            plant_locations.push(
                WorldPosition {
                    coord: TileCoordinate {
                        x: result.rng.gen_range(0..GRID_DIM) as i32,
                        y: (GRID_DIM / 3) as i32,
                    },
                    offset: TileOffset {
                        x: result.rng.gen_range(0.0..1.0),
                        y: result.rng.gen_range(0.0..1.0),
                    },
                }
            );

            plant_locations.push(
                WorldPosition {
                    coord: TileCoordinate {
                        x: result.rng.gen_range(0..GRID_DIM) as i32,
                        y: ((GRID_DIM / 3) * 2) as i32,
                    },
                    offset: TileOffset {
                        x: result.rng.gen_range(0.0..1.0),
                        y: result.rng.gen_range(0.0..1.0),
                    },
                }
            );
        }

        let species = [
            TreeSpecies::Ash,
            TreeSpecies::Fir,
            TreeSpecies::CottonWood,
        ];

        for pos in plant_locations {
            let index = result.rng.gen_range(0..species.len());
            result.plant_tree(pos, species[index]);
            // result.plant_tree(pos, TreeSpecies::Ash);
        }

        result
    }

    pub unsafe fn iter_trees_on_tile_unchecked_mut<'s, 't>(&'s mut self, tile_index: usize) -> impl Iterator<Item=&'t mut Tree>
    where
        's: 't
    {
        // SAFETY:
        //  tile_index assumed to be in bounds.
        let num_trees_on_tile = (*self.per_tile_tree_count.get_unchecked(tile_index)) as usize;

        let begin = tree_slot_index!(tile_index, 0);
        let end = begin + num_trees_on_tile;

        debug_assert!(end <= MAX_NUM_TREES);

        // SAFETY:
        //  begin is usize, cannot be < 0
        //  end is <= MAX_NUM_TREES
        let slice = self.trees.get_unchecked_mut(begin..end);

        // SAEFTY:
        //  Trees are packed in the front of the sub-array for each tile.
        let result = slice.iter_mut().map(|t| t.as_mut().unwrap());

        result
    }

    pub unsafe fn iter_trees_on_tile_unchecked<'s, 't>(&'s self, tile_index: usize) -> impl Iterator<Item=&'t Tree>
    where
        's: 't
    {
        // SAFETY:
        //  tile_index assumed to be in bounds.
        let num_trees_on_tile = (*self.per_tile_tree_count.get_unchecked(tile_index)) as usize;

        let begin = tree_slot_index!(tile_index, 0);
        let end = begin + num_trees_on_tile;

        debug_assert!(end <= MAX_NUM_TREES);

        // SAFETY:
        //  tile_index assumed to be in bounds.
        //  self.num_trees_on_tile assumed to be accurate.
        let slice = self.trees.get_unchecked(begin..end);

        // SAEFTY:
        //  Trees are packed in the front of the sub-array for each tile.
        let result = slice.iter().map(|t| t.as_ref().unwrap());

        result
    }


    pub fn iter_trees_in_radius<'s, 't>(&'s self, pos: WorldPosition, radius: f32) -> impl Iterator<Item=(usize, &'t Tree)>
    where
        's: 't
    {
        let radius_offset = TileOffset { x: radius, y: radius };

        let min = pos - radius_offset;
        let min_x = i32::max(min.coord.x, 0) as usize;
        let min_y = i32::max(min.coord.y, 0) as usize;

        let max = pos + radius_offset;
        let max_x = i32::min(max.coord.x, GRID_DIM as i32 - 1) as usize;
        let max_y = i32::min(max.coord.y, GRID_DIM as i32 - 1) as usize;

        // SAEFTY:
        //  Trees min, max have jsut been clamped against bounds
        unsafe {
            TreeRegionIterator::new((min_x, min_y), (max_x, max_y), &self.trees)
                .filter(move |t| t.1.position.distance_sq(&pos) <= (radius * radius))
        }
    }

    pub fn iter_trees_in_radius_mut<'s, 't>(&'s mut self, pos: WorldPosition, radius: f32) -> impl Iterator<Item=(usize, &'t mut Tree)>
    where
        's: 't
    {
        let radius_offset = TileOffset { x: radius, y: radius };

        let min = pos - radius_offset;
        let min_x = i32::max(min.coord.x, 0) as usize;
        let min_y = i32::max(min.coord.y, 0) as usize;

        let max = pos + radius_offset;
        let max_x = i32::min(max.coord.x, GRID_DIM as i32 - 1) as usize;
        let max_y = i32::min(max.coord.y, GRID_DIM as i32 - 1) as usize;

        // SAEFTY:
        //  Trees min, max have jsut been clamped against bounds
        unsafe {
            TreeRegionIteratorMut::new((min_x, min_y), (max_x, max_y), &mut self.trees)
                .filter(move |t| t.1.position.distance_sq(&pos) <= (radius * radius))
        }
    }

    unsafe fn get_tree_slots_on_tile_unchecked_mut(&mut self, tile_index: usize) -> & mut[Option<Tree>] {
        let begin = tree_slot_index!(tile_index, 0);
        let end = tree_slot_index!(tile_index, NUM_TREES_PER_TILE);

        // SAEFTY:
        //  tile_index assumed to be in bounds
        self.trees.get_unchecked_mut(begin..end)
    }

    fn plant_tree(&mut self, pos: WorldPosition, species: TreeSpecies)  {
        let x = pos.coord.x;
        let y = pos.coord.y;

        debug_assert!(x < GRID_DIM as i32);
        debug_assert!(x >= 0);
        debug_assert!(y < GRID_DIM as i32);
        debug_assert!(y >= 0);

        let tile_index = tile_index!(x, y);

        // SAEFTY:
        //  We've just checked that x, y are in bounds
        let num_trees_on_tile = unsafe { *(self.per_tile_tree_count.get_unchecked(tile_index)) as usize };
        if num_trees_on_tile < NUM_TREES_PER_TILE {
            let tree_slot_index = tree_slot_index!(tile_index, num_trees_on_tile);

            // SAFETY:
            //  self.num_trees_on_tile assumed to be accurate.
            let tree_opt = unsafe { self.trees.get_unchecked_mut(tree_slot_index) };

            debug_assert!(tree_opt.is_none());

            *(tree_opt) = Some(Tree::new(species, pos));

            // SAEFTY:
            //  We've just checked the x, y uset to create tile_index
            unsafe { *(self.per_tile_tree_count.get_unchecked_mut(tile_index)) += 1 };
            self.count_trees += 1;

            self.set_shade_from_surrounding_trees(tree_slot_index);
        }
    }

    fn kill_tree(&mut self, tree_slot_index: usize)  {
        debug_assert!(tree_slot_index < MAX_NUM_TREES);
        let tree_stage = self.trees.get(tree_slot_index).unwrap().as_ref().unwrap().stage;

        use TreeGrowthStage::*;
        match tree_stage {
            Sprout | Seedling => unsafe { self.delete_tree(tree_slot_index) },
            Sapling | Mature | Old | Decline => {
                let tree = self.trees.get_mut(tree_slot_index).unwrap().as_mut().unwrap();
                tree.stage = Snag;
                tree.growth_target = tree.growth_required_for_next_stage();
            },
            Snag | Stump => (), // already dead
        }
    }

    /// SAFETY: A group of N calls to this function _MUST_ be followed by a call to pack_trees() otherwise tree iteration invariants are broken. lol
    unsafe fn delete_tree(&mut self, tree_slot_index: usize)  {
        debug_assert!(tree_slot_index < MAX_NUM_TREES);

        let tile_index = tree_slot_index / NUM_TREES_PER_TILE;
        let tree_index = tree_slot_index % NUM_TREES_PER_TILE;

        let count_trees_on_tile = *(self.per_tile_tree_count.get_unchecked(tile_index)) as usize;
        debug_assert!(count_trees_on_tile > 0);

        let tree_slots = self.get_tree_slots_on_tile_unchecked_mut(tile_index);
        *(tree_slots.get_unchecked_mut(tree_index)) = None;
    }

    /// SAFETY: tile_index must be in bounds
    unsafe fn pack_trees(&mut self, tile_index: usize)  {
        let count_trees = *(self.per_tile_tree_count.get_unchecked_mut(tile_index)) as usize;

        let mut read_index = 0;
        let mut write_index = 0;

        let tree_slots = self.get_tree_slots_on_tile_unchecked_mut(tile_index);
        while write_index < count_trees && read_index < NUM_TREES_PER_TILE {
            if read_index != write_index && tree_slots.get(read_index).unwrap().is_some() {
                tree_slots.swap(read_index, write_index);
            }

            read_index += 1;
            if tree_slots.get(write_index).unwrap().is_some() {
                write_index += 1;
            }
        }

        *(self.per_tile_tree_count.get_unchecked_mut(tile_index)) = write_index as u8;
    }

    pub fn set_shade_from_surrounding_trees(&mut self, tree_slot_index: usize) {
        let (tree_pos, tree_stage) = {
            let t_ref = self.trees.get(tree_slot_index).unwrap().as_ref().unwrap();
            (t_ref.position, t_ref.stage)
        };

        let mut shade_factor = 1.0;
        let radius = 1.0; //Something big... No tree is gonna be 2 tiles wide... probably.

        for (near_tree_index, near_tree) in self.iter_trees_in_radius(tree_pos, radius) {
            if near_tree_index == tree_slot_index { continue; } // Skip the tree we're updating.

            let near_tree_shad_rad = near_tree.species.shadow_radius(near_tree.stage);
            let distance = tree_pos.distance(&near_tree.position);

            if near_tree_shad_rad <= 0.0     { continue; } // 0 causes undesirable flipping with smoothstep, negative radius should be impossible.
            if distance <= near_tree_shad_rad {
                shade_factor *= (1.0 - smoothstep(near_tree_shad_rad, 0.0, distance));
            } // Tree is too far to cast shade.
        }

        let t_ref_mut = self.trees.get_mut(tree_slot_index).unwrap().as_mut().unwrap();
        t_ref_mut.shade_factor = shade_factor;
    }

    pub fn update_shade_for_surrounding_trees(&mut self, tree_slot_index: usize, previous_stage: TreeGrowthStage) {
        let t_ref = self.trees.get(tree_slot_index).unwrap().as_ref().unwrap();
        let tree_pos = t_ref.position;
        let tree_species = t_ref.species;
        let tree_stage = t_ref.stage;

        let old_shadow_radius = tree_species.shadow_radius(previous_stage);
        let new_shadow_radius = tree_species.shadow_radius(tree_stage);
        let max_shadow_radius = f32::max(old_shadow_radius, new_shadow_radius);

        for (near_tree_slot, near_tree) in self.iter_trees_in_radius_mut(tree_pos, max_shadow_radius) {
            if near_tree_slot == tree_slot_index { continue; }

            let distance = tree_pos.distance(&near_tree.position);

            // Remove effect of old shadow
            if distance <= old_shadow_radius && old_shadow_radius > 0.0 {
                near_tree.shade_factor /= (1.0 - smoothstep(old_shadow_radius, 0.0, distance));
            }

            // Add effect of new shadow
            if distance <= new_shadow_radius && new_shadow_radius > 0.0 {
                near_tree.shade_factor *= (1.0 - smoothstep(new_shadow_radius, 0.0, distance));
            }
        }
    }

    pub fn update(&mut self, input: &Input) {
        let left_amt  = if input.left  { -self.speed * self.camera.zoom_level } else { 0.0 };
        let right_amt = if input.right {  self.speed * self.camera.zoom_level } else { 0.0 };
        let down_amt  = if input.down  { -self.speed * self.camera.zoom_level } else { 0.0 };
        let up_amt    = if input.up    {  self.speed * self.camera.zoom_level } else { 0.0 };

        self.camera.position.x += left_amt;
        self.camera.position.x += right_amt;
        self.camera.position.y += down_amt;
        self.camera.position.y += up_amt;

        let zoom_dir = if input.zoom_in { -1.0 } else if input.zoom_out { 1.0 } else { 0.0 };
        let zoom_amt = self.zoom_factor * zoom_dir * self.camera.zoom_level;

        self.camera.zoom_level += zoom_amt;
        if self.camera.zoom_level > 30.0 { self.camera.zoom_level = 30.0; }
        if self.camera.zoom_level < 1.5 { self.camera.zoom_level = 1.5; }

        let dt_s = input.dt.as_secs_f32();

        self.debug.show_grid = input.show_grid;
        self.debug.show_dual = input.show_dual;
        self.debug.show_trees = input.show_trees;

        self.paused = input.pause;
        if self.paused { return; }

        self.update_trees(dt_s);

        measure!(self.perf_timer, {
            self.update_grass();
        });

        self.one_sec_sin = f32::sin(dt_s);

        // if let TimerState::Ready(_) = self.debug_log_timer.check() {
        //     self.debug_log_timer.reset();
        //     debug!("{:?}", self.perf_timer.as_ref().unwrap().average());
        // }
    }

    fn update_trees(&mut self, dt_s: f32) {
        const MAX_NUM_EVENTS: usize = GRID_SIZE * NUM_TREES_PER_TILE;

        let mut count_events = 0;
        let mut tree_events = [MaybeUninit::uninit(); MAX_NUM_EVENTS];

        #[derive(Copy, Clone, Debug)]
        enum Event {
            Plant { pos: WorldPosition, species: TreeSpecies },
            Kill { tree_slot_index: usize },
            Delete { tree_slot_index: usize }
        }

        macro_rules! push_event {
            ($e:expr) => {
                if count_events < MAX_NUM_EVENTS {
                    tree_events.get_mut(count_events).unwrap().write($e);
                    count_events += 1;
                }
            };
        }

        let mut tile_index = 0;
        while tile_index < GRID_SIZE {
            // SAFETY:
            //  tile_index ranging from 0..GRID_SIZE, and every tile has a corresponding tree count.
            let num_trees_on_tile = {
                *(unsafe { self.per_tile_tree_count.get_unchecked(tile_index) }) as usize
            };

            // SAFETY:
            //  tile_index ranging from 0..GRID_SIZE.
            let soil_type = unsafe { self.tiles.get_unchecked(tile_index).1 };

            let mut tree_index = 0;
            while tree_index < num_trees_on_tile {
                let slot_index = tree_slot_index!(tile_index, tree_index);

                // SAFETY:
                //  index is constructed by tile_index (see above) incremented by tree_index.
                //  tree_index must be < num_trees_on_tile
                let tree = unsafe { self.trees.get_unchecked_mut(slot_index).as_mut().unwrap_unchecked() };

                let soil_multiplier = if soil_type == tree.species.soil_preference() { 1.0 } else { 0.4 };

                let old_shade_factor = tree.shade_factor;

                //Kill the tree if it's not getting enough oomph.
                let mut growth_multiplier = 1.0;
                if tree.is_alive() {
                    growth_multiplier *= tree.shade_factor * soil_multiplier;
                }

                if growth_multiplier <= 0.05 {
                    push_event!(Event::Kill { tree_slot_index: slot_index });
                    tree_index += 1;
                    continue;
                };

                let old_grow_stage = tree.stage;
                let new_grow_stage = tree.grow(dt_s * growth_multiplier);
                drop(tree);

                if (old_grow_stage != new_grow_stage) {
                    self.update_shade_for_surrounding_trees(slot_index, old_grow_stage);

                    let new_shade_factor = unsafe { self.trees.get_unchecked_mut(slot_index).as_ref().unwrap_unchecked().shade_factor };
                    if old_shade_factor != new_shade_factor {
                        let t_ref = self.trees.get(slot_index).unwrap().as_ref().unwrap();
                        let tree_pos = t_ref.position;
                        let tree_species = t_ref.species;
                        let old_shadow_radius = tree_species.shadow_radius(old_grow_stage);
                        let new_shadow_radius = tree_species.shadow_radius(new_grow_stage);
                        let max_shadow_radius = f32::max(old_shadow_radius, new_shadow_radius);

                        let slots = self.iter_trees_in_radius_mut(tree_pos, max_shadow_radius).map(|(i, _)| i).collect::<Vec<_>>();

                        log::warn!("Tree {slot_index} grew from {old_grow_stage:?} -> {new_grow_stage:?} but it's own shade changed from {old_shade_factor} -> {new_shade_factor}!!");
                        log::warn!("Tree slots near by {slots:?}");
                    }
                }

                // SAFETY:
                //  index is constructed by tile_index (see above) incremented by tree_index.
                //  tree_index must be < num_trees_on_tile
                let tree = unsafe { self.trees.get_unchecked_mut(slot_index).as_mut().unwrap_unchecked() };

                use TreeGrowthStage::*;
                match new_grow_stage {
                    Mature | Old | Decline => {
                        let seed_multiplier = match new_grow_stage {
                            Mature  => 1.0,
                            Old     => 0.5,
                            Decline => 0.2,
                            _ => 0.0,
                        };

                        if tree.seed_timer <= 0.0 {
                            let numerator = 1;
                            let mut denominator = (10.0 * (1.0 / seed_multiplier)) as u32;
                            if soil_type != tree.species.soil_preference() { denominator *= 2; }

                            for _ in 0..3 {
                                if self.rng.gen_ratio(numerator, denominator) {
                                    let (min_r, max_r) = tree.species.seed_radius();

                                    let angle: f32 = {
                                        let deg: f32 = self.rng.gen_range(0.0..360.0);
                                        deg.to_radians()
                                    };
                                    let radius: f32 = self.rng.gen_range(min_r..=max_r);

                                    let x = radius * angle.cos();
                                    let y = radius * angle.sin();

                                    let plant_position = tree.position + TileOffset { x, y };

                                    // Fail to plant if we're oob.
                                    if plant_position.coord.x < GRID_DIM as i32 && plant_position.coord.x >= 0 &&
                                       plant_position.coord.y < GRID_DIM as i32 && plant_position.coord.y >= 0
                                    {
                                        push_event!(
                                            Event::Plant {
                                                pos: plant_position,
                                                species: tree.species,
                                            }
                                        );
                                    }

                                    tree.seed_timer = {
                                        let seed_rate = tree.species.seed_success_rate();
                                        let min = seed_rate.average - seed_rate.variation;
                                        let max = seed_rate.average + seed_rate.variation;

                                        self.rng.gen_range(min..=max)
                                    };
                                }
                            }
                        }
                    },
                    Stump => {
                        if self.rng.gen_ratio(1, 500) {
                            push_event!(Event::Delete { tree_slot_index: slot_index });
                        }
                    },
                    _ => {}
                }

                tree_index += 1;
            }

            tile_index += 1;
        }

        let mut tiles_to_repack = std::collections::HashSet::<usize>::new();

        for index in 0..count_events {
            // SAFETY:
            //  count_events was used to write the values, now upper bound for iteration.
            let event = unsafe { tree_events.get_unchecked(index).assume_init() };

            match event {
                Event::Plant { pos, species } => self.plant_tree(pos, species),
                Event::Kill { tree_slot_index } => {
                    // Not strictly necessary, but we don't know if kill_tree() is going to delete a tree.
                    tiles_to_repack.insert(tree_slot_index / NUM_TREES_PER_TILE);
                    self.kill_tree(tree_slot_index);
                },
                Event::Delete { tree_slot_index } => {
                    tiles_to_repack.insert(tree_slot_index / NUM_TREES_PER_TILE);
                    // SAFETY:
                    //  tree_slot_index comes directly from iteration index when updating trees above.
                    unsafe { self.delete_tree(tree_slot_index) }
                },
            }
        }

        for tile_index in tiles_to_repack {
            // SAFETY:
            //  tile_index was created from tree_slot_index in previous loop.
            //  tree_slot_index comes directly from iteration index when updating trees above.
            unsafe { self.pack_trees(tile_index) };
        }
    }

    fn update_grass(&mut self) {
        let mut new_grass_state: [(GroundCover, SoilType); GRID_SIZE] = self.tiles;

        for x in 0..(GRID_DIM as i32) {
            for y in 0..(GRID_DIM as i32) {
                let tile_index = tile_index!(x, y);
                let mut grassy_neighbor_count = 0;

                // SAFETY:
                //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                let light_amt = unsafe { self.iter_trees_on_tile_unchecked(tile_index) }
                    .fold(1.0, |mut acc, tree| {
                        match tree.stage {
                            TreeGrowthStage::Seedling => acc *= (1.0 - 0.01),
                            TreeGrowthStage::Sapling => acc *= (1.0 - 0.1),
                            TreeGrowthStage::Mature | TreeGrowthStage::Old => acc *= (1.0 - 0.5),
                            _ => {},
                        }

                        acc
                    });

                // SAFETY:
                //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                unsafe {
                    let tile_light = self.tile_light_amt.get_unchecked_mut(tile_index);
                    *tile_light = light_amt;
                }

                if light_amt <= 0.25 {
                    // SAFETY:
                    //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                    unsafe {
                        new_grass_state.get_unchecked_mut(tile_index).0 = GroundCover::Dirt;
                    }
                } else {
                    // SAFETY:
                    //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                    if let (GroundCover::Dirt, _) = unsafe { self.tiles.get_unchecked(tile_index) } {
                        for neighbor_x in (x-1)..=(x+1) {
                            for neighbor_y in (y-1)..=(y+1) {
                                if neighbor_x == neighbor_y { continue; }

                                if
                                    (neighbor_x >= 0) && (neighbor_x < GRID_DIM as i32) &&
                                    (neighbor_y >= 0) && (neighbor_y < GRID_DIM as i32)
                                {
                                    let neighbor_index = tile_index!(neighbor_x, neighbor_y);
                                    if let (GroundCover::Grass, _) = self.tiles.get(neighbor_index).unwrap() {
                                        grassy_neighbor_count += 1;
                                    }
                                }
                            }
                        }

                        let growth_chance = match grassy_neighbor_count {
                            1     => 0.00001,
                            2     => 0.00005,
                            3..=5 => 0.0001,
                            6..=8 => 0.0004,
                            _ => 0.0,
                        };

                        let grow_roll = self.rng.gen_range(0.0..=1.0);
                        if grow_roll > (1.0 - growth_chance) {
                            // SAFETY:
                            //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                            unsafe {
                                new_grass_state.get_unchecked_mut(tile_index).0 = GroundCover::Grass;
                            }
                        }
                    }
                }

            }
        }

        self.tiles = new_grass_state;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Input {
    pub t: std::time::Duration,
    pub dt: std::time::Duration,

    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    pub zoom_in: bool,
    pub zoom_out: bool,

    pub pause: bool,
    pub show_grid: bool,
    pub show_dual: bool,
    pub show_trees: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            t: Default::default(),
            dt: Default::default(),
            up: Default::default(),
            down: Default::default(),
            left: Default::default(),
            right: Default::default(),
            zoom_in: Default::default(),
            zoom_out: Default::default(),
            pause: Default::default(),

            show_grid: false,
            show_dual: false,
            show_trees: true,
        }
    }
}
