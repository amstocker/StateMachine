use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};


const WIDTH: usize = 100;
const HEIGHT: usize = 100;

pub type ItemID = usize;
pub const NoItem: ItemID = usize::MAX;

fn generate_id() -> ItemID {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}


#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize
}

impl Coordinate {
    pub fn to_index(&self) -> usize {
        self.y * WIDTH + self.x
    }

    pub fn from_index(index: usize) -> Self {
        Coordinate { x: index % WIDTH, y: index / WIDTH }
    }
}

impl From<(usize, usize)> for Coordinate {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize
}

impl From<(usize, usize)> for Size {
    fn from((width, height): (usize, usize)) -> Self {
        Size { width, height }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Region {
    top_left: Coordinate,
    size: Size
}

impl From<[(usize, usize); 2]> for Region {
    fn from([(x1, y1), (x2, y2)]: [(usize, usize); 2]) -> Self {
        Region {
            top_left: (x1, y1).into(),
            size: (x2 - x1, y2 - y1).into()
        }
    }
}

impl Region {
    pub fn bottom_right(&self) -> Coordinate {
        Coordinate {
            x: self.top_left.x + self.size.width,
            y: self.top_left.y + self.size.height
        }
    }

    pub fn contains(&self, Coordinate { x, y }: Coordinate) -> bool {
        let bottom_right = self.bottom_right();
        return x >= self.top_left.x
            && x <= bottom_right.x
            && y >= self.top_left.y
            && y <= bottom_right.y
    }

    pub fn intersects(&self, other: Region) -> bool {
        let bottom_right = self.bottom_right();
        let bottom_right_other = other.bottom_right();
        return !(
            other.top_left.x > bottom_right.x ||
            bottom_right_other.y > self.top_left.y ||
            bottom_right_other.x < self.top_left.x ||
            other.top_left.y < bottom_right.y
        )
    }
}


pub enum Widget {
    Sound
}

pub struct Item {
    id: ItemID,
    widget: Widget,
    size: Size
}

struct Grid {
    occupation_map: [ItemID; WIDTH * HEIGHT],
    items: HashMap<ItemID, Item>
}

impl Grid {
    pub fn new() -> Self {
        Self {
            occupation_map: [NoItem; WIDTH * HEIGHT],
            items: HashMap::new()
        }
    }

    pub fn is_occupied_in_region(&self, region: Region) -> bool {
        let top_left_index = region.top_left.to_index();
        for j in 0..=region.size.height {
            for i in 0..=region.size.width {
                if self.occupation_map[top_left_index + (j * WIDTH) + i] != NoItem {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_item(&self, coordinate: Coordinate) -> Option<&Item> {
        let id = self.occupation_map[coordinate.to_index()];
        if id != NoItem {
            self.items.get(&id)
        } else {
            None
        }
    }

    pub fn set_occupied_in_region(&mut self, id: ItemID, region: Region) {
        let top_left_index = region.top_left.to_index();
        for j in 0..=region.size.height {
            for i in 0..=region.size.width {
                self.occupation_map[top_left_index + (j * WIDTH) + i] = id;
            }
        } 
    }

    pub fn add_item(&mut self, top_left: Coordinate, item: Item) -> Result<ItemID, GridError> {
        if top_left.x + item.size.width >= WIDTH || top_left.y + item.size.height >= HEIGHT {
            return Err(GridError::OutOfBounds);
        }

        let region = Region {
            top_left,
            size: item.size
        };
        if self.is_occupied_in_region(region) {
            return Err(GridError::Occupied);
        }
        self.set_occupied_in_region(item.id, region);

        let index = top_left.to_index();
        self.items.insert(item.id, item);
        Ok(index)
    }

    pub fn coord_to_screen(&self, coordinate: Coordinate) -> Option<(f32, f32)> {
        // Grid coord may not be shown on the screen?
        todo!()
    }
}

pub enum GridError {
    Occupied,
    OutOfBounds
}