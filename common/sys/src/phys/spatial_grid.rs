use vek::*;

pub struct SpatialGrid {
    // Uses two scales of grids so that we can have a hard limit on how far to search in the
    // smaller grid
    grid: hashbrown::HashMap<Vec2<i32>, Vec<specs::Entity>>,
    large_grid: hashbrown::HashMap<Vec2<i32>, Vec<specs::Entity>>,
    // Log base 2 of the cell size of the spatial grid
    lg2_cell_size: usize,
    // Log base 2 of the cell size of the large spatial grid
    lg2_large_cell_size: usize,
    // Entities with a radius over this value are store in the coarser large_grid
    // This is the amount of buffer space we need to add when finding the intersections with cells
    // in the regular grid
    radius_cutoff: u32,
    // Stores the largest radius of the entities in the large_grid
    // This is the amount of buffer space we need to add when finding the intersections with cells
    // in the larger grid
    // note: could explore some distance field type thing for querying whether there are large
    // entities nearby that necessitate expanding the cells searched for collision (and querying
    // how much it needs to be expanded)
    // TODO: log this to metrics?
    largest_large_radius: u32,
}

impl SpatialGrid {
    pub fn new(lg2_cell_size: usize, lg2_large_cell_size: usize, radius_cutoff: u32) -> Self {
        Self {
            grid: Default::default(),
            large_grid: Default::default(),
            lg2_cell_size,
            lg2_large_cell_size,
            radius_cutoff,
            largest_large_radius: radius_cutoff,
        }
    }

    /// Add an entity at the provided 2d pos into the spatial grid
    pub fn insert(&mut self, pos: Vec2<i32>, radius: u32, entity: specs::Entity) {
        if radius <= self.radius_cutoff {
            let cell = pos.map(|e| e >> self.lg2_cell_size);
            self.grid.entry(cell).or_default().push(entity);
        } else {
            let cell = pos.map(|e| e >> self.lg2_large_cell_size);
            self.large_grid.entry(cell).or_default().push(entity);
            self.largest_large_radius = self.largest_large_radius.max(radius);
        }
    }

    /// Get an iterator over the entities overlapping the
    /// provided axis aligned bounding region
    /// NOTE: for best optimization of the iterator use `for_each` rather than a
    /// for loop
    // TODO: a circle would be tighter (how efficient would it be to query the cells
    // intersecting a circle?)
    pub fn in_aabr<'a>(&'a self, aabr: Aabr<i32>) -> impl Iterator<Item = specs::Entity> + 'a {
        let iter = |max_entity_radius, grid: &'a hashbrown::HashMap<_, _>, lg2_cell_size| {
            // Add buffer for other entity radius
            let min = aabr.min - max_entity_radius as i32;
            let max = aabr.max + max_entity_radius as i32;
            // Convert to cells
            let min = min.map(|e| e >> lg2_cell_size);
            let max = max.map(|e| (e + (1 << lg2_cell_size) - 1) >> lg2_cell_size);

            (min.x..=max.x)
                .flat_map(move |x| (min.y..=max.y).map(move |y| Vec2::new(x, y)))
                .flat_map(move |cell| grid.get(&cell).into_iter().flatten())
                .copied()
        };

        iter(self.radius_cutoff, &self.grid, self.lg2_cell_size).chain(iter(
            self.largest_large_radius,
            &self.large_grid,
            self.lg2_large_cell_size,
        ))
    }
}
