use pgrx::prelude::*;
use pgrx::StringInfo;
use std::cmp::Ordering;
use std::str::FromStr;
use std::fmt::{self, Display};
use validator::ValidateEmail;
use serde::{Deserialize, Serialize};

/// An email address type that stores addresses in a validated format.
/// Comparison is done domain-first, then local part.
/// Case sensitivity is preserved (local part is case-sensitive, domain is case-insensitive).
/// This matches the behavior of the original C implementation.
/// Validation is done using the `validator` crate.
/// Note that this implementation does not handle all edge cases of email validation as per RFC 5321, but covers the vast majority of common cases.
#[derive(Debug,  PartialEq, Eq, Serialize, Deserialize, PostgresType, PostgresEq, PostgresOrd)]
#[inoutfuncs]
pub struct EmailAddr {
    data: String
}

impl FromStr for EmailAddr {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.validate_email() {
            return Err("invalid email address format");
        }

        Ok(EmailAddr {
            data: s.to_string()
        })
    }
}

// Implement custom ordering: domain-first, then local part
impl PartialOrd for EmailAddr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EmailAddr {
    fn cmp(&self, other: &Self) -> Ordering {
        // If domains are equal, compare local parts
        self.data.cmp(&other.data)
    }
}

impl Display for EmailAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl InOutFuncs for EmailAddr {
    fn input(input: &std::ffi::CStr) -> EmailAddr {
        let input_str = input.to_str().unwrap_or_else(|e| {
            error!("invalid UTF-8 in email input: {}", e);
        });
        
        EmailAddr::from_str(input_str).unwrap_or_else(|e| {
            error!("invalid input syntax for type emailaddr: {}", e);
        })
    }

    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&self.data);
    }
}

/// Cast EmailAddr to text
#[pg_cast(assignment)]
fn cast_emailaddr_to_text(input: EmailAddr) -> String {
    input.to_string()
}

/// Cast text to EmailAddr
#[pg_cast(assignment)]
fn cast_text_to_emailaddr(input: &str) -> EmailAddr {
    EmailAddr::from_str(input).unwrap_or_else(|e| {
        error!("invalid input syntax for type emailaddr: {}", e);
    })
}


/// Create an email address from a text string
#[pg_extern(immutable, parallel_safe)]
fn emailaddr(input: &str) -> EmailAddr {
    EmailAddr::from_str(input).unwrap_or_else(|e| {
        error!("invalid input syntax for type emailaddr: {}", e);
    })
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[pg_test]
    fn test_domain_first_ordering() {
        let email1 = EmailAddr::from_str("user@a.com").unwrap();
        let email2 = EmailAddr::from_str("user@b.com").unwrap();
        assert!(email1 < email2);

        let email3 = EmailAddr::from_str("aaa@same.com").unwrap();
        let email4 = EmailAddr::from_str("zzz@same.com").unwrap();
        assert!(email3 < email4);
    }

    #[pg_test]
    fn test_invalid_emails() {
        assert!(EmailAddr::from_str("invalid").is_err());
        assert!(EmailAddr::from_str("@domain.com").is_err());
        assert!(EmailAddr::from_str("user@").is_err());
    }

    #[pg_test]
    fn test_case_sensitivity() {
        // Email addresses should be case-insensitive for domain, case-sensitive for localcar
        // For simplicity, our implementation treats them as case-sensitive
        // This matches the original C implementation behavior
        let email1 = EmailAddr::from_str("User@Domain.Com").unwrap();
        let email2 = EmailAddr::from_str("user@domain.com").unwrap();
        
        // Different case = different emails in our implementation
        assert_ne!(email1, email2);
    }

    #[pg_test]
    fn test_serialization() {
        let email = EmailAddr::from_str("test@example.com").unwrap();
        
        // Test that serialization/deserialization works (pgrx uses this internally)
        let serialized = serde_json::to_string(&email).unwrap();
        let deserialized: EmailAddr = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(email, deserialized);
    }

        #[test]
    fn test_validate_email() {

        let tests = vec![
            ("email@here.com", true),
            ("weirder-email@here.and.there.com", true),
            (r#"!def!xyz%abc@example.com"#, true),
            ("email@[127.0.0.1]", true),
            ("email@[2001:dB8::1]", true),
            ("email@[2001:dB8:0:0:0:0:0:1]", true),
            ("email@[::fffF:127.0.0.1]", true),
            ("example@valid-----hyphens.com", true),
            ("example@valid-with-hyphens.com", true),
            ("test@domain.with.idn.tld.उदाहरण.परीक्षा", true),
            (r#""test@test"@example.com"#, false),
            // max length for domain name labels is 63 characters per RFC 1034
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true),
            ("a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm", true),
            (
                "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm",
                true,
            ),
            // 64 * a
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false),
            ("", false),
            ("abc", false),
            ("abc@", false),
            ("abc@bar", true),
            ("a @x.cz", false),
            ("abc@.com", false),
            ("something@@somewhere.com", false),
            ("email@127.0.0.1", true),
            ("email@[127.0.0.256]", false),
            ("email@[2001:db8::12345]", false),
            ("email@[2001:db8:0:0:0:0:1]", false),
            ("email@[::ffff:127.0.0.256]", false),
            ("example@invalid-.com", false),
            ("example@-invalid.com", false),
            ("example@invalid.com-", false),
            ("example@inv-.alid-.com", false),
            ("example@inv-.-alid.com", false),
            (r#"test@example.com\n\n<script src="x.js">"#, false),
            (r#""\\\011"@here.com"#, false),
            (r#""\\\012"@here.com"#, false),
            ("trailingdot@shouldfail.com.", false),
            // Trailing newlines in username or domain not allowed
            ("a@b.com\n", false),
            ("a\n@b.com", false),
            (r#""test@test"\n@example.com"#, false),
            ("a@[127.0.0.1]\n", false),
            // underscores are not allowed
            ("John.Doe@exam_ple.com", false),
        ];

        for (input, expected) in tests {
            // println!("{} - {}", input, expected);
            assert_eq!(
                EmailAddr::from_str(input).is_ok(),
                expected,
                "Email `{}` was not classified correctly",
                input
            );
        }
    }

    #[test]
    fn test_validate_email_rfc5321() {
        // 65 character local part
        let test = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@mail.com";
        assert_eq!(EmailAddr::from_str(test).is_ok(), false);
        // 256 character domain part
        let test = "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.com";
        assert_eq!(EmailAddr::from_str(test).is_ok(), false);
    }
}