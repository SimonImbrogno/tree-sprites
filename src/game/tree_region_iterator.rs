use super::trees::Tree;
use super::game_state::{ GRID_DIM, NUM_TREES_PER_TILE };
use super::game_state::tree_slot_index_xyt;

pub struct TreeRegionIterator<'t> {
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
    type Item = (usize, &'t Tree);

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
                return Some( (slot_index, result.as_ref().unwrap()) );
            }
        }

        None
    }
}

pub struct TreeRegionIteratorMut<'t> {
    min_x:  usize,
    min_y:  usize,

    max_x:  usize,
    max_y:  usize,

    curr_x: usize,
    curr_y: usize,

    tree_sub_index: usize,

    trees: &'t mut [Option<Tree>],
}

impl <'t> TreeRegionIteratorMut<'t> {
    pub unsafe fn new(min: (usize, usize), max: (usize, usize), trees: &'t mut [Option<Tree>]) -> Self {
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

impl<'t> Iterator for TreeRegionIteratorMut<'t> {
    type Item = (usize, &'t mut Tree);

    fn next(&mut self) -> Option<Self::Item> {

        while self.curr_y <= self.max_y {
            // SAFETY:
            //  curr_x, curr_y are both in range 0..GRID_DIM, tree_sub_index is reset when >= NUM_TREES_PER_TILE
            let slot_index = tree_slot_index_xyt!(self.curr_x, self.curr_y, self.tree_sub_index);

            // SAFETY:
            //  Compiler cannot statically determine that our iterator won't return a reference to the same part of the slice twice.
            //  We know that we won't but we must unsafely re-cast a ptr to work around this.
            let result = unsafe {
                let slice_ptr = self.trees.as_mut_ptr();
                let tree_ptr = slice_ptr.add(slot_index);

                &mut *tree_ptr
            };

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
                return Some( (slot_index, result.as_mut().unwrap()) );
            }
        }

        None
    }
}
