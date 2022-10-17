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