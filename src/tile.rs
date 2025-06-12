use geo::{Polygon, Coord, LineString, MultiPolygon, BooleanOps};
use crate::GeoTilerError;
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
        
        for rp in resulting_polygons {
            tile.polygons.push(rp);
        }
    }

    Ok(())
}