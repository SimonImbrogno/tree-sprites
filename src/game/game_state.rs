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

use super::tiles::TileType;
use super::position::{WorldPosition, TileOffset, TileCoordinate};
use super::trees::{Tree, TreeGrowthStage, TreeSpecies};

pub const GRID_DIM: usize = 16;
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

pub struct GameCamera {
    pub position: cgmath::Point3<f32>,
    pub zoom_level: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GroundCover {
    Grass,
    Dirt,
}

struct TreeRegionIterator<'t> {
    min_x:  usize,
    min_y:  usize,

    max_x:  usize,
    max_y:  usize,

    curr_x: usize,
    curr_y: usize,

    tree_sub_index: usize,

    trees: &'t [Option<Tree>],
}

impl <'t> TreeRegionIterator<'t> {
    pub unsafe fn new(min: (usize, usize), max: (usize, usize), trees: &'t [Option<Tree>]) -> Self {
        debug_assert!(max.0 < GRID_DIM);
        debug_assert!(max.1 < GRID_DIM);
        debug_assert!(min.0 <= max.0);
        debug_assert!(min.1 <= max.1);

        Self {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
            curr_x: min.0,
            curr_y: min.1,
            tree_sub_index: 0,
            trees,
        }
    }
}

impl<'t> Iterator for TreeRegionIterator<'t> {
    type Item = &'t Tree;

    fn next(&mut self) -> Option<Self::Item> {

        while self.curr_y <= self.max_y {
            // SAFETY:
            //  curr_x, curr_y are both in range 0..GRID_DIM, tree_sub_index is reset when >= NUM_TREES_PER_TILE
            let slot_index = tree_slot_index_xyt!(self.curr_x, self.curr_y, self.tree_sub_index);
            let result = unsafe { self.trees.get_unchecked(slot_index) };

            self.tree_sub_index += 1;

            if self.tree_sub_index >= NUM_TREES_PER_TILE {
                self.tree_sub_index = 0;

                if self.curr_x == self.max_x {
                    self.curr_x = self.min_x;
                    self.curr_y += 1;
                } else {
                    self.curr_x += 1;
                }
            }

            if result.is_some() {
                return result.as_ref();
            }
        }

        None
    }
}

pub struct DebugFlags {
    pub show_grid: bool,
    pub show_trees: bool,
    pub highlight_impending_seed: bool,
}

pub struct GameState {
    pub camera: GameCamera,
    pub tiles: [GroundCover; GRID_SIZE],
    pub tile_light_amt: [f32; GRID_SIZE],

    pub count_trees: usize,
    pub per_tile_tree_count: [u8; GRID_SIZE],
    pub trees: [Option<Tree>; MAX_NUM_TREES],

    paused: bool,
    pub debug: DebugFlags,

    //Timers...
    debug_log_timer: TargetTimer,
    perf_timer: Option<AverageDurationTimer<20>>,

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
            tiles: [GroundCover::Grass; GRID_SIZE],
            tile_light_amt: [0.0; GRID_SIZE],
            count_trees: 0,
            per_tile_tree_count: [0; GRID_SIZE],
            trees: [None; MAX_NUM_TREES],

            paused: false,
            debug: DebugFlags {
                highlight_impending_seed: false,
                show_grid: false,
                show_trees: true,
            },

            debug_log_timer: TargetTimer::new(Duration::from_secs(1)),
            perf_timer: Some(AverageDurationTimer::new()),

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
                *tile = GroundCover::Grass;
            }
        }

        const NUM_INITIAL_TREES: usize = 6;
        let mut plant_locations = Vec::with_capacity(NUM_INITIAL_TREES);
        for _ in 0..NUM_INITIAL_TREES {
            plant_locations.push(
                WorldPosition {
                    coord: TileCoordinate {
                        x: result.rng.gen_range(0..GRID_DIM) as i32,
                        y: result.rng.gen_range(0..GRID_DIM) as i32,
                    },
                    offset: TileOffset {
                        x: result.rng.gen_range(0.0..1.0),
                        y: result.rng.gen_range(0.0..1.0),
                    },
                }
                // WorldPosition {
                //     coord: TileCoordinate {
                //         x: (GRID_DIM / 2 )as i32,
                //         y: (GRID_DIM / 2 )as i32,
                //     },
                //     offset: TileOffset {
                //         x: result.rng.gen_range(0.0..1.0),
                //         y: result.rng.gen_range(0.0..1.0),
                //     },
                // }
            );
        }

        let species = [
            TreeSpecies::Ash,
            TreeSpecies::Fir,
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


    pub fn iter_trees_on_tiles_in_radius<'s, 't>(&'s self, pos: WorldPosition, radius: f32) -> impl Iterator<Item=&'t Tree>
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
        unsafe { TreeRegionIterator::new((min_x, min_y), (max_x, max_y), &self.trees) }
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
            let tree_index = tree_slot_index!(tile_index, num_trees_on_tile);

            // SAFETY:
            //  self.num_trees_on_tile assumed to be accurate.
            let tree_opt = unsafe { self.trees.get_unchecked_mut(tree_index) };

            debug_assert!(tree_opt.is_none());

            *(tree_opt) = Some(Tree::new(species, pos));

            // SAEFTY:
            //  We've just checked the x, y uset to create tile_index
            unsafe { *(self.per_tile_tree_count.get_unchecked_mut(tile_index)) += 1 };
            self.count_trees += 1;
        }
    }

    //TODO: unsafe delete_tree(tree_slot_index)
    fn process_pending_tree_deletes(&mut self, pos: TileCoordinate)  {
        let x = pos.x as usize;
        let y = pos.y as usize;

        debug_assert!(x < GRID_DIM);
        debug_assert!(x >= 0);
        debug_assert!(y < GRID_DIM);
        debug_assert!(y >= 0);

        let tile_index = tile_index!(x, y);

        // Set all pending deletes to None
        {
            let mut delete_count = 0;

            // SAEFTY:
            //  We've just checked that x, y are in bounds
            for tree_opt in unsafe { self.get_tree_slots_on_tile_unchecked_mut(tile_index) } {
                if tree_opt.is_some() && tree_opt.unwrap().to_delete == true {
                    *tree_opt = None;
                    delete_count += 1;
                }
            }

            // SAEFTY:
            //  We've just checked that x, y are in bounds
            unsafe { *self.per_tile_tree_count.get_unchecked_mut(tile_index) -= delete_count as u8; }
            self.count_trees -= delete_count;
        }

        // Pack all living trees to the front of the sub-array
        {
            // SAEFTY:
            //  We've just checked that x, y are in bounds
            let count_trees = unsafe { *self.per_tile_tree_count.get_unchecked_mut(tile_index) as usize };

            let mut read_index = 0;
            let mut write_index = 0;

            // SAEFTY:
            //  We've just checked that x, y are in bounds
            let tree_slots = unsafe { self.get_tree_slots_on_tile_unchecked_mut(tile_index) };
            while write_index < count_trees && read_index < NUM_TREES_PER_TILE {
                if read_index != write_index && tree_slots.get(read_index).unwrap().is_some() {
                    tree_slots.swap(read_index, write_index);
                }

                read_index += 1;
                if tree_slots.get(write_index).unwrap().is_some() {
                    write_index += 1;
                }
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
        self.debug.show_trees = input.show_trees;
        self.debug.highlight_impending_seed = input.show_seed;

        self.paused = input.pause;
        if self.paused { return; }

        self.update_trees(dt_s);

        //Janky workaround for borrow check
        let mut timer = unsafe { self.perf_timer.take().unwrap_unchecked() };
        timer.measure(|| {
            self.update_grass();
        });
        self.perf_timer = Some(timer);

        self.one_sec_sin = f32::sin(dt_s);

        // if let TimerState::Ready(_) = self.debug_log_timer.check() {
        //     self.debug_log_timer.reset();
        //     debug!("{:?}", self.perf_timer.as_ref().unwrap().average());
        // }
    }

    fn update_trees(&mut self, dt_s: f32) {
        let mut count_plant_events = 0;
        let mut count_pending_deletes = 0;

        let mut plant_events = [MaybeUninit::uninit(); 100];
        let mut pending_deletes = [MaybeUninit::uninit(); 100];

        let mut push_new_tree_pos = |plant_event| {
            plant_events.get_mut(count_plant_events).unwrap().write(plant_event);
            count_plant_events += 1;
        };

        let mut push_new_tree_delete = |pos| {
            pending_deletes.get_mut(count_pending_deletes).unwrap().write(pos);
            count_pending_deletes += 1;
        };

        #[derive(Copy, Clone)]
        struct PlantEvent {
            pos: WorldPosition,
            species: TreeSpecies,
        }

        let mut tile_index = 0;
        while tile_index < GRID_SIZE {
            // SAFETY:
            //  tile_index ranging from 0..GRID_SIZE, and every tile has a corresponding tree count.
            let num_trees_on_tile = {
                *(unsafe { self.per_tile_tree_count.get_unchecked(tile_index) }) as usize
            };

            let mut tree_index = 0;
            while tree_index < num_trees_on_tile {
                let index = tree_slot_index!(tile_index, tree_index);

                // SAFETY:
                //  index is constructed by tile_index (see above) incremented by tree_index.
                //  tree_index must be < num_trees_on_tile
                let tree = unsafe { self.trees.get_unchecked_mut(index).as_mut().unwrap_unchecked() };
                let grow_stage = tree.grow(dt_s);

                use TreeGrowthStage::*;
                match grow_stage {
                    Mature | Old | Decline => {
                        let seed_multiplier = match grow_stage {
                            Mature  => 1.0,
                            Old     => 0.5,
                            Decline => 0.2,
                            _ => 0.0,
                        };

                        if tree.seed_timer <= 0.0 {
                            let numerator = 1;
                            let denominator = (10.0 * (1.0 / seed_multiplier)) as u32;

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
                                        push_new_tree_pos(
                                            PlantEvent {
                                                pos: plant_position,
                                                species: tree.species,
                                            }
                                        );
                                    }

                                    tree.seed_timer = {
                                        let seed_rate = tree.species.seed_rate();
                                        let min = seed_rate.average - seed_rate.variation;
                                        let max = seed_rate.average + seed_rate.variation;

                                        self.rng.gen_range(min..=max)
                                    };
                                }
                            }
                        }
                    },
                    Stump => {
                        if self.rng.gen_ratio(1, 1000) {
                            tree.to_delete = true;
                            push_new_tree_delete(tree.position.coord);
                        }
                    },
                    _ => {}
                }

                tree_index += 1;
            }

            tile_index += 1;
        }

        // NOTE:
        //  Only checking the crowd radius of the _PLANTED_ tree.
        //  i.e a tree with a large crowd radius will not prevent being "crowded" by tigher packing plants.
        for index in 0..count_plant_events {
            // SAFETY:
            //  count_plant_events was used to write the values, now upper bound for iteration.
            let PlantEvent { pos, species } = unsafe { plant_events.get_unchecked(index).assume_init() };
            let (crowd_radius, _) = species.seed_radius();
            let crowd_radius_sq = crowd_radius * crowd_radius;

            let clear_to_plant = self
                .iter_trees_on_tiles_in_radius(pos, crowd_radius)
                .all(|t| pos.distance_sq(&t.position) >= crowd_radius_sq);

            if clear_to_plant {
                self.plant_tree(pos, species);
            }
        }

        if count_pending_deletes > 0 {
            for index in 0..count_pending_deletes {
                // SAFETY:
                //  count_pending_deletes was used to write the values, now upper bound for iteration.
                let tile_pos = unsafe { pending_deletes.get_unchecked(index).assume_init() };
                self.process_pending_tree_deletes(tile_pos);
            }
        }
    }

    fn update_grass(&mut self) {
        let mut new_grass_state: [GroundCover; GRID_SIZE] = self.tiles;

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
                        *(new_grass_state.get_unchecked_mut(tile_index)) = GroundCover::Dirt;
                    }
                } else {
                    // SAFETY:
                    //  tile_index constructed from : x, y ranging from 0..GRID_DIM
                    if let GroundCover::Dirt = unsafe { self.tiles.get_unchecked(tile_index) } {
                        for neighbor_x in (x-1)..=(x+1) {
                            for neighbor_y in (y-1)..=(y+1) {
                                if neighbor_x == neighbor_y { continue; }

                                if
                                    (neighbor_x >= 0) && (neighbor_x < GRID_DIM as i32) &&
                                    (neighbor_y >= 0) && (neighbor_y < GRID_DIM as i32)
                                {
                                    let neighbor_index = tile_index!(neighbor_x, neighbor_y);
                                    if let GroundCover::Grass = self.tiles.get(neighbor_index).unwrap() {
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
                                *(new_grass_state.get_unchecked_mut(tile_index)) = GroundCover::Grass;
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
    pub show_seed: bool,
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
            show_grid: Default::default(),
            show_seed: Default::default(),

            show_trees: true,
        }
    }
}
