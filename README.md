# Geo Tiler

A Rust library for converting 2D geographic polygons into 3D spherical meshes. Transform GeoJSON coordinates into properly triangulated meshes that conform to a sphere's curvature—ideal for rendering geographic data on 3D globes. [Crates.io link](https://crates.io/crates/geo_tiler).

![Procedurally curved Earth mesh with point-based continental generation](https://github.com/user-attachments/assets/2ec19871-0a8d-4f2e-8bbe-6ecc225d6ae5)

## The Problem

Projecting 2D coordinates onto a sphere creates flat triangles that cut through the surface. Geo Tiler solves this by:

1. Filling polygon interiors with evenly-distributed Fibonacci lattice points
2. Using stereographic projection to perform accurate 2D triangulation
3. Mapping the triangulation back to 3D coordinates on a unit sphere

For a deeper explanation of the algorithm, see [this article](https://tricolor-albacore-d39.notion.site/From-Coordinates-to-Continents-Creating-a-3D-Globe-from-2D-Coordinates-1f83a2e4815d809b9606c3b5be791506).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
geo_tiler = "0.1"
```

## Quick Start

```rust
use geo::{coord, LineString, Polygon};
use geo_tiler::{generate_polygon_feature_mesh, PolygonMeshData};

let polygon = Polygon::new(
    LineString::new(vec![
        coord! {x: -30.0, y: -15.0},
        coord! {x: 0.0, y: 30.0},
        coord! {x: 30.0, y: -15.0},
        coord! {x: -30.0, y: -15.0}, // closing coordinate
    ]),
    vec![],
);

let mesh: PolygonMeshData = generate_polygon_feature_mesh(&polygon)?;
// mesh.vertices: Vec<(f64, f64, f64)> - 3D points on unit sphere
// mesh.triangles: Vec<u32> - triangle indices [i0, i1, i2, j0, j1, j2, ...]
```

## API

### Mesh Generation

| Function | Description |
|----------|-------------|
| `generate_polygon_feature_mesh(&Polygon)` | Generates a complete triangulated 3D mesh from a geographic polygon |
| `get_mesh_points(&Polygon)` | Returns 3D Cartesian points (boundary + interior) without triangulation |

### Coordinate Conversion

| Function | Description |
|----------|-------------|
| `ll_to_cartesian(lon, lat)` | Converts longitude/latitude (degrees) to 3D Cartesian coordinates on a unit sphere |
| `stereographic_projection((x, y, z))` | Projects a 3D point to 2D using stereographic projection from the north pole |
| `rotate_points_to_south_pole(&Vec<(f64, f64, f64)>)` | Rotates points so their centroid aligns with the south pole |

### Tiling

| Function | Description |
|----------|-------------|
| `generate_grid(step)` | Creates a grid of tiles covering the Earth's surface with the given angular step (degrees) |
| `clip_polygon_to_tiles(&mut grid, &Polygon)` | Clips a polygon against all tiles, storing intersections |
| `clamp_polygons(&mut tiles)` | Fixes floating-point precision errors at tile boundaries |

### Utilities

| Function | Description |
|----------|-------------|
| `fibonacci_sphere(n)` | Generates `n` evenly-distributed points on a sphere using the Fibonacci spiral method |
| `densify_edges(&mut Polygon, max_distance)` | Subdivides polygon edges that exceed `max_distance` |

## Data Structures

```rust
/// Triangulated mesh output
pub struct PolygonMeshData {
    pub vertices: Vec<(f64, f64, f64)>,  // 3D points on unit sphere
    pub triangles: Vec<u32>,              // flattened triangle indices
}

/// A tile in the geographic grid
pub struct Tile {
    pub vertices: Polygon<f64>,          // tile boundary
    pub polygons: Vec<Polygon<f64>>,     // clipped polygon fragments
}
```

## Error Handling

All fallible functions return `Result<T, GeoTilerError>`. Error variants:

| Variant | Cause |
|---------|-------|
| `CoordinateRangeError` | Longitude outside [-180, 180] or latitude outside [-90, 90] |
| `ProjectionError` | Attempting to project from the north pole singularity |
| `InverseProjectionError` | Invalid input (NaN or infinite values) |
| `FibonacciError` | Invalid point count (zero or negative) |
| `RotationError` | Zero-magnitude centroid or undefined rotation axis |
| `EmptyPointSetError` | Empty input where points are required |
| `MeshGenerationError` | Polygon with fewer than 3 vertices |
| `GridGenerationError` | Invalid step size (zero, too large, or doesn't divide evenly) |
| `InvalidPolygonError` | Malformed polygon geometry |
| `TriangulationError` | Constrained Delaunay triangulation failure |

## Algorithm Pipeline

1. **Parse** polygon boundaries from GeoJSON or `geo` types
2. **Generate interior points** using Fibonacci sphere distribution, filtered to polygon interior
3. **Convert** all points to 3D Cartesian coordinates
4. **Rotate** points so the centroid is at the south pole (optimal for projection)
5. **Project** to 2D using stereographic projection
6. **Triangulate** using constrained Delaunay triangulation with boundary edges as constraints
7. **Output** 3D vertices and triangle indices

## References

- [Delaunay/Voronoi on a Sphere](https://www.redblobgames.com/x/1842-delaunay-voronoi-sphere/) – Red Blob Games
- [Evenly Distributing Points on a Sphere](https://extremelearning.com.au/how-to-evenly-distribute-points-on-a-sphere-more-effectively-than-the-canonical-fibonacci-lattice/) – Martin Roberts
- [Martinez-Rueda Polygon Clipping](https://liorsinai.github.io/mathematics/2025/01/11/bentley-ottman.html) – Lior Sinai
