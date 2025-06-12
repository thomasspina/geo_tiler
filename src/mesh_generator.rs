use d3_geo_rs::polygon_contains::polygon_contains;
use geo::{coord, Coord, HasDimensions, LineString, Polygon};
use ghx_constrained_delaunay::{
    constrained_triangulation::ConstrainedTriangulationConfiguration, constrained_triangulation_from_2d_vertices, types::{Edge, Vertex2d}, Triangulation
};
use crate::{
    fibonacci_sphere, 
    ll_to_cartesian, 
    rotate_points_to_south_pole, 
    stereographic_projection, 
    GeoTilerError
};


const DEFAULT_FIBONACCI_POINT_COUNT: usize = 3000;

/// Represents the geometric data for a triangulated polygon mesh on a sphere.
///
/// This structure stores both the vertices (as 3D Cartesian coordinates) and the triangulation
/// information that defines the mesh. The triangles are represented as indices into the vertices array.
///
/// # Fields
///
/// * `vertices` - 3D points forming the mesh in Cartesian coordinates (x, y, z).
///   Each vertex is a tuple of (f64, f64, f64) representing a point on a unit sphere.
///
/// * `triangles` - Triangle indices for the mesh, flattened as [i1, i2, i3, j1, j2, j3, ...].
///   Each consecutive triplet of indices defines one triangle by referencing vertices in the
///   `vertices` field.
#[derive(Debug, Clone)]
pub struct PolygonMeshData {
    /// 3D points forming the mesh (x, y, z coordinates)
    pub vertices: Vec<(f64, f64, f64)>,
    
    /// Triangle indices for the mesh, flattened as [i1, i2, i3, j1, j2, j3, ...]
    pub triangles: Vec<u32>,
}

/// Generates a triangulated 3D mesh from a 2D geographic polygon using constrained Delaunay triangulation.
///
/// This function creates a spherical mesh representation of a geographic polygon by:
/// 1. Extracting boundary points from the polygon and filling the interior with Fibonacci sphere points
/// 2. Converting all points to 3D Cartesian coordinates on a unit sphere
/// 3. Applying stereographic projection for 2D triangulation
/// 4. Performing constrained Delaunay triangulation to generate the mesh
/// 5. Returning both the 3D vertices and triangle connectivity information
///
/// The resulting mesh preserves the polygon's boundary as constrained edges while efficiently
/// triangulating the interior using a mathematically optimal point distribution.
///
/// # Arguments
///
/// * `polygon` - A geographic polygon with coordinates in decimal degrees (longitude, latitude).
///               The polygon must have at least 3 boundary points and cannot be empty.
///
/// # Returns
///
/// * `Ok(PolygonMeshData)` - Contains:
///   - `vertices`: 3D Cartesian coordinates (x, y, z) of all mesh points on the unit sphere
///   - `triangles`: Flattened triangle indices [i1, i2, i3, j1, j2, j3, ...] referencing the vertices
///
/// * `Err(GeoTilerError)` - Returns an error if:
///   - The polygon is empty or has fewer than 3 boundary points
///   - Coordinate conversion fails (invalid longitude/latitude values)
///   - Stereographic projection fails
///   - Constrained Delaunay triangulation fails
pub fn generate_polygon_feature_mesh(polygon: &Polygon) -> Result<PolygonMeshData, GeoTilerError> {
    let num_points: usize = polygon.exterior().points().len();

    let mesh_points: Vec<(f64, f64, f64)> = get_mesh_points(polygon)?;

    // calculate edges for outer ring
    let mut edges: Vec<Edge> = Vec::with_capacity(num_points);
    for i in (0..num_points).rev() {
        let edge: Edge = Edge {
            from: i as u32,
            to: ((i + num_points - 1) % num_points) as u32
        };
        edges.push(edge);
    }

    // rotate points to south pole for better stereographic projection
    let rotated_points: Vec<(f64, f64, f64)> = rotate_points_to_south_pole(&mesh_points)?;

    // do a stereographic projection
    let mut projected_points: Vec<CoordVertex<f64>> = Vec::new();
    for point in rotated_points {
        let projected_point: Coord<f64> = stereographic_projection(point)?;

        let projected_point: CoordVertex<f64> = CoordVertex { x: projected_point.x, y: projected_point.y };

        projected_points.push(projected_point);
    }
    

    let config: ConstrainedTriangulationConfiguration = ConstrainedTriangulationConfiguration {
        bin_vertex_density_power: 1.0,
    };

    // generate mesh triangles using constrained delaunay triangulation
    let delaunay_triangles: Triangulation = match constrained_triangulation_from_2d_vertices(&projected_points, &edges, config) {
        Ok(triangles) => triangles,
        Err(err) => return Err(GeoTilerError::TriangulationError(format!("Failed to generate triangulation: {}", err)))
    };

    let flattened_delaunay: Vec<u32> = delaunay_triangles.triangles.into_iter()
        .flat_map(|triangle| triangle.into_iter())
        .collect();
    
    Ok(PolygonMeshData {
        vertices: mesh_points,
        triangles: flattened_delaunay
    })
}

/// Generates a set of 3D mesh points from a geographic polygon by combining the polygon's
/// boundary points with interior points generated using a Fibonacci sphere distribution.
///
/// This function takes an outer ring of a polygon defined by longitude and latitude coordinates,
/// fills it with points from a Fibonacci sphere distribution, and converts all points to 3D
/// Cartesian coordinates on a unit sphere.
///
/// # Arguments
///
/// * `outer_ring` - A vector of (longitude, latitude) pairs in decimal degrees that define the boundary
///                 of the polygon. Longitude should be in the range [-180, 180] and latitude in [-90, 90].
///
/// # Returns
///
/// * `Ok(Vec<(f64, f64, f64)>)` - A vector of 3D Cartesian coordinates representing the mesh points
///                                (both boundary and interior)
/// * `Err(String)` - An error message if the mesh generation cannot be performed
pub fn get_mesh_points(polygon: &Polygon) -> Result<Vec<(f64, f64, f64)>, GeoTilerError> {
    if polygon.exterior().is_empty() {
        return Err(GeoTilerError::EmptyPointSetError("Outer ring cannot be empty".to_string()));
    }

    if polygon.exterior().points().len() < 3 {
        return Err(GeoTilerError::MeshGenerationError("Outer ring must have at least 3 points to form a valid polygon".to_string()));
    }

    // get fibonacci points
    let fibonacci_points: Vec<Coord<f64>> = fibonacci_sphere(DEFAULT_FIBONACCI_POINT_COUNT)?;
    let mut mesh_points_2d: Vec<Coord<f64>> = polygon.exterior().0.clone();
    let outer_ring: [LineString; 1] = [polygon.exterior().clone()];
    for point in fibonacci_points {

        // keep fibonacci points which are contained in the shape
        if polygon_contains(&outer_ring, &point) {
            mesh_points_2d.push(coord! {x: point.x.to_degrees(), y: point.y.to_degrees()});
        }
    }

    let mut mesh_points_3d: Vec<(f64, f64, f64)> = Vec::new();
    for point in mesh_points_2d {
        let point_3d: (f64, f64, f64) = ll_to_cartesian(point.x, point.y)?;
        mesh_points_3d.push(point_3d);
    }

    Ok(mesh_points_3d) 
}

/// Wrapper for 2D coordinates that implements Vertex2d trait.
/// Needed because we can't implement external traits on geo::Coord due to orphan rule.
#[derive(Debug, Clone, Copy)]
struct CoordVertex<T> {
    x: T,
    y: T
}

impl Vertex2d for CoordVertex<f64> {
    fn x(self) -> f64 {
        self.x
    }

    fn y(self) -> f64 {
        self.y
    }
}