use std::{ops::{Deref, DerefMut}, fmt::Display};

use serde::{Serialize, Deserialize, de::Visitor};

// Bite me.
#[macro_export]
macro_rules! stringable {
    ($i:ident : $t:ty, $pi:ident, $pl:literal) => {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
        #[serde(into=$pl)]
        #[serde(try_from=$pl)]
        pub struct $i(pub $t);
        impl From<$i> for $pi {
            fn from(v:$i) -> $pi {
                $pi::String(v.0.to_string())
            }
        }
        impl TryFrom<$pi> for $i {
            type Error = <$t as std::str::FromStr>::Err;
            fn try_from(other: $pi) -> Result<$i, Self::Error> {
                match other {
                    $pi::String(val) => val.parse().map(|v| $i(v)),
                    $pi::Other(val) => Ok($i(val))
                }
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
        #[serde(untagged)]
        enum $pi {
            String(String),
            Other($t)
        }
    };
}

stringable!(StringBool: bool, BoolProxy, "BoolProxy");
stringable!(StringInteger: u64, IntegerProxy, "IntegerProxy");

pub struct StackString<const N: usize>([u8; N], usize); // If you use it for N>48 I'll kill you.
impl<const N: usize> StackString<N> {
    // Take &str to promise that it's valid utf8
    fn new(from: &str) -> Self {
        let len = from.len();
        let mut data = [0u8;N];
        let (left, _) = data.split_at_mut(len); // We already promise len <= N; use unchecked when stable
        left.copy_from_slice(from.as_bytes());
        StackString(data, len)
    }

    fn try_new(from: &str) -> Result<Self, ()> {
        if from.len() > N {
            Err(()) // It's too long.
        } else {
            Ok(Self::new(from))
        }
    }
}

impl<const N: usize> Copy for StackString<N> {}

impl<const N: usize> Deref for StackString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // I promise that this contains a viable string of bytes by contract.
        // This is fine for deref because this uses mem::transmute under the hood.
        unsafe {std::str::from_utf8_unchecked(&self.0[..self.1])}
    }
}

impl<const N: usize> DerefMut for StackString<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {std::str::from_utf8_unchecked_mut(&mut self.0[..self.1])}
    }
}

impl<const N: usize> AsRef<str> for StackString<N> {
    fn as_ref(&self) -> &str {
        &**self
    }
}

impl<const N: usize> AsMut<str> for StackString<N> {
    fn as_mut(&mut self) -> &mut str {
        &mut *self
    }
}

impl<const N: usize> std::fmt::Debug for StackString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StackString").field(&self.0).finish()
    }
}

impl<const N: usize> Display for StackString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Inherit Display impl from associated str
        <Self as AsRef<str>>::as_ref(self).fmt(f)
    }
}

impl<const N: usize> Serialize for StackString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&**self)
    }
}

impl<const N: usize> PartialOrd for StackString<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        <Self as AsRef<str>>::as_ref(self).partial_cmp(other.as_ref())
    }
}

impl<const N: usize> Ord for StackString<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        <Self as AsRef<str>>::as_ref(self).cmp(other.as_ref())
    }
}

impl<const N: usize> Clone for StackString<N> {
    fn clone(&self) -> Self {
        *self // We implement Copy
    }
}

impl<const N: usize> PartialEq for StackString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0[..self.1] == other.0[..other.1]
    }
}

impl<const N: usize> Eq for StackString<N> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<const N: usize> std::hash::Hash for StackString<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Self as AsRef<str>>::as_ref(self).hash(state);
    }
}

impl<const N: usize> Default for StackString<N> {
    fn default() -> Self {
        StackString([0; N], 0)
    }
}

impl<const N: usize> From<&str> for StackString<N> {
    fn from(v: &str) -> Self {
        StackString::new(v)
    }
}

struct StackStringVisitor<const N: usize>();
impl<'de, const N: usize> Visitor<'de> for StackStringVisitor<N> {
    type Value = StackString<N>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        StackString::try_new(v).map_err(|_|E::custom("deserialized string is too long"))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string of N length")
    }
}

impl<'de, const N: usize> Deserialize<'de> for StackString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_str(StackStringVisitor::<N>())
    }
}

pub(crate) mod timestamp {
    use chrono::{DateTime, Utc, NaiveDateTime};
    use serde::{Serializer, de::Visitor, Deserializer};

    // Why not NaiveDateTime? 
    // The server has a timezone in mind when it sends these timestamps.
    // Converting to/from it is important for consumers.
    pub type Timestamp = DateTime<Utc>;

    pub fn serialize<S>(timestamp: &Timestamp, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u64(timestamp.timestamp() as u64)
    }

    struct TimestampVisitor;
    impl<'de> Visitor<'de> for TimestampVisitor {
        type Value = Timestamp;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("unix timestamp (UTC)")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error, {
            Ok(DateTime::from_utc(NaiveDateTime::from_timestamp(v, 0), Utc))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error, {
            // Okay, it's i64. Still...
            Ok(DateTime::from_utc(NaiveDateTime::from_timestamp(v as i64, 0), Utc))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error, {
            // Floats lose their precision long before they represent values larger than i64 max value
            Ok(DateTime::from_utc(NaiveDateTime::from_timestamp(v as i64, 0), Utc))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_i64(TimestampVisitor)
    }
}