use pgrx::prelude::*;
use pgrx::StringInfo;
use std::cmp::Ordering;
use std::str::FromStr;
use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};

/// A Taiwan National ID type that stores Taiwan identification numbers in a validated format.
/// 
/// Taiwan National ID format: 1 letter (region code) + 9 digits
/// Taiwan National ID format: 1 letter (region code) + 9 digits
/// - First letter: Region code (A-Z, excluding I and O in original format)
/// - Second digit: Gender/status code (1=male, 2=female, 8-9 for new format)
/// - Digits 3-9: Sequential number
/// - Last digit: Checksum digit
/// 
/// Validation follows the official Taiwan National ID checksum algorithm:
/// 1. Convert region letter to corresponding number
/// 2. Apply coefficients [1, 8, 7, 6, 5, 4, 3, 2, 1, 1] to all 10 digits
/// 3. Sum all products
/// 4. Valid if sum is divisible by 10
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, PostgresType, PostgresEq, PostgresOrd)]
#[inoutfuncs]
pub struct Twid {
    data: String,
}

impl FromStr for Twid {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_valid_taiwan_id(s) {
            return Err("invalid Taiwan National ID format");
        }

        Ok(Twid {
            data: s.to_uppercase(),
        })
    }
}

impl PartialOrd for Twid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Twid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl Display for Twid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl InOutFuncs for Twid {
    fn input(input: &std::ffi::CStr) -> Twid {
        let input_str = input.to_str().unwrap_or_else(|e| {
            error!("invalid UTF-8 in TWID input: {}", e);
        });
        
        Twid::from_str(input_str).unwrap_or_else(|e| {
            error!("invalid input syntax for type twid: {}", e);
        })
    }

    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&self.data);
    }
}

/// Cast TWID to text
#[pg_cast(assignment)]
fn cast_twid_to_text(input: Twid) -> String {
    input.to_string()
}

/// Cast text to TWID
#[pg_cast(assignment)]
fn cast_text_to_twid(input: &str) -> Twid {
    Twid::from_str(input).unwrap_or_else(|e| {
        error!("invalid input syntax for type twid: {}", e);
    })
}

/// Create a Taiwan National ID from a text string
#[pg_extern(immutable, parallel_safe)]
fn twid(input: &str) -> Twid {
    Twid::from_str(input).unwrap_or_else(|e| {
        error!("invalid input syntax for type twid: {}", e);
    })
}

/// Check if a string is a valid Taiwan National ID
#[pg_extern(immutable, parallel_safe)]
fn is_valid_twid(input: &str) -> bool {
    is_valid_taiwan_id(input)
}

/// Get the gender from a Taiwan National ID
/// Returns 'M' for male, 'F' for female, 'U' for unknown/other
#[pg_extern(immutable, parallel_safe)]
fn twid_gender(input: Twid) -> String {
    get_gender_from_twid(&input.data)
}

/// Get the region code from a Taiwan National ID
#[pg_extern(immutable, parallel_safe)]
fn twid_region(input: Twid) -> String {
    input.data.chars().next().unwrap_or('?').to_string()
}

/// Validates a Taiwan National ID according to the official algorithm
fn is_valid_taiwan_id(input: &str) -> bool {
    // Check basic format: 1 letter + 9 digits
    if input.len() != 10 {
        return false;
    }

    let chars: Vec<char> = input.to_uppercase().chars().collect();
    
    // First character must be a letter
    if !chars[0].is_ascii_alphabetic() {
        return false;
    }

    // Remaining 9 characters must be digits
    for &c in &chars[1..] {
        if !c.is_ascii_digit() {
            return false;
        }
    }
    
    // Validate gender code (second character)
    // Traditional format: 1=male, 2=female
    // Note: 8=male foreign national, 9=female foreign national (new format)
    let gender_char = chars[1];
    if !matches!(gender_char, '1' | '2' | '8' | '9') {
        return false;
    }

    // Convert region letter to number
    let region_code = match get_region_number(chars[0]) {
        Some(num) => num,
        None => return false,
    };

    // Extract digits
    let mut digits = Vec::with_capacity(10);
    
    // Add region code digits (split into tens and ones)
    digits.push(region_code / 10);
    digits.push(region_code % 10);
    
    // Add the remaining 9 digits
    for &c in &chars[1..] {
        digits.push(c.to_digit(10).unwrap() as u16);
    }

    // Apply Taiwan ID checksum algorithm
    // Weights: [1, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1]
    let coefficients = [1, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1];
    let sum: u16 = digits.iter()
        .zip(coefficients.iter())
        .map(|(digit, coeff)| digit * coeff)
        .sum();
    
    sum % 10 == 0
}

/// Maps Taiwan region letters to their corresponding numbers
/// Uses the official sequence: ABCDEFGHJKLMNPQRSTUVXYWZIO
fn get_region_number(region: char) -> Option<u16> {
    match region {
        'A' => Some(10), // 臺北市
        'B' => Some(11), // 臺中市
        'C' => Some(12), // 基隆市
        'D' => Some(13), // 臺南市
        'E' => Some(14), // 高雄市
        'F' => Some(15), // 新北市
        'G' => Some(16), // 宜蘭縣
        'H' => Some(17), // 桃園市
        'I' => Some(34), // 嘉義市
        'J' => Some(18), // 新竹縣
        'K' => Some(19), // 苗栗縣
        'L' => Some(20), // 臺中縣 (已併入臺中市，但編號保留)
        'M' => Some(21), // 南投縣
        'N' => Some(22), // 彰化縣
        'O' => Some(35), // 新竹市
        'P' => Some(23), // 雲林縣
        'Q' => Some(24), // 嘉義縣
        'R' => Some(25), // 臺南縣 (已併入臺南市，但編號保留)
        'S' => Some(26), // 高雄縣 (已併入高雄市，但編號保留)
        'T' => Some(27), // 屏東縣
        'U' => Some(28), // 花蓮縣
        'V' => Some(29), // 臺東縣
        'W' => Some(32), // 金門縣
        'X' => Some(30), // 澎湖縣
        'Y' => Some(31), // 陽明山管理局 (已廢除，但編號保留)
        'Z' => Some(33), // 連江縣
        _ => None,
    }
}

/// Extracts gender information from Taiwan National ID
fn get_gender_from_twid(twid: &str) -> String {
    if twid.len() < 2 {
        return "U".to_string();
    }

    let second_char = twid.chars().nth(1).unwrap();
    match second_char {
        '1' => "M".to_string(), // Male (traditional format)
        '2' => "F".to_string(), // Female (traditional format)
        '8' => "M".to_string(), // Male (new format for foreign nationals)
        '9' => "F".to_string(), // Female (new format for foreign nationals)
        _ => "U".to_string(),   // Unknown/Other
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[pg_test]
    fn test_valid_taiwan_ids() {
        // Test known valid Taiwan IDs with correct checksums
        assert!(is_valid_taiwan_id("A123456789")); // From HackMD example
        assert!(is_valid_taiwan_id("F131232216")); // Valid F region ID
        
        // Test case insensitive
        assert!(is_valid_taiwan_id("a123456789"));
        assert!(is_valid_taiwan_id("f131232216"));
    }

    #[pg_test]
    fn test_invalid_taiwan_ids() {
        // Wrong length
        assert!(!is_valid_taiwan_id("A12345678"));
        assert!(!is_valid_taiwan_id("A1234567890"));
        
        // Invalid first character
        assert!(!is_valid_taiwan_id("1123456789"));
        assert!(!is_valid_taiwan_id("!123456789"));
        
        // Invalid region code
        assert!(!is_valid_taiwan_id("?123456789"));
        
        // Non-digit characters
        assert!(!is_valid_taiwan_id("A12345678A"));
        assert!(!is_valid_taiwan_id("AB23456789"));
        
        // Invalid gender codes
        assert!(!is_valid_taiwan_id("A323456789")); // 3 is not valid gender code
        assert!(!is_valid_taiwan_id("A523456789")); // 5 is not valid gender code
        
        // Empty string
        assert!(!is_valid_taiwan_id(""));
    }

    #[pg_test]
    fn test_twid_creation() {
        let twid = Twid::from_str("A123456789");
        assert!(twid.is_ok());
        
        let twid = Twid::from_str("F131232216");
        assert!(twid.is_ok());
        
        let twid = Twid::from_str("invalid");
        assert!(twid.is_err());
    }

    #[pg_test]
    fn test_twid_gender() {
        assert_eq!(get_gender_from_twid("A123456789"), "M");
        assert_eq!(get_gender_from_twid("A223456789"), "F");
        assert_eq!(get_gender_from_twid("A823456789"), "M");
        assert_eq!(get_gender_from_twid("A923456789"), "F");
        assert_eq!(get_gender_from_twid("A323456789"), "U");
    }

    #[pg_test]
    fn test_region_mapping() {
        // Test the official region sequence: ABCDEFGHJKLMNPQRSTUVXYWZIO
        assert_eq!(get_region_number('A'), Some(10)); // Position 0 + 10
        assert_eq!(get_region_number('B'), Some(11)); // Position 1 + 10
        assert_eq!(get_region_number('J'), Some(18)); // Position 8 + 10 (J is 9th letter)
        assert_eq!(get_region_number('I'), Some(34)); // Position 24 + 10 (I is 25th letter)
        assert_eq!(get_region_number('O'), Some(35)); // Position 25 + 10 (O is last)
        assert_eq!(get_region_number('?'), None);     // Invalid character
    }

    #[pg_test]
    fn test_case_insensitive() {
        let twid_lower = Twid::from_str("a123456789");
        let twid_upper = Twid::from_str("A123456789");
        
        assert!(twid_lower.is_ok());
        assert!(twid_upper.is_ok());
        assert_eq!(twid_lower.unwrap().data, twid_upper.unwrap().data);
    }
}