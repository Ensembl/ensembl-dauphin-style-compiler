use serde::ser::SerializeSeq;
use serde::{Deserialize, Serializer};
use serde::de::{self, SeqAccess };

pub fn de_seq_next<'de,S,T>(seq: &mut S) -> Result<T,S::Error> where S: SeqAccess<'de>, T: Deserialize<'de> {
    let out = seq.next_element::<T>()?;
    match out {
        Some(t) => Ok(t),
        None => Err(de::Error::custom(&"premature sqeuence termination"))
    }
}

pub fn de_wrap<'de,E,T,V>(value: Result<T,V>) -> Result<T,E> where V: ToString, E: de::Error {
    match value {
        Ok(out) => Ok(out),
        Err(e) => Err(de::Error::custom(e.to_string()))
    }
}

pub struct EnVarySeq(Vec<Box<dyn erased_serde::Serialize>>);

impl EnVarySeq {
    pub fn new() -> EnVarySeq { EnVarySeq(vec![]) }

    pub fn add(&mut self, e: Box<dyn erased_serde::Serialize>) {
        self.0.push(e);
    }

    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

#[macro_export]
macro_rules! envaryseq_addn {
    ($obj:expr,$($value:expr),*) => {
        {
            $($obj.add(Box::new($value));)*
        }
    };
}

#[macro_export]
macro_rules! envaryseq {
    ($serializer:expr,$($value:expr),*) => {
        {
            let mut seq = $crate::serde::EnVarySeq::new();
            $(seq.add(Box::new($value));)*
            seq.serialize($serializer)
        }
    }
}
