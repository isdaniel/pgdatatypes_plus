# pgdatatypes_plus

A PostgreSQL extension that provides additional data types and spatial functions for storing and validating specialized data efficiently. Currently includes `emailaddr` for email addresses, `twid` for Taiwan National IDs, and `geohash` functions for geospatial encoding.

## Overview

This extension provides specialized PostgreSQL data types and geospatial functions:

### EmailAddr Type
The `emailaddr` type is a PostgreSQL custom data type that:
- Validates email addresses on input using RFC-compliant validation
- Stores email addresses efficiently
- Provides natural ordering (lexicographic comparison)
- Supports indexing for improved query performance
- Is fully compatible with PostgreSQL's type system

### Taiwan National ID (TWID) Type
The `twid` type is a PostgreSQL custom data type that:
- Validates Taiwan National IDs using the official checksum algorithm
- Stores Taiwan National IDs in a standardized uppercase format
- Supports gender and region extraction
- Provides natural ordering (lexicographic comparison)
- Supports indexing for improved query performance
- Handles both traditional format (1=male, 2=female) and new format (8=male foreign national, 9=female foreign national)

### Geohash Functions
The extension provides comprehensive geohash functionality for spatial data encoding:
- **Encoding**: Convert latitude/longitude coordinates to geohash strings with configurable precision
- **Decoding**: Convert geohash strings back to coordinate points
- **Neighbor Finding**: Find adjacent geohashes in specific directions (N, NE, E, SE, S, SW, W, NW)
- **Spatial Indexing**: Efficient geospatial indexing and proximity queries using geohash algorithms
- **Precision Control**: Support for precision levels 1-12 for different spatial resolutions

## Features

- **Type Safety**: Strong typing prevents invalid email addresses and Taiwan National IDs from being stored
- **Indexing Support**: Full support for B-tree, Hash, and other index types for both data types
- **Cast Support**: Automatic casting between custom types and `text` types
- **Validation**: Built-in validation using official algorithms (RFC for emails, Taiwan government standard for National IDs)
- **Utility Functions**: Additional functions for extracting metadata (gender and region from TWID)
- **Geospatial Functions**: Comprehensive geohash encoding/decoding for efficient spatial data operations
## Installation

### Prerequisites

- PostgreSQL 13, 14, 15, 16, 17, or 18
- Rust toolchain
- `pgrx` (PostgreSQL Rust eXtension framework)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/isdaniel/pgdatatypes_plus.git
cd pgdatatypes_plus

# Install pgrx if not already installed
cargo install --locked cargo-pgrx

# Initialize pgrx for your PostgreSQL version
cargo pgrx init --pg13  # or pg14, pg15, pg16, pg17, pg18

# Install the extension
cargo pgrx install
```

### Enable the Extension

```sql
CREATE EXTENSION pgdatatypes_plus;
```

## Usage

### Basic Usage

#### Creating Tables with EmailAddr

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email emailaddr NOT NULL
);
```

#### Inserting Data

```sql
-- Single insert
INSERT INTO users (name, email) 
VALUES ('John Doe', 'john.doe@example.com');

-- Bulk insert example
INSERT INTO users (name, email)
SELECT 
    'User' || i::text,
    'user' || i::text || '@company.com'
FROM generate_series(1, 100000) i;
```

#### Querying Data

```sql
-- Basic select
SELECT * FROM users WHERE email = 'john.doe@example.com';

-- Pattern matching (cast to text for LIKE operations)
SELECT * FROM users WHERE email::text LIKE '%@gmail.com';

-- Ordering (emails are sorted lexicographically)
SELECT * FROM users ORDER BY email;

-- Range queries
SELECT * FROM users 
WHERE email BETWEEN 'a@domain.com' AND 'z@domain.com';
```

The `emailaddr` type automatically validates email addresses on input:

```sql
-- This will work
INSERT INTO users (name, email) VALUES ('Valid User', 'user@domain.com');

-- This will raise an error
INSERT INTO users (name, email) VALUES ('Invalid User', 'not-an-email');
-- ERROR: invalid input syntax for type emailaddr: invalid email address format
```

#### Type Casting

```sql
-- Explicit casting
SELECT email::text FROM users;
SELECT 'user@domain.com'::emailaddr;

-- Using the emailaddr() function
SELECT emailaddr('user@domain.com');
```

### Taiwan National ID (TWID) Usage

#### Creating Tables with TWID

```sql
CREATE TABLE citizens (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    national_id twid NOT NULL UNIQUE
);
```

#### Inserting Data

```sql
-- Single insert
INSERT INTO citizens (name, national_id) 
VALUES ('王小明', 'A123456789');

-- Bulk insert example
INSERT INTO citizens (name, national_id)
VALUES 
    ('陳美麗', 'F131232216'),
    ('李大華', 'B234567890'),
    ('張小芳', 'A287654321');
```

#### Querying Data

```sql
-- Basic select
SELECT * FROM citizens WHERE national_id = 'A123456789';

-- Pattern matching (cast to text for LIKE operations)
SELECT * FROM citizens WHERE national_id::text LIKE 'A%';

-- Ordering (IDs are sorted lexicographically)
SELECT * FROM citizens ORDER BY national_id;

-- Range queries
SELECT * FROM citizens 
WHERE national_id BETWEEN 'A000000000' AND 'A999999999';
```

#### TWID-Specific Functions

```sql
-- Check if a string is a valid Taiwan National ID
SELECT is_valid_twid('A123456789'); -- Returns true
SELECT is_valid_twid('A123456788'); -- Returns false (invalid checksum)
SELECT is_valid_twid('invalid');    -- Returns false

-- Extract gender from Taiwan National ID
SELECT twid_gender('A123456789'::twid); -- Returns 'M' (male)
SELECT twid_gender('A223456789'::twid); -- Returns 'F' (female)
SELECT twid_gender('A823456789'::twid); -- Returns 'M' (male foreign national)
SELECT twid_gender('A923456789'::twid); -- Returns 'F' (female foreign national)

-- Extract region code from Taiwan National ID
SELECT twid_region('A123456789'::twid); -- Returns 'A' (臺北市)
SELECT twid_region('F131232216'::twid); -- Returns 'F' (新北市)

-- Advanced query examples
SELECT name, national_id, twid_gender(national_id) as gender
FROM citizens 
WHERE twid_region(national_id) = 'A'  -- Taipei residents only
AND twid_gender(national_id) = 'F';   -- Female only
```

The `twid` type automatically validates Taiwan National IDs on input:

```sql
-- This will work
INSERT INTO citizens (name, national_id) VALUES ('Valid User', 'A123456789');

-- This will raise an error (invalid checksum)
INSERT INTO citizens (name, national_id) VALUES ('Invalid User', 'A123456788');
-- ERROR: invalid input syntax for type twid: invalid Taiwan National ID format

-- This will also raise an error (wrong format)
INSERT INTO citizens (name, national_id) VALUES ('Invalid User', 'not-a-twid');
-- ERROR: invalid input syntax for type twid: invalid Taiwan National ID format
```

#### TWID Type Casting

```sql
-- Explicit casting
SELECT national_id::text FROM citizens;
SELECT 'A123456789'::twid;

-- Using the twid() function
SELECT twid('A123456789');

-- Case insensitive input (automatically converted to uppercase)
SELECT twid('a123456789'); -- Stored as 'A123456789'
```

### Geohash Functions Usage

The extension provides comprehensive geohash functionality for encoding and working with geospatial data. Geohash is a geocoding system that represents geographic coordinates as short alphanumeric strings, making it ideal for spatial indexing and proximity queries.

#### Basic Geohash Operations

```sql
-- Encode coordinates to geohash with default precision (12 characters)
SELECT geohash_encode(point(-122.4194, 37.7749)); -- San Francisco
-- Returns: '9q8yyk8yugs8'

-- Encode with specific precision (1-12 characters)
SELECT geohash_encode_with_precision(point(-122.4194, 37.7749), 5);
-- Returns: '9q8yy'

-- Decode geohash back to coordinates
SELECT geohash_decode('9q8yy');
-- Returns: point(-122.4194, 37.7749) (approximately)
```

#### Geohash Neighbor Operations

```sql
-- Find neighboring geohash in a specific direction
-- Directions: 0=North, 1=NorthEast, 2=East, 3=SouthEast, 
--            4=South, 5=SouthWest, 6=West, 7=NorthWest
SELECT geohash_neighbor('9q8yy', 0); -- North neighbor
-- Returns: '9q8yz'

SELECT geohash_neighbor('9q8yy', 2); -- East neighbor
-- Returns: '9q8yv'

-- Get all 8 neighboring geohashes at once
SELECT geohash_neighbors('9q8yy');
-- Returns: ['{9q8yz,9q8yv,9q8ys,9q8yr,9q8yq,9q8yw,9q8yt,9q8yu}']
-- Order: [N, NE, E, SE, S, SW, W, NW]
```

#### Geohash Precision Levels

Different precision levels provide different spatial resolutions:

| Precision | Lat Error | Lng Error | Area Coverage |
|-----------|-----------|-----------|---------------|
| 1         | ±23°      | ±23°      | Continent     |
| 2         | ±2.8°     | ±5.6°     | Large Country |
| 3         | ±0.70°    | ±0.70°    | Country/State |
| 4         | ±0.087°   | ±0.18°    | Large City    |
| 5         | ±0.022°   | ±0.022°   | City District |
| 6         | ±0.0027°  | ±0.0055°  | Neighborhood  |
| 7         | ±0.00068° | ±0.00068° | City Block    |
| 8         | ±0.000085°| ±0.00017° | Building      |
| 9         | ±0.000021°| ±0.000021°| Building Room |
| 10        | ±0.0000027°| ±0.0000054°| Small Room   |
| 11        | ±0.00000067°| ±0.00000067°| Desk         |
| 12        | ±0.00000008°| ±0.000000017°| Person       |

```sql
-- Example: Find appropriate precision for different use cases
-- City-level search (precision 5)
SELECT geohash_encode_with_precision(point(-122.4194, 37.7749), 5);

-- Building-level search (precision 8)  
SELECT geohash_encode_with_precision(point(-122.4194, 37.7749), 8);

-- High-precision GPS tracking (precision 12)
SELECT geohash_encode_with_precision(point(-122.4194, 37.7749), 12);
```

### EmailAddr Type
1. **Validation Scope**: Uses standard email validation rules; may not cover all RFC 5321 edge cases
2. **Case Sensitivity**: Currently treats email addresses as case-sensitive (local part and domain)
3. **Internationalization**: Supports international domain names but may have limitations with some Unicode characters

### TWID Type
1. **Taiwan-Specific**: Only validates Taiwan National IDs according to Taiwan government standards
2. **Format Support**: Supports both traditional format (1=male, 2=female) and new format (8/9 for foreign nationals)
3. **Region Mapping**: Uses the official Taiwan region code mapping (A-Z excluding some letters)
4. **Case Handling**: Input is automatically converted to uppercase for consistency

### Geohash Functions
1. **Precision Range**: Supports precision levels 1-12; higher precision provides more accuracy but longer strings
2. **Coordinate Bounds**: Works with standard geographic coordinate ranges (latitude: -90 to 90, longitude: -180 to 180)
3. **Approximation**: Geohash encoding/decoding introduces small coordinate approximation errors based on precision level
4. **Spatial Accuracy**: Distance estimation using geohash similarity is approximate; use PostGIS for precise distance calculations
5. **Grid Boundaries**: Geohash grid cells have discrete boundaries; nearby points may have different geohashes if they cross cell boundaries

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/isdaniel/pgdatatypes_plus/issues)
- **Repository**: [GitHub Repository](https://github.com/isdaniel/pgdatatypes_plus)
