![Procedurally curved Earth mesh with point-based continental generation](https://github.com/user-attachments/assets/2ec19871-0a8d-4f2e-8bbe-6ecc225d6ae5)

# geo_tiler

A Rust library for converting 2D geographic coordinates (GeoJSON) into 3D meshes suitable for rendering on spherical globes. This library comes with an executable main file which creates JSON files that stores the 3D meshes generated from the GeoJSON provided. This library solves the fundamental challenge of representing flat map data on curved surfaces by generating properly triangulated meshes that conform to a sphere's curvature. You can find part of the logic for this crate explained in this article I [wrote](https://tricolor-albacore-d39.notion.site/From-Coordinates-to-Continents-Creating-a-3D-Globe-from-2D-Coordinates-1f83a2e4815d809b9606c3b5be791506?pvs=74).

## Features

- **Coordinate Transformation**: Convert latitude/longitude coordinates to 3D Cartesian coordinates
- **Spherical Interpolation**: Add intermediate points along polygon edges using proper spherical interpolation
- **Uniform Point Distribution**: Generate evenly distributed points on sphere surfaces using Fibonacci spiral method
- **Stereographic Projection**: Project 3D points to 2D for accurate triangulation
- **Constrained Delaunay Triangulation**: Generate optimal mesh triangulation while preserving polygon boundaries
- **Tile-based Processing**: Divide large geographic regions into manageable tiles for efficient processing

## Quick Start

```rust
use geo::{coord, LineString, Polygon};
use geo_tiler::{generate_polygon_feature_mesh, PolygonMeshData};

// Create a simple triangle polygon
let coords = vec![
    coord! {x: -30.0, y: -15.0},
    coord! {x: 0.0, y: 30.0},
    coord! {x: 30.0, y: -15.0},
    coord! {x: -30.0, y: -15.0},  // Closing coordinate
];

let polygon = Polygon::new(LineString::new(coords), vec![]);

// Generate 3D mesh data
match generate_polygon_feature_mesh(&polygon) {
    Ok(mesh_data) => {
        println!("Generated {} vertices and {} triangles", 
            mesh_data.vertices.len(), 
            mesh_data.triangles.len() / 3);
    }
    Err(e) => eprintln!("Error generating mesh: {}", e),
}
```

## Core Concepts

### The Problem

When rendering geographic data on a 3D globe, simply projecting 2D coordinates onto a sphere creates flat triangles that don't follow the sphere's curvature. This library solves this by:

1. **Adding interior points**: Using Fibonacci sphere distribution to create evenly-spaced points
2. **Proper interpolation**: Using spherical linear interpolation (SLERP) for curved edges
3. **Accurate triangulation**: Using stereographic projection to preserve angles during triangulation

### Algorithm Pipeline

1. **Parse polygon boundaries** from GeoJSON or geo types
2. **Generate mesh points**:
   - Optionally densify edges with interpolated points
   - Create Fibonacci lattice points and filter those inside the polygon
   - Convert all points to 3D Cartesian coordinates
3. **Triangulate**:
   - Rotate points to south pole for optimal projection
   - Apply stereographic projection to 2D
   - Perform constrained Delaunay triangulation
   - Map triangulation back to original 3D points
4. **Output mesh data** ready for rendering

## API Reference

### Primary Functions

#### `generate_polygon_feature_mesh`
Generates a complete 3D mesh from a 2D geographic polygon.

```rust
pub fn generate_polygon_feature_mesh(polygon: &Polygon) -> Result<PolygonMeshData, GeoTilerError>
```

#### `ll_to_cartesian`
Converts longitude/latitude to 3D Cartesian coordinates on a unit sphere.

```rust
pub fn ll_to_cartesian(longitude: f64, latitude: f64) -> Result<(f64, f64, f64), GeoTilerError>
```

#### `generate_grid`
Creates a grid of tiles covering the Earth's surface.

```rust
pub fn generate_grid(step: usize) -> Result<Vec<Tile>, GeoTilerError>
```

#### `clip_polygon_to_tiles`
Clips a polygon across multiple tiles for distributed processing.

```rust
pub fn clip_polygon_to_tiles(grid: &mut Vec<Tile>, polygon: &Polygon<f64>) -> Result<(), GeoTilerError>
```

### Data Structures

#### `PolygonMeshData`
```rust
pub struct PolygonMeshData {
    pub vertices: Vec<(f64, f64, f64)>,  // 3D points on unit sphere
    pub triangles: Vec<u32>,              // Flattened triangle indices
}
```

#### `Tile`
```rust
pub struct Tile {
    pub vertices: Polygon<f64>,           // Tile boundary
    pub polygons: Vec<Polygon<f64>>,      // Polygon fragments in this tile
}
```

## Error Handling

The library provides comprehensive error handling through the `GeoTilerError` enum:

- `CoordinateRangeError`: Invalid longitude/latitude values
- `ProjectionError`: Stereographic projection failures
- `FibonacciError`: Issues with point generation
- `RotationError`: Problems rotating points
- `MeshGenerationError`: General mesh generation failures
- `TriangulationError`: Constrained Delaunay triangulation failures

## Examples

### Processing GeoJSON Files

See the included binary example that processes Natural Earth data:

```bash
cargo run input.geojson output_directory/
```

### Custom Point Density

```rust
use geo_tiler::{get_mesh_points, fibonacci_sphere};

// Generate mesh with custom point density
let mut points = get_mesh_points(&polygon)?;

// Add more Fibonacci points if needed
let extra_points = fibonacci_sphere(5000)?;
// ... filter and add points as needed
```

## Sources

- [Martinez-Rueda Polygon Clipping](https://liorsinai.github.io/mathematics/2025/01/11/bentley-ottman.html) - Lior Sinai
- [Delaunay/Voronoi on a Sphere](https://www.redblobgames.com/x/1842-delaunay-voronoi-sphere/) – Red Blob Games
- [How to evenly distribute points on a sphere more effectively than the canonical Fibonacci Lattice](https://extremelearning.com.au/how-to-evenly-distribute-points-on-a-sphere-more-effectively-than-the-canonical-fibonacci-lattice/) - Martin Roberts
