use geo_types::Polygon;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Tile {
    pub vertices: Vec<(f64, f64)>,
    pub polygons: Vec<Polygon<f64>>,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertices: {:?} and {} polygons",
            self.vertices,
            self.polygons.len()
        )
    }
}

/// Generates a grid of tiles covering the entire Earth's surface using longitude and latitude coordinates.
///
/// This function creates a uniform grid by dividing the Earth's surface into rectangular tiles
/// of equal angular size. The grid covers longitude from -180° to 180° and latitude from -90° to 90°.
/// Each tile is represented as a quadrilateral with four corner vertices.
///
/// # Arguments
///
/// * `step` - The angular step size in degrees for both longitude and latitude divisions.
///
/// # Returns
///
/// * `Vec<Tile>` - A vector containing all generated tiles with four vertices each and empty polygons.
///
/// # Grid Coverage
///
/// * Longitude: -180° to +180° (360° total)
/// * Latitude: -90° to +90° (180° total)
/// * Total tiles: (360 / step) × (180 / step)
///
/// # Vertex Ordering
///
/// * `vertices[0]`: Bottom-left (longitude, latitude)
/// * `vertices[1]`: Bottom-right (longitude + step, latitude)
/// * `vertices[2]`: Top-left (longitude, latitude + step)
/// * `vertices[3]`: Top-right (longitude + step, latitude + step)
///
/// # Examples
///
/// ```
/// use geo_tiler::generate_grid;
/// 
/// let grid = generate_grid(10);
/// assert_eq!(grid.len(), 648); // 36 × 18 tiles
///
/// let first_tile = &grid[0];
/// assert_eq!(first_tile.vertices[0], (-180.0, -90.0)); // bottom-left corner
/// ```
pub fn generate_grid(step: usize) -> Vec<Tile> {
    let mut grid: Vec<Tile> = Vec::new();

    for i in (-180..180).step_by(step) {
        for j in (-90..90).step_by(step) {
            let bl: (f64, f64) = (i as f64, j as f64);
            let br: (f64, f64) = ((i + step as i32) as f64, j as f64);
            let tl: (f64, f64) = (i as f64, (j + step as i32) as f64);
            let tr: (f64, f64) = ((i + step as i32) as f64, (j + step as i32) as f64);

            let tile: Tile = Tile {
                vertices: vec![bl, br, tl, tr],
                polygons: Vec::new()
            };

            grid.push(tile);
        }
    }

    grid
}