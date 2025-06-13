use std::{env, fs};
use geo::{coord, Coord, LineString, Polygon};
use geojson::{FeatureCollection, GeoJson, Geometry, PolygonType, Value};
use geo_tiler::{Tile, generate_grid, clip_polygon_to_tiles};



fn main() {
    /* get file path from args */
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let file_path: &str = &args[1];


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


    /* apply algorithm to each polygon */
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
        




    }

    
    
}