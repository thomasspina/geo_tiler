use geo::{Polygon, Coord, LineString, MultiPolygon, BooleanOps};
use crate::{GeoTilerError, densify_edges};
use std::fmt;

/// Default maximum distance in degrees between consecutive points during edge densification.
const DEFAULT_MAX_DISTANCE_BETWEEN_POINTS: f64 = 0.5;

/// Represents a single tile in a geographic grid system.
/// Contains the tile's rectangular boundary and any polygon fragments that intersect with it.
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
/// * `Result<Vec<Tile>, GeoTilerError>` - A vector containing all generated tiles with four vertices each 
///   and empty polygons, or an error if the parameters are invalid.
///
/// # Errors
///
/// Returns `GeoTilerError::GridGenerationError` if:
/// * `step` is 0 (would cause infinite loop)
/// * `step` is greater than 180 (would produce no tiles or invalid tiles)
/// * `step` does not evenly divide 360 or 180 (would produce incomplete coverage)
///
/// # Grid Coverage
///
/// * Longitude: -180° to +180° (360° total)
/// * Latitude: -90° to +90° (180° total)
/// * Total tiles: (360 / step) × (180 / step)
pub fn generate_grid(step: usize) -> Result<Vec<Tile>, GeoTilerError> {
    if step == 0 {
        return Err(GeoTilerError::GridGenerationError(
            "Step size must be greater than 0".to_string()
        ));
    }

    if step > 180 {
        return Err(GeoTilerError::GridGenerationError(
            format!("Step size {} is too large. Maximum allowed is 180 degrees", step)
        ));
    }

    if 360 % step != 0 {
        return Err(GeoTilerError::GridGenerationError(
            format!("Step size {} does not evenly divide 360 degrees. This would result in incomplete longitude coverage", step)
        ));
    }

    if 180 % step != 0 {
        return Err(GeoTilerError::GridGenerationError(
            format!("Step size {} does not evenly divide 180 degrees. This would result in incomplete latitude coverage", step)
        ));
    }

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

    Ok(grid)
}

/// Clips a polygon to a grid of tiles and stores the resulting intersections in each tile.
///
/// This function takes a polygon and computes its intersection with each tile in the grid.
/// The resulting polygon fragments are stored in each tile's `polygons` vector. 
///
/// # Arguments
///
/// * `grid` - A mutable reference to a vector of tiles. Each tile's `polygons` vector will be
///            updated with any intersection fragments.
/// * `polygon` - The polygon to be clipped against the tile grid.
pub fn clip_polygon_to_tiles(grid: &mut Vec<Tile>, polygon: &Polygon<f64>) -> Result<(), GeoTilerError> {
    
    let vertex_count: usize = polygon.exterior().coords().count();
    if vertex_count < 4 {  
        return Err(GeoTilerError::InvalidPolygonError(
            format!("Polygon must have at least 3 vertices, found {}", vertex_count - 1)
        ));
    }

    for coord in polygon.exterior().coords() {
        if !coord.x.is_finite() || !coord.y.is_finite() {
            return Err(GeoTilerError::InvalidPolygonError(
                "Polygon contains NaN or infinite coordinates".to_string()
            ));
        }
    }

    for tile in grid {
        let resulting_polygons: MultiPolygon<f64> = tile.vertices.intersection(polygon);

        for mut rp in resulting_polygons {
            densify_edges(&mut rp, DEFAULT_MAX_DISTANCE_BETWEEN_POINTS);
            tile.polygons.push(rp);
        }
    }

    Ok(())
}

/// Clamps all polygons in each tile to ensure their coordinates stay within the tile boundaries.
///
/// This function addresses floating-point precision errors that can occur during polygon intersection
/// operations, which may result in polygon vertices slightly extending beyond their containing tile's
/// boundaries. Such precision errors can prevent proper triangulation of the polygons.
///
/// # Arguments
///
/// * `tiles` - A mutable reference to a vector of tiles. Each tile's polygons will have their
///             coordinates clamped to the tile's boundary limits.
pub fn clamp_polygons(tiles: &mut Vec<Tile>) {
    for tile in tiles {

        // clamping is required because of float math inaccuracies which prevent triangulation from working
        let tile_exterior: &LineString = tile.vertices.exterior();
        for polygon in tile.polygons.iter_mut() {
            
            clamp_polygon(polygon, tile_exterior);
        }

    }
}

/// Clamps a single polygon's coordinates to fit within the specified tile boundary.
///
/// This function calculates the minimum and maximum x and y coordinates from the tile's exterior
/// boundary, then ensures all coordinates in the polygon's exterior ring fall within these bounds
/// using the clamp operation.
///
/// # Arguments
///
/// * `polygon` - A mutable reference to the polygon whose coordinates will be clamped.
/// * `tile_exterior` - The exterior boundary of the tile used to determine clamping limits.
fn clamp_polygon(polygon: &mut Polygon, tile_exterior: &LineString<f64>) {
    polygon.exterior_mut(|exterior| {
        let mut max_x: f64 = f64::MIN; let mut max_y: f64 = f64::MIN; 
        let mut min_x: f64 = f64::MAX; let mut min_y: f64 = f64::MAX;
        
        for coord in tile_exterior {
            max_x = max_x.max(coord.x);
            max_y = max_y.max(coord.y);
            min_x = min_x.min(coord.x);
            min_y = min_y.min(coord.y);
        }

        for coord in exterior.coords_mut() {
            coord.x = coord.x.clamp(min_x, max_x);
            coord.y = coord.y.clamp(min_y, max_y);
        }
    });
}