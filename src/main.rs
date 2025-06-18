use std::{env, fs::{self, File, OpenOptions}, path::Path, io::Write};
use geo::{coord, Coord, LineString, Polygon};
use geojson::{FeatureCollection, GeoJson, Geometry, PolygonType, Value};
use geo_tiler::{
        Tile, 
        PolygonMeshData, 
        generate_grid, 
        clip_polygon_to_tiles, 
        generate_polygon_feature_mesh, 
        clamp_polygons
    };



fn main() {
    /* get file path from args */
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <directory_path>", args[0]);
        std::process::exit(1);
    }
    let file_path: &str = &args[1];
    let dir_path: &str = &args[2];


    /* parse geojson */
    let file_content: String = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Failed to read GeoJSON file: {}", e);
        std::process::exit(1);
    });
    let geojson: GeoJson = file_content.parse().unwrap_or_else(|e| {
        eprintln!("Failed to parse GeoJson from file: {}", e);
        std::process::exit(1);
    });
    let features: FeatureCollection = FeatureCollection::try_from(geojson).unwrap_or_else(|e| {
        eprintln!("Failed to collect features from parsed GeoJson: {}", e);
        std::process::exit(1);
    });


    /* generate grid */
    let mut grid: Vec<Tile> = generate_grid(20).unwrap_or_else(|e| {
        eprintln!("Failed to generate grid: {}", e);
        std::process::exit(1);
    });


    /* clip every polygon */
    for feature in features {
        let geometry: &Geometry = feature.geometry.as_ref().unwrap_or_else(|| {
            eprintln!("Feature without a geometry: {}", feature);
            std::process::exit(1);
        });
        let polygon: &PolygonType = match &geometry.value {
            Value::Polygon(polygon) => polygon,
            _ => {
                eprintln!("Expected a Polygon as a geometry");
                std::process::exit(1);
            }
        };
        let outer_ring: Vec<Coord<f64>> = polygon[0]
            .iter()
            .map(|pos| coord! {x: pos[0], y: pos[1]})
            .collect();

        let polygon: Polygon = Polygon::new(LineString::new(outer_ring), vec![]);
        
        clip_polygon_to_tiles(&mut grid, &polygon).unwrap_or_else(|e| {
            eprintln!("Failed to clip polygon to grid: {}", e);
            std::process::exit(1);
        });
    }
    clamp_polygons(&mut grid); // needed for clipping floating number math inaccuracies

    /* obtain 3D coordinates for these polygons and save them */
    for tile in grid {
        let file_name: String = get_tile_file_name(&tile);
        let path: String = format!("{}/{}", dir_path, file_name);

        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("Failed to create directories: {}", e);
                std::process::exit(1);
            });
        }

        let mut file: File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to open file: {}", e);
                std::process::exit(1);
            });

        writeln!(file, "[\n").unwrap();
        for (i, polygon) in tile.polygons.iter().enumerate() {
            let mesh_data: PolygonMeshData = generate_polygon_feature_mesh(&polygon).unwrap_or_else(|e| {
                eprintln!("Failed to generate mesh from polygon: {}\n{}", e, &tile);
                std::process::exit(1);
            });
            let polygon_string: String = serde_json::to_string(&mesh_data).unwrap_or_else(|e| {
                eprintln!("Failed to serialize polygon: {}", e);
                std::process::exit(1);
            });
            
            if i == tile.polygons.len() - 1 {
                writeln!(file, "\t{}", polygon_string).unwrap();
            } else {
                writeln!(file, "\t{},", polygon_string).unwrap();
            }
        }
        writeln!(file, "\n]").unwrap();
    }
}

fn get_tile_file_name(tile: &Tile) -> String {
    let mut name: String = String::new();

    for vertex in tile.vertices.exterior() {
        name += format!("{},{};", vertex.x, vertex.y).as_str();
    }
    name.pop();
    name.push_str(".json");

    name
}