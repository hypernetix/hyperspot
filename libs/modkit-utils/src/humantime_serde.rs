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
//!
//! Or use the `Serde` wrapper type:
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use modkit_utils::humantime_serde::Serde;
//! use std::time::Duration;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Foo {
//!     timeout: Vec<Serde<Duration>>,
//! }
//! ```

/// Reexport module.
pub mod re {
    pub use humantime;
}

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use humantime;
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

/// Deserializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
/// # Errors
/// Returns a `humantime::Error` if string is not a valid Duration
pub fn deserialize<'a, T, D>(d: D) -> Result<T, D::Error>
where
    Serde<T>: Deserialize<'a>,
    D: Deserializer<'a>,
{
    Serde::deserialize(d).map(Serde::into_inner)
}

/// Serializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `serialize_with` annotations.
/// # Errors
/// Returns a `humantime::Error` if string is not a valid Duration
pub fn serialize<T, S>(d: &T, s: S) -> Result<S::Ok, S::Error>
where
    for<'a> Serde<&'a T>: Serialize,
    S: Serializer,
{
    Serde::from(d).serialize(s)
}

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `Duration`.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(T);

impl<T> fmt::Debug for Serde<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Serde<T> {
    /// Consumes the `De`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Serde<T> {
    fn from(val: T) -> Serde<T> {
        Serde(val)
    }
}

impl<'de> Deserialize<'de> for Serde<Duration> {
    fn deserialize<D>(d: D) -> Result<Serde<Duration>, D::Error>
    where
        D: Deserializer<'de>,
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

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Duration>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Serde<Duration>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl ser::Serialize for Serde<&'_ Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        humantime::format_duration(*self.0)
            .to_string()
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        humantime::format_duration(self.0)
            .to_string()
            .serialize(serializer)
    }
}

impl ser::Serialize for Serde<&'_ Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self.0 {
            Some(dur) => serializer.serialize_some(&Serde(dur)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Serde(&self.0).serialize(serializer)
    }
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
    //!     #[serde(default)]
    //!     #[serde(with = "modkit_utils::humantime_serde::option")]
    //!     timeout: Option<Duration>,
    //! }
    //! ```

    use super::Serde;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes an `Option<Duration>`
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `deserialize_with` annotations.
    /// # Errors
    /// Returns a `humantime::Error` if string is not a valid Duration
    pub fn serialize<T, S>(d: &Option<T>, s: S) -> Result<S::Ok, S::Error>
    where
        for<'a> Serde<&'a T>: Serialize,
        S: Serializer,
    {
        let nested: Option<Serde<&T>> = d.as_ref().map(Into::into);
        nested.serialize(s)
    }

    /// Deserialize an `Option<Duration>`
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `deserialize_with` annotations.
    /// # Errors
    /// Returns a `humantime::Error` if string is not a valid Duration
    pub fn deserialize<'a, T, D>(d: D) -> Result<Option<T>, D::Error>
    where
        Serde<T>: Deserialize<'a>,
        D: Deserializer<'a>,
    {
        let got: Option<Serde<T>> = Deserialize::deserialize(d)?;
        Ok(got.map(Serde::into_inner))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            #[serde(with = "super", default)]
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
            #[serde(with = "super", default)]
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
            #[serde(with = "option")]
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

    #[test]
    fn test_serde_traits() {
        let mut s = Serde(Duration::from_secs(1));

        // Test Deref
        assert_eq!(*s, Duration::from_secs(1));

        // Test DerefMut
        *s = Duration::from_secs(2);
        assert_eq!(s.0, Duration::from_secs(2));

        // Test Debug
        let debug = format!("{s:?}");
        assert!(debug.contains("2s"));

        // Test From
        let from: Serde<Duration> = Duration::from_secs(3).into();
        assert_eq!(from.0, Duration::from_secs(3));
    }
}
