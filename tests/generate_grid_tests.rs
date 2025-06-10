use geo_tiler::{generate_grid, Tile};
use approx::assert_relative_eq;

#[test]
fn test_generate_grid_basic_functionality() {
    let step: usize = 10;
    let grid: Vec<Tile> = generate_grid(step);
    
    // longitude: -180 to 180 (360 degrees) with step 10 = 36 tiles
    // latitude: -90 to 90 (180 degrees) with step 10 = 18 tiles
    // total: 36 * 18 = 648 tiles
    assert_eq!(grid.len(), 648);
    
    for tile in &grid {
        assert_eq!(tile.vertices.len(), 4);
        assert_eq!(tile.polygons.len(), 0);
    }
}

#[test]
fn test_generate_grid_step_1() {
    let step: usize = 1;
    let grid: Vec<Tile> = generate_grid(step);
    
    // 360 * 180 = 64,800 tiles for step size 1
    assert_eq!(grid.len(), 64800);
}

#[test]
fn test_generate_grid_large_step() {
    let step: usize = 90;
    let grid: Vec<Tile> = generate_grid(step);
    
    // longitude: 360/90 = 4 tiles, latitude: 180/90 = 2 tiles, total: 8
    assert_eq!(grid.len(), 8);
}

#[test]
fn test_coordinate_ranges() {
    let step: usize = 45;
    let grid: Vec<Tile> = generate_grid(step);
    
    for tile in &grid {
        for vertex in &tile.vertices {
            assert!(vertex.0 >= -180.0);
            assert!(vertex.0 <= 180.0);
            assert!(vertex.1 >= -90.0);
            assert!(vertex.1 <= 90.0);
        }
    }
}

#[test]
fn test_tile_dimensions() {
    let step: usize = 20;
    let grid: Vec<Tile> = generate_grid(step);
    
    for tile in &grid {
        let vertices: &Vec<(f64, f64)> = &tile.vertices;
        let width: f64 = (vertices[1].0 - vertices[0].0).abs();
        let height: f64 = (vertices[2].1 - vertices[0].1).abs();
        
        assert_relative_eq!(width, step as f64);
        assert_relative_eq!(height, step as f64);
    }
}

#[test]
fn test_no_overlapping_tiles() {
    let step: usize = 60;
    let grid: Vec<Tile> = generate_grid(step);
    
    for (i, tile1) in grid.iter().enumerate() {
        for (j, tile2) in grid.iter().enumerate() {
            if i != j {
                let v1: &Vec<(f64, f64)> = &tile1.vertices;
                let v2: &Vec<(f64, f64)> = &tile2.vertices;
                
                if (v1[0].0 - v2[0].0).abs() < 0.001 && (v1[1].0 - v2[1].0).abs() < 0.001 {
                    assert!(
                        v1[0].1 >= v2[2].1 || v2[0].1 >= v1[2].1,
                        "tiles with same longitude should not overlap in latitude"
                    );
                }
            }
        }
    }
}

#[test]
fn test_complete_coverage() {
    let step: usize = 90;
    let grid: Vec<Tile> = generate_grid(step);
    
    assert_eq!(grid.len(), 8);
    
    let mut min_lon: f64 = f64::INFINITY;
    let mut max_lon: f64 = f64::NEG_INFINITY;
    let mut min_lat: f64 = f64::INFINITY;
    let mut max_lat: f64 = f64::NEG_INFINITY;
    
    for tile in &grid {
        for vertex in &tile.vertices {
            min_lon = min_lon.min(vertex.0);
            max_lon = max_lon.max(vertex.0);
            min_lat = min_lat.min(vertex.1);
            max_lat = max_lat.max(vertex.1);
        }
    }
    
    assert_relative_eq!(min_lon, -180.0);
    assert_relative_eq!(max_lon, 180.0);
    assert_relative_eq!(min_lat, -90.0);
    assert_relative_eq!(max_lat, 90.0);
}
