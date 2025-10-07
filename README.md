# pgdatatypes_plus

A PostgreSQL extension that provides additional data types, starting with the `emailaddr` type for storing and validating email addresses efficiently.

## Overview

The `emailaddr` type is a PostgreSQL custom data type that:
- Validates email addresses on input using RFC-compliant validation
- Stores email addresses efficiently
- Provides natural ordering (lexicographic comparison)
- Supports indexing for improved query performance
- Is fully compatible with PostgreSQL's type system

## Features

- **Type Safety**: Strong typing prevents invalid email addresses from being stored
- **Indexing Support**: Full support for B-tree, Hash, and other index types
- **Cast Support**: Automatic casting between `emailaddr` and `text` types

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

## Limitations

1. **Validation Scope**: Uses standard email validation rules; may not cover all RFC 5321 edge cases
2. **Case Sensitivity**: Currently treats email addresses as case-sensitive (local part and domain)
3. **Internationalization**: Supports international domain names but may have limitations with some Unicode characters

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/isdaniel/pgdatatypes_plus/issues)
- **Repository**: [GitHub Repository](https://github.com/isdaniel/pgdatatypes_plus)

---

*Built with ❤️ using [pgrx](https://github.com/pgcentralfoundation/pgrx)*