use super::vector::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct TileCoordinate {
    pub x: i32,
    pub y: i32,
}

pub type TileOffset = Vec2<f32>;

#[derive(Copy, Clone, Debug,)]
pub struct WorldPosition {
    pub coord: TileCoordinate,
    pub offset: TileOffset,
}

impl std::ops::Sub for WorldPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            coord: TileCoordinate {
                x: self.coord.x - rhs.coord.x,
                y: self.coord.y - rhs.coord.y,
            },
            offset: TileOffset {
                x: self.offset.x - rhs.offset.x,
                y: self.offset.y - rhs.offset.y,
            },
        }.normalize()
    }
}

impl std::ops::Add for WorldPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            coord: TileCoordinate {
                x: self.coord.x + rhs.coord.x,
                y: self.coord.y + rhs.coord.y,
            },
            offset: TileOffset {
                x: self.offset.x + rhs.offset.x,
                y: self.offset.y + rhs.offset.y,
            },
        }.normalize()
    }
}

impl std::ops::Sub<TileOffset> for WorldPosition {
    type Output = Self;

    fn sub(self, rhs: TileOffset) -> Self::Output {
        Self {
            coord: self.coord,
            offset: TileOffset {
                x: self.offset.x - rhs.x,
                y: self.offset.y - rhs.y,
            },
        }.normalize()
    }
}

impl std::ops::Add<TileOffset> for WorldPosition {
    type Output = Self;

    fn add(self, rhs: TileOffset) -> Self::Output {
        Self {
            coord: self.coord,
            offset: TileOffset {
                x: self.offset.x + rhs.x,
                y: self.offset.y + rhs.y,
            },
        }.normalize()
    }
}

impl WorldPosition {
    pub fn normalize(self) -> Self {
        let floor_x = (self.offset.x).floor();
        let floor_y = (self.offset.y).floor();

        Self {
            coord: TileCoordinate {
                x: self.coord.x + floor_x as i32,
                y: self.coord.y + floor_y as i32,
            },
            offset: TileOffset {
                x: self.offset.x - floor_x,
                y: self.offset.y - floor_y,
            },
        }
    }

    pub fn distance_sq(&self, other: &Self) -> f32 {
        let diff = (*other - *self);
        let diff_x = diff.coord.x as f32 + diff.offset.x;
        let diff_y = diff.coord.y as f32 + diff.offset.y;

        (diff_x * diff_x) + (diff_y * diff_y)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_sq(other).sqrt()
    }
}
