use std::error::Error;
use pgrx::prelude::*;
use pgrx::pg_sys::Point;
use geohash::{encode, decode, neighbor, neighbors, Direction, Coord};

/// Encode a coordinate to geohash with default precision of 12
#[pg_extern]
fn geohash_encode(point: Point) -> Result<String, Box<dyn Error + Send + Sync>> {
    let coord = Coord { x: point.x, y: point.y };
    encode(coord, 12).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

/// Encode a coordinate to geohash with specified precision
#[pg_extern]
fn geohash_encode_with_precision(
    point: Point, 
    precision: i32
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if precision < 1 || precision > 12 {
        return Err("Precision must be between 1 and 12".into());
    }
    
    let coord = Coord { x: point.x, y: point.y };
    encode(coord, precision as usize).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

/// Decode a geohash string to a coordinate point
#[pg_extern]
fn geohash_decode(hash_str: String) -> Result<Point, Box<dyn Error + Send + Sync>> {
    let (coord, _, _) = decode(&hash_str).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    Ok(Point {
        x: coord.x,
        y: coord.y,
    })
}

/// Find neighboring geohash for the given geohash and direction
/// Direction: 0=North, 1=NorthEast, 2=East, 3=SouthEast, 4=South, 5=SouthWest, 6=West, 7=NorthWest
#[pg_extern]
fn geohash_neighbor(
    hash_str: String, 
    direction: i32
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let dir = match direction {
        0 => Direction::N,
        1 => Direction::NE,
        2 => Direction::E,
        3 => Direction::SE,
        4 => Direction::S,
        5 => Direction::SW,
        6 => Direction::W,
        7 => Direction::NW,
        _ => return Err("Invalid direction. Must be 0-7 (N, NE, E, SE, S, SW, W, NW)".into()),
    };
    
    neighbor(&hash_str, dir).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}

/// Get all neighboring geohashes for the given geohash
#[pg_extern]
fn geohash_neighbors(hash_str: String) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let neighbors_result = neighbors(&hash_str).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    
    Ok(vec![
        neighbors_result.n,
        neighbors_result.ne,
        neighbors_result.e,
        neighbors_result.se,
        neighbors_result.s,
        neighbors_result.sw,
        neighbors_result.w,
        neighbors_result.nw,
    ])
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use pgrx::pg_sys::Point;

    #[pg_test]
    fn test_geohash_encode_via_spi() {
        let result = Spi::get_one::<String>(
            "SELECT geohash_encode(point(-5.60302734375, 42.60498046875))"
        ).expect("SPI result should not be NULL").unwrap();
        
        // Should start with expected prefix for the given coordinates
        assert!(result.starts_with("ezs42"));
        assert_eq!(result.len(), 12); // Default precision
    }

    #[pg_test]
    fn test_geohash_encode_precision_via_spi() {
        let result = Spi::get_one::<String>(
            "SELECT geohash_encode_with_precision(point(-5.60302734375, 42.60498046875), 5)"
        ).expect("SPI result should not be NULL").unwrap();
        
        assert_eq!(result, "ezs42");
    }

    #[pg_test]
    fn test_geohash_decode_via_spi() {
        let result = Spi::get_one::<Point>(
            "SELECT geohash_decode('ezs42')"
        ).expect("SPI result should not be NULL").unwrap();
        
        // Check coordinates are approximately correct (within reasonable tolerance)
        assert!((result.x - (-5.60302734375)).abs() < 0.1);
        assert!((result.y - 42.60498046875).abs() < 0.1);
    }

    #[pg_test]
    fn test_geohash_neighbor_via_spi() {
        let result = Spi::get_one::<String>(
            "SELECT geohash_neighbor('ezs42', 0)" // North
        ).expect("SPI result should not be NULL").unwrap();
        
        assert_eq!(result, "ezs48");
        
        let result_east = Spi::get_one::<String>(
            "SELECT geohash_neighbor('ezs42', 2)" // East
        ).expect("SPI result should not be NULL").unwrap();
        
        assert_eq!(result_east, "ezs43");
    }

    #[pg_test]
    fn test_geohash_neighbors_via_spi() {
        let result = Spi::get_one::<Vec<String>>(
            "SELECT geohash_neighbors('ezs42')"
        ).expect("SPI result should not be NULL").unwrap();
        
        assert_eq!(result.len(), 8); // Should have 8 neighbors
        assert!(result.contains(&"ezs48".to_string())); // North
        assert!(result.contains(&"ezs43".to_string())); // East
    }

    #[pg_test]
    #[should_panic(expected = "Invalid direction")]
    fn test_geohash_invalid_direction_via_spi() {
        Spi::get_one::<String>(
            "SELECT geohash_neighbor('ezs42', 8)" // Invalid direction
        ).expect("SPI call failed");
    }

    #[pg_test]
    #[should_panic]
    fn test_geohash_invalid_hash_via_spi() {
        Spi::get_one::<Point>(
            "SELECT geohash_decode('invalid_hash_123')"
        ).expect("SPI call failed");
    }

    #[pg_test]
    #[should_panic(expected = "Precision must be between 1 and 12")]
    fn test_geohash_invalid_precision_via_spi() {
        Spi::get_one::<String>(
            "SELECT geohash_encode_with_precision(point(0.0, 0.0), 13)" // Invalid precision
        ).expect("SPI call failed");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_geohash_encode_basic() {
        let point = Point { x: -5.60302734375, y: 42.60498046875 };
        let result = geohash_encode(point).unwrap();
        assert!(result.starts_with("ezs42"));
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn test_geohash_encode_with_precision_basic() {
        let point = Point { x: -5.60302734375, y: 42.60498046875 };
        let result = geohash_encode_with_precision(point, 5).unwrap();
        assert_eq!(result, "ezs42");
    }

    #[test]
    fn test_geohash_encode_with_precision_invalid() {
        let point = Point { x: 0.0, y: 0.0 };
        
        // Test precision too low
        assert!(geohash_encode_with_precision(point, 0).is_err());
        
        // Test precision too high
        assert!(geohash_encode_with_precision(point, 13).is_err());
    }

    #[test]
    fn test_geohash_decode_basic() {
        let result = geohash_decode("ezs42".to_string()).unwrap();
        assert!((result.x - (-5.60302734375)).abs() < 0.1);
        assert!((result.y - 42.60498046875).abs() < 0.1);
    }

    #[test]
    fn test_geohash_decode_invalid() {
        assert!(geohash_decode("invalid_hash".to_string()).is_err());
    }

    #[test]
    fn test_geohash_neighbor_all_directions() {
        let hash = "ezs42".to_string();
        
        // Test all valid directions
        for direction in 0..8 {
            let result = geohash_neighbor(hash.clone(), direction);
            assert!(result.is_ok(), "Direction {} should be valid", direction);
        }
        
        // Test invalid direction
        assert!(geohash_neighbor(hash, 8).is_err());
        assert!(geohash_neighbor("ezs42".to_string(), -1).is_err());
    }

    #[test]
    fn test_geohash_neighbor_specific_directions() {
        let hash = "ezs42".to_string();
        
        assert_eq!(geohash_neighbor(hash.clone(), 0).unwrap(), "ezs48"); // North
        assert_eq!(geohash_neighbor(hash.clone(), 2).unwrap(), "ezs43"); // East
    }

    #[test]
    fn test_geohash_neighbors_count() {
        let result = geohash_neighbors("ezs42".to_string()).unwrap();
        assert_eq!(result.len(), 8);
        
        // Check specific neighbors
        assert!(result.contains(&"ezs48".to_string())); // North
        assert!(result.contains(&"ezs43".to_string())); // East
    }

    #[test]
    fn test_geohash_round_trip() {
        let original_point = Point { x: 112.5584, y: 37.8324 };
        
        // Encode then decode
        let encoded = geohash_encode_with_precision(original_point, 8).unwrap();
        let decoded = geohash_decode(encoded).unwrap();
        
        // Check that we get back approximately the same coordinates
        assert!((decoded.x - original_point.x).abs() < 0.01);
        assert!((decoded.y - original_point.y).abs() < 0.01);
    }

    #[test]
    fn test_geohash_edge_cases() {
        // Test coordinates at extremes
        let north_pole = Point { x: 0.0, y: 90.0 };
        let south_pole = Point { x: 0.0, y: -90.0 };
        let date_line = Point { x: 180.0, y: 0.0 };
        let antimeridian = Point { x: -180.0, y: 0.0 };
        
        assert!(geohash_encode(north_pole).is_ok());
        assert!(geohash_encode(south_pole).is_ok());
        assert!(geohash_encode(date_line).is_ok());
        assert!(geohash_encode(antimeridian).is_ok());
    }

    #[test]
    fn test_geohash_precision_levels() {
        let point = Point { x: 0.0, y: 0.0 };
        
        // Test all valid precision levels
        for precision in 1..=12 {
            let result = geohash_encode_with_precision(point, precision).unwrap();
            assert_eq!(result.len(), precision as usize);
        }
    }
}