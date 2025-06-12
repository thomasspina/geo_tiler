use geo::{Polygon, Coord, LineString, MultiPolygon, BooleanOps};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Tile {
    pub vertices: Polygon<f64>,
    pub polygons: Vec<Polygon<f64>>,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tile {{ vertices: {:?}, polygons: {:?} }}",
            self.vertices, self.polygons
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
/// use geo::Coord;
///
/// let grid = generate_grid(10);
/// assert_eq!(grid.len(), 648); // 36 × 18 tiles
///
/// let first_tile = &grid[0];
/// let coords: Vec<Coord<f64>> = first_tile.vertices.exterior().coords().cloned().collect();
/// assert_eq!(coords[0], Coord { x: -180.0, y: -90.0 }); // bottom-left corner
/// ```
pub fn generate_grid(step: usize) -> Vec<Tile> {
    let mut grid: Vec<Tile> = Vec::new();

    for i in (-180..180).step_by(step) {
        for j in (-90..90).step_by(step) {
            let bl: Coord<f64> = Coord { x: i as f64, y: j as f64 };
            let br: Coord<f64> = Coord { x: (i + step as i32) as f64, y: j as f64 };
            let tl: Coord<f64> = Coord { x: i as f64, y: (j + step as i32) as f64 };
            let tr: Coord<f64> = Coord { x: (i + step as i32) as f64, y: (j + step as i32) as f64 };

            let tile: Tile = Tile {
                vertices: Polygon::new(LineString::new(vec![bl, br, tl, tr]), vec![]),
                polygons: Vec::new()
            };

            grid.push(tile);
        }
    }

    grid
}

pub fn clip_polygon_to_tiles(grid: &mut Vec<Tile>, polygon: &Polygon<f64>) {
    for tile in grid {
        let resulting_polygons: MultiPolygon<f64> = tile.vertices.intersection(polygon);
        
        for rp in resulting_polygons {
            tile.polygons.push(rp);
        }
    }
}