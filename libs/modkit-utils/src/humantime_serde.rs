#![forbid(unsafe_code)]

//! Serde support for the `humantime` crate.
//!
//! Based on [this fork](https://github.com/jean-airoldie/humantime-serde).
//!
//! Currently `std::time::Duration` is supported.
//!
//! # Example
//! ```
//! use serde::{Serialize, Deserialize};
//! use std::time::Duration;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Foo {
//!     #[serde(with = "modkit_utils::humantime_serde")]
//!     timeout: Duration,
//! }
//! ```

use std::fmt;
use std::time::Duration;

use humantime;
use serde::{Deserializer, Serializer, de};

/// Deserializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
///
/// # Errors
///
/// Returns an error if the string is not a valid duration.
pub fn deserialize<'a, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'a>,
{
    struct V;

    impl de::Visitor<'_> for V {
        type Value = Duration;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a duration")
        }

        fn visit_str<E>(self, v: &str) -> Result<Duration, E>
        where
            E: de::Error,
        {
            humantime::parse_duration(v)
                .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
        }
    }

    d.deserialize_str(V)
}

/// Serializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `serialize_with` annotations.
///
/// # Errors
/// None
pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.collect_str(&humantime::format_duration(*d))
}

pub mod option {
    //! Convenience module to allow serialization via `humantime_serde` for `Option`
    //!
    //! # Example
    //!
    //! ```
    //! use serde::{Serialize, Deserialize};
    //! use std::time::Duration;
    //!
    //! #[derive(Serialize, Deserialize)]
    //! struct Foo {
    //!     #[serde(with = "modkit_utils::humantime_serde::option")]
    //!     timeout: Option<Duration>,
    //! }
    //! ```

    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes an `Option<Duration>`
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `serialize_with` annotations.
    ///
    /// # Errors
    /// None
    pub fn serialize<S>(d: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match d {
            Some(d) => super::serialize(d, s),
            None => s.serialize_none(),
        }
    }

    /// Deserialize an `Option<Duration>`
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `deserialize_with` annotations.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid duration.
    pub fn deserialize<'a, D>(d: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'a>,
    {
        struct Wrapper(Duration);

        impl<'de> Deserialize<'de> for Wrapper {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::deserialize(d).map(Wrapper)
            }
        }

        let v: Option<Wrapper> = Option::deserialize(d)?;
        Ok(v.map(|Wrapper(d)| d))
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    #[test]
    fn with() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: super::Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, super::Duration::from_secs(15));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);
    }

    #[test]
    fn with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super::option", default)]
            time: Option<super::Duration>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(super::Duration::from_secs(15)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":null}"#);

        let json = r"{}";
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
    }

    #[test]
    fn time() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            duration: super::Duration,
        }

        let json = r#"{"duration": "10m 10s"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, super::Duration::new(610, 0));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":"10m 10s"}"#);
    }

    #[test]
    fn time_with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super::option", default)]
            duration: Option<super::Duration>,
        }

        let json = r#"{"duration": "5m"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, Some(super::Duration::new(300, 0)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":"5m"}"#);

        let json = r#"{"duration": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":null}"#);

        let json = r"{}";
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, None);
    }

    #[test]
    fn test_option_module() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super::option")]
            duration: Option<super::Duration>,
        }

        let json = r#"{"duration": "1m"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, Some(super::Duration::from_secs(60)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":"1m"}"#);

        let json = r#"{"duration": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":null}"#);
    }

    #[test]
    fn test_expecting_message() {
        #[derive(Serialize, Deserialize, Debug)]
        struct Foo {
            #[serde(with = "super")]
            duration: super::Duration,
        }

        let json = r#"{"duration": 123}"#;
        let err = serde_json::from_str::<Foo>(json).unwrap_err();
        assert!(err.to_string().contains("expected a duration"));
    }

    #[test]
    fn test_invalid_string() {
        #[derive(Serialize, Deserialize, Debug)]
        struct Foo {
            #[serde(with = "super")]
            duration: super::Duration,
        }

        let json = r#"{"duration": "not a duration"}"#;
        let err = serde_json::from_str::<Foo>(json).unwrap_err();
        assert!(err.to_string().contains("expected a duration"));
    }
}
