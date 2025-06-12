use geo::{polygon, Polygon};
use geo_tiler::{clip_polygon_to_tiles, Tile};

// helper function to create a square tile
fn create_square_tile(x: f64, y: f64, size: f64) -> Tile {
    let vertices: Polygon = polygon![
        (x: x, y: y),
        (x: x + size, y: y),
        (x: x + size, y: y + size),
        (x: x, y: y + size),
        (x: x, y: y),
    ];
    Tile {
        vertices,
        polygons: Vec::new(),
    }
}

// helper function to create a 2x2 grid of tiles
fn create_2x2_grid(tile_size: f64) -> Vec<Tile> {
    vec![
        create_square_tile(0.0, 0.0, tile_size),
        create_square_tile(tile_size, 0.0, tile_size),
        create_square_tile(0.0, tile_size, tile_size),
        create_square_tile(tile_size, tile_size, tile_size),
    ]
}

#[test]
fn test_polygon_fully_within_single_tile() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 2.0, y: 2.0),
        (x: 4.0, y: 2.0),
        (x: 4.0, y: 4.0),
        (x: 2.0, y: 4.0),
        (x: 2.0, y: 2.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // first tile should have the polygon
    assert_eq!(grid[0].polygons.len(), 1);
    // other tiles should have no polygons
    assert_eq!(grid[1].polygons.len(), 0);
    assert_eq!(grid[2].polygons.len(), 0);
    assert_eq!(grid[3].polygons.len(), 0);
}

#[test]
fn test_polygon_spanning_multiple_tiles() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 5.0, y: 5.0),
        (x: 15.0, y: 5.0),
        (x: 15.0, y: 15.0),
        (x: 5.0, y: 15.0),
        (x: 5.0, y: 5.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // all tiles should have intersection polygons
    assert_eq!(grid[0].polygons.len(), 1);
    assert_eq!(grid[1].polygons.len(), 1);
    assert_eq!(grid[2].polygons.len(), 1);
    assert_eq!(grid[3].polygons.len(), 1);
}

#[test]
fn test_polygon_no_intersection() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 25.0, y: 25.0),
        (x: 30.0, y: 25.0),
        (x: 30.0, y: 30.0),
        (x: 25.0, y: 30.0),
        (x: 25.0, y: 25.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // no tiles should have polygons
    assert_eq!(grid[0].polygons.len(), 0);
    assert_eq!(grid[1].polygons.len(), 0);
    assert_eq!(grid[2].polygons.len(), 0);
    assert_eq!(grid[3].polygons.len(), 0);
}

#[test]
fn test_polygon_edge_intersection() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 8.0, y: 0.0),
        (x: 12.0, y: 0.0),
        (x: 12.0, y: 10.0),
        (x: 8.0, y: 10.0),
        (x: 8.0, y: 0.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // First and second tile should have intersections
    assert_eq!(grid[0].polygons.len(), 1);
    assert_eq!(grid[1].polygons.len(), 1);
    assert_eq!(grid[2].polygons.len(), 0);
    assert_eq!(grid[3].polygons.len(), 0);
}

#[test]
fn test_complex_polygon_intersection() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    // L-shaped polygon
    let polygon: Polygon = polygon![
        (x: 5.0, y: 5.0),
        (x: 15.0, y: 5.0),
        (x: 15.0, y: 10.0),
        (x: 10.0, y: 10.0),
        (x: 10.0, y: 15.0),
        (x: 5.0, y: 15.0),
        (x: 5.0, y: 5.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // tiles 0, 1, and 2 should have intersections
    assert!(grid[0].polygons.len() > 0);
    assert!(grid[1].polygons.len() > 0);
    assert!(grid[2].polygons.len() > 0);
    // tile 3 might or might not have intersection depending on exact calculation
}

#[test]
fn test_empty_grid() {
    let mut grid: Vec<Tile> = Vec::new();
    
    let polygon: Polygon = polygon![
        (x: 0.0, y: 0.0),
        (x: 10.0, y: 0.0),
        (x: 10.0, y: 10.0),
        (x: 0.0, y: 10.0),
        (x: 0.0, y: 0.0),
    ];

    // should not panic with empty grid
    clip_polygon_to_tiles(&mut grid, &polygon);
    
    assert_eq!(grid.len(), 0);
}

#[test]
fn test_multiple_polygons_same_tile() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon1: Polygon = polygon![
        (x: 1.0, y: 1.0),
        (x: 3.0, y: 1.0),
        (x: 3.0, y: 3.0),
        (x: 1.0, y: 3.0),
        (x: 1.0, y: 1.0),
    ];

    let polygon2: Polygon = polygon![
        (x: 5.0, y: 5.0),
        (x: 7.0, y: 5.0),
        (x: 7.0, y: 7.0),
        (x: 5.0, y: 7.0),
        (x: 5.0, y: 5.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon1);
    clip_polygon_to_tiles(&mut grid, &polygon2);

    // first tile should have two polygons
    assert_eq!(grid[0].polygons.len(), 2);
}

#[test]
fn test_polygon_touching_tile_corner() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 9.0, y: 9.0),
        (x: 11.0, y: 9.0),
        (x: 11.0, y: 11.0),
        (x: 9.0, y: 11.0),
        (x: 9.0, y: 9.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // all four tiles should have small intersections
    assert_eq!(grid[0].polygons.len(), 1);
    assert_eq!(grid[1].polygons.len(), 1);
    assert_eq!(grid[2].polygons.len(), 1);
    assert_eq!(grid[3].polygons.len(), 1);
}

#[test]
fn test_concave_polygon() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 10.0, y: 5.0),
        (x: 12.0, y: 8.0),
        (x: 15.0, y: 8.0),
        (x: 13.0, y: 11.0),
        (x: 14.0, y: 15.0),
        (x: 10.0, y: 13.0),
        (x: 6.0, y: 15.0),
        (x: 7.0, y: 11.0),
        (x: 5.0, y: 8.0),
        (x: 8.0, y: 8.0),
        (x: 10.0, y: 5.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // multiple tiles should have intersections
    let total_intersections: usize = grid.iter().map(|tile| tile.polygons.len()).sum();
    assert!(total_intersections > 0);
}

#[test]
fn test_very_small_polygon() {
    let mut grid: Vec<Tile> = create_2x2_grid(10.0);
    
    let polygon: Polygon = polygon![
        (x: 5.0, y: 5.0),
        (x: 5.1, y: 5.0),
        (x: 5.1, y: 5.1),
        (x: 5.0, y: 5.1),
        (x: 5.0, y: 5.0),
    ];

    clip_polygon_to_tiles(&mut grid, &polygon);

    // should still work with very small polygons
    assert_eq!(grid[0].polygons.len(), 1);
}