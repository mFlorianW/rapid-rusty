use crate::serde::duration;
use chrono::Duration;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{self, Deserializer, Serializer};
use std::fmt;

const FORMAT: &str = "%H:%M:%S%.3f";

pub fn serialize<S>(durations: &Vec<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(durations.len()))?;
    for dur in durations {
        let s = duration::duration_to_string::<S>(dur)?;
        seq.serialize_element(&s)?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVecVisitor;

    impl<'de> Visitor<'de> for DurationVecVisitor {
        type Value = Vec<Duration>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of duration strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<Duration>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut durations = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                let d = duration::deserialize(serde::de::IntoDeserializer::into_deserializer(s))?;
                durations.push(d);
            }
            Ok(durations)
        }
    }

    deserializer.deserialize_seq(DurationVecVisitor)
}
