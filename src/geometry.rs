use std::f64::consts::PI;
use std::f64::EPSILON;
use crate::GeoTilerError;
use nalgebra::{Rotation, Rotation3, Unit, Vector3};
use geo::{coord, Coord};

/// Converts geographic coordinates (longitude and latitude) from decimal degrees to 3D Cartesian coordinates
/// on a unit sphere.
///
/// This function transforms a point specified by longitude and latitude into the corresponding point 
/// on a unit sphere in 3D Cartesian space where the center of the sphere is at the origin (0,0,0).
///
/// # Arguments
///
/// * `longitude` - The longitude in decimal degrees (-180 to 180), where 0° is the prime meridian,
///   positive values are east, and negative values are west
/// * `latitude` - The latitude in decimal degrees (-90 to 90), where 0° is the equator,
///   90° is the north pole, and -90° is the south pole
///
/// # Returns
///
/// * `Ok((f64, f64, f64))` - A tuple of (x, y, z) Cartesian coordinates on the unit sphere
/// * `Err(GeoTilerError::CoordinateRangeError)` - An error if the longitude or latitude values are outside their valid ranges
///
/// # Mathematical formula
///
/// For coordinates in degrees:
/// * x = cos(latitude_rad) * cos(longitude_rad)
/// * y = cos(latitude_rad) * sin(longitude_rad)
/// * z = sin(latitude_rad)
///
/// Where latitude_rad = latitude * π/180 and longitude_rad = longitude * π/180
pub fn ll_to_cartesian(longitude: f64, latitude: f64) -> Result<(f64, f64, f64), GeoTilerError> {

    if longitude.abs() > (180.0 + 0.1)  || latitude.abs() > (90.0 + 0.1) { // return error if data is outside of reasonable floating point error
        return Err(GeoTilerError::CoordinateRangeError { longitude, latitude });
    }

    let epsilon: f64 = 1e-10;
    let (longitude, latitude) = sanitize_coordinates(longitude, latitude, epsilon);

    let longitude_rad: f64 = longitude * PI / 180.0;
    let latitude_rad: f64 = latitude * PI / 180.0;
    
    let x: f64 = latitude_rad.cos() * longitude_rad.cos();
    let y: f64 = latitude_rad.cos() * longitude_rad.sin();
    let z: f64 = latitude_rad.sin();
    
    Ok((x, y, z))
}  


/// Projects a point from the unit sphere in 3D space onto a 2D plane using stereographic projection.
///
/// Stereographic projection maps points from a sphere to a plane, preserving angles but not areas.
/// This implementation projects from the north pole (0, 0, 1) onto the plane z = 0.
/// Every point on the sphere except the north pole itself has a unique corresponding point on the plane.
///
/// # Arguments
///
/// * `point` - A 3D point (x, y, z) on or near the unit sphere
///
/// # Returns
///
/// * `Ok(Coord<f64>)` - A 2D point (x_2d, y_2d) representing the projected coordinates on the plane
/// * `Err(GeoTilerError::ProjectionError)` - An error if the point is at or very close to the north pole
///
/// # Mathematical formula
///
/// For a 3D point (x, y, z) on the unit sphere, the projected 2D point (x_2d, y_2d) is:
/// * x_2d = x / (1 - z)
/// * y_2d = y / (1 - z)
pub fn stereographic_projection(point: (f64, f64, f64)) -> Result<Coord<f64>, GeoTilerError> {
    let (x, y, z) = point;

    // check if point is at or very close to the north pole
    if (z - 1.0).abs() < f64::EPSILON {
        return Err(GeoTilerError::ProjectionError("Cannot project from the north pole (0, 0, 1)".to_string()));
    }

    let x_2d: f64 = x / (1.0 - z);
    let y_2d: f64 = y / (1.0 - z);

    Ok(coord! {x: x_2d, y: y_2d})
}

/// Rotates a set of 3D points on a unit sphere so that their centroid aligns with the south pole.
///
/// This function calculates the center point of the provided set of 3D points, then creates a rotation
/// that maps this center to the south pole (0, 0, -1). All points in the set are then rotated using
/// this same rotation matrix.
///
/// # Arguments
///
/// * `points` - A vector of 3D points `(x, y, z)` on or near the unit sphere
///
/// # Returns
///
/// * `Ok(Vec<(f64, f64, f64)>)` - A vector containing the rotated 3D points
/// * `Err(GeoTilerError)` - An error if the rotation cannot be performed:
///   - `EmptyPointSetError` if the input vector is empty
///   - `RotationError` if the centroid of points is too close to zero (evenly distributed points)
///   - `RotationError` if a rotation axis cannot be found (when points are at the north pole, 
///     on the equator, or in other special configurations)
pub fn rotate_points_to_south_pole(points: &Vec<(f64, f64, f64)>) -> Result<Vec<(f64, f64, f64)>, GeoTilerError> {
    if points.is_empty() {
        return Err(GeoTilerError::EmptyPointSetError("Cannot rotate an empty set of points".to_string()));
    }

    let mut center = Vector3::new(0.0, 0.0, 0.0);
    
    for (x, y, z) in points.iter() {
        center.x += x;
        center.y += y;
        center.z += z;
    }
    center /= points.len() as f64;

    // check if center is too small to normalize (should only happen if there's an even distribution of points in the set all over the sphere)
    if center.magnitude() < EPSILON {
        return Err(GeoTilerError::RotationError("Points centroid is effectively zero; cannot determine rotation direction".to_string()));
    }

    let center = Unit::new_normalize(center);

    let south_pole = Vector3::new(0.0, 0.0, -1.0);

    // make rotation object which defines rotation between center of polygon and south pole
    let rotation: Rotation<f64, 3> = match Rotation3::rotation_between(&center, &south_pole) {
        Some(rotation) => rotation,
        None => return Err(GeoTilerError::RotationError("Failed to compute rotation between points centroid and south pole".to_string())),
    };

    let mut rotated_points: Vec<(f64, f64, f64)> = Vec::with_capacity(points.len());
    for point in points.iter() {
        let p = rotation * Vector3::new(point.0, point.1, point.2);
        rotated_points.push((p.x, p.y, p.z));
    }

    Ok(rotated_points)
}


/// Helper function to sanitizes geographic coordinates to ensure they fall within valid ranges.
/// 
/// This function corrects coordinates that are just slightly outside the valid ranges
/// due to floating-point precision issues. It clamps longitude to [-180, 180] and 
/// latitude to [-90, 90] if they're within a small epsilon of the boundaries.
///
/// # Arguments
///
/// * `longitude` - The longitude in decimal degrees 
/// * `latitude` - The latitude in decimal degrees
/// * `epsilon` - The tolerance for floating-point comparisons (default: 1e-10)
///
/// # Returns
///
/// * `(f64, f64)` - A tuple of sanitized (longitude, latitude) coordinates
fn sanitize_coordinates(longitude: f64, latitude: f64, epsilon: f64) -> (f64, f64) {
    let sanitized_longitude: f64 = 
        if longitude > 180.0 && longitude <= 180.0 + epsilon {
            180.0
        } else if longitude < -180.0 && longitude >= -180.0 - epsilon {
            -180.0
        } else {
            longitude
        };
    
    let sanitized_latitude: f64 = 
    if latitude > 90.0 && latitude <= 90.0 + epsilon {
        90.0
    } else if latitude < -90.0 && latitude >= -90.0 - epsilon {
        -90.0
    } else {
        latitude
    };
    
    (sanitized_longitude, sanitized_latitude)
}