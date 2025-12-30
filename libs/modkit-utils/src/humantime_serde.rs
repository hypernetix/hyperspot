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

/// Reexport module.
pub mod re {
    pub use humantime;
}

use std::fmt;
use std::time::Duration;

use humantime;
use serde::{de, Deserializer, Serializer};

/// Deserializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
/// # Errors
/// Returns a `humantime::Error` if string is not a valid Duration
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
/// # Errors
/// Returns a `humantime::Error` if string is not a valid Duration
pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration_str = humantime::format_duration(*d).to_string();
    s.serialize_str(&duration_str)
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
    /// `deserialize_with` annotations.
    /// # Errors
    /// Returns a `humantime::Error` if string is not a valid Duration
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
    /// # Errors
    /// Returns a `humantime::Error` if string is not a valid Duration
    pub fn deserialize<'a, D>(d: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'a>,
    {
        Option::deserialize(d).and_then(|opt: Option<String>| {
            opt.map(|s| humantime::parse_duration(&s).map_err(serde::de::Error::custom))
                .transpose()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn with() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Duration::from_secs(15));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);
    }

    #[test]
    fn with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super::option", default)]
            time: Option<Duration>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(Duration::from_secs(15)));
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
            duration: Duration,
        }

        let json = r#"{"duration": "10m 10s"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, Duration::new(610, 0));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":"10m 10s"}"#);
    }

    #[test]
    fn time_with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super::option", default)]
            duration: Option<Duration>,
        }

        let json = r#"{"duration": "5m"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, Some(Duration::new(300, 0)));
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
            duration: Option<Duration>,
        }

        let json = r#"{"duration": "1m"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, Some(Duration::from_secs(60)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":"1m"}"#);

        let json = r#"{"duration": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.duration, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"duration":null}"#);
    }
}
