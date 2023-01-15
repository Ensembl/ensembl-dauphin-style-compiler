use std::{str::{Chars, from_utf8}, sync::{Arc}, fmt};
use peregrine_toolkit::{lesqlite2::lesqlite2_decode, serdetools::{st_field, ByteData}, log};
use serde::{Deserialize, Deserializer, de::{Visitor, self, IgnoredAny}};

use super::data::{ReceivedData, ReceivedDataType};

fn pop(data: &mut Vec<ReceivedData>) -> Result<ReceivedData,()> {
    data.pop().ok_or(())
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum NumberSourceAlgorithm {
    Array(ReceivedData),
    Lesqlite2(ReceivedData),
}

impl NumberSourceAlgorithm {
    fn new<'a>(code: &mut Chars<'a>, data: &mut Vec<ReceivedData>) -> Result<NumberSourceAlgorithm,()> {
        Ok(match code.next() {
            Some('A') => NumberSourceAlgorithm::Array(pop(data)?),
            Some('L') => NumberSourceAlgorithm::Lesqlite2(pop(data)?),
            _ => { return Err(()); }
        })
    }

    fn specify<'a>(code: &mut Chars<'a>, spec: &mut Vec<ReceivedDataType>) -> Result<(),()> {
         match code.next() {
            Some('A') => { spec.push(ReceivedDataType::Numbers); },
            Some('L') => { spec.push(ReceivedDataType::Bytes); },
            _ => { return Err(()); }
        }
        Ok(())
    }

    fn make<'a> (&self) -> Result<Arc<Vec<f64>>,()> {
        match self {
            NumberSourceAlgorithm::Array(data) => {
                Ok(data.data_as_numbers()?.clone())
            },
            NumberSourceAlgorithm::Lesqlite2(data) => {
                Ok(Arc::new(lesqlite2_decode(&data.data_as_bytes()?)?))
            }
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum NumberAlgorithm {
    Raw(NumberSourceAlgorithm),
    Zigzag(Box<NumberAlgorithm>),
    Delta(Box<NumberAlgorithm>)
}

impl NumberAlgorithm {
    fn new<'a>(code: &mut Chars<'a>, data: &mut Vec<ReceivedData>) -> Result<NumberAlgorithm,()> {
        Ok(match code.next() {
            Some('R') => NumberAlgorithm::Raw(NumberSourceAlgorithm::new(code,data)?),
            Some('Z') => NumberAlgorithm::Zigzag(Box::new(NumberAlgorithm::new(code,data)?)),
            Some('D') => NumberAlgorithm::Delta(Box::new(NumberAlgorithm::new(code,data)?)),
            _ => { return Err(()); }
        })
    }

    fn specify<'a>(code: &mut Chars<'a>, spec: &mut Vec<ReceivedDataType>) -> Result<(),()> {
        Ok(match code.next() {
            Some('R') => { NumberSourceAlgorithm::specify(code,spec)?; },
            Some('Z') => { NumberAlgorithm::specify(code,spec)?; },
            Some('D') => { NumberAlgorithm::specify(code,spec)?; },
            _ => { return Err(()); }
        })
    }

    fn make<'a>(&self) -> Result<Arc<Vec<f64>>,()> {
        match self {
            NumberAlgorithm::Raw(inner) => { 
                inner.make()
            },
            NumberAlgorithm::Zigzag(data) => {
                Ok(Arc::new(data.make()?.iter().map(|v| {
                    let v = *v as i64;
                    (if v%2 == 1 { -((v+1)/2) } else { v/2 }) as f64
                }).collect()))
            },
            NumberAlgorithm::Delta(data) => {
                let mut prev = 0.;
                Ok(Arc::new(data.make()?.iter().map(|v| {
                    prev += v;
                    prev
                }).collect()))
            }
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum StringAlgorithm {
    Array(ReceivedData),
    CharacterSplit(ReceivedData),
    ZeroSplit(ReceivedData),
    Classify(NumberAlgorithm,Box<StringAlgorithm>)
}

impl StringAlgorithm {
    fn new<'a>(code: &mut Chars<'a>, data: &mut Vec<ReceivedData>) -> Result<StringAlgorithm,()> {
        Ok(match code.next() {
            Some('A') => StringAlgorithm::Array(pop(data)?),
            Some('C') => StringAlgorithm::CharacterSplit(pop(data)?),
            Some('Z') => StringAlgorithm::ZeroSplit(pop(data)?),
            Some('Y') => {
                let index = NumberAlgorithm::new(code,data)?;
                let values = StringAlgorithm::new(code,data)?;
                StringAlgorithm::Classify(index,Box::new(values))
            },
            _ => { return Err(()); }
        })
    }

    fn specify<'a>(code: &mut Chars<'a>, spec: &mut Vec<ReceivedDataType>) -> Result<(),()> {
        match code.next() {
            Some('A') => { spec.push(ReceivedDataType::Strings); },
            Some('C') => { spec.push(ReceivedDataType::Bytes); },
            Some('Z') => { spec.push(ReceivedDataType::Bytes); },
            Some('Y') => {
                NumberAlgorithm::specify(code,spec)?;
                StringAlgorithm::specify(code,spec)?;
            },
            _ => { return Err(()); }
        }
        Ok(())
    }

    fn make<'a> (&self) -> Result<Arc<Vec<String>>,()> {
        match self {
            StringAlgorithm::Array(data) => {
                Ok(data.data_as_strings()?.clone())
            },
            StringAlgorithm::CharacterSplit(data) => {
                let bytes = data.data_as_bytes()?;
                let bytes = from_utf8(&bytes).map_err(|_| ())?;
                Ok(Arc::new(bytes.chars().map(|x| x.to_string()).collect()))
            },
            StringAlgorithm::ZeroSplit(data) => {
                let bytes = data.data_as_bytes()?;
                let mut values = bytes.split(|b| *b==0).map(|b| {
                    Ok(from_utf8(b).map_err(|_| ())?.to_string())
                }).collect::<Result<Vec<_>,_>>()?;
                values.pop();
                Ok(Arc::new(values))
            },
            StringAlgorithm::Classify(index,values) => {
                let values = values.make()?;
                let out = index.make()?.iter().map(|p| {
                    let p = *p as usize;
                    if p < values.len() { Ok(values[p].clone()) } else { Err(()) }
                }).collect::<Result<Vec<_>,_>>()?;
                Ok(Arc::new(out))
            }
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum BooleanAlgorithm {
    Array(ReceivedData),
    Bytes(ReceivedData)
}

impl BooleanAlgorithm {
    fn new<'a>(code: &mut Chars<'a>, data: &mut Vec<ReceivedData>) -> Result<BooleanAlgorithm,()> {
        Ok(match code.next() {
            Some('A') => BooleanAlgorithm::Array(pop(data)?),
            Some('B') => BooleanAlgorithm::Bytes(pop(data)?),
            _ => { return Err(()); }
        })
    }

    fn specify<'a>(code: &mut Chars<'a>, spec: &mut Vec<ReceivedDataType>) -> Result<(),()> {
        match code.next() {
            Some('A') => { spec.push(ReceivedDataType::Booleans); },
            Some('B') => { spec.push(ReceivedDataType::Bytes); },
            _ => { return Err(()); }
        }
        Ok(())
    }

    fn make<'a> (&self) -> Result<Arc<Vec<bool>>,()> {
        match self {
            BooleanAlgorithm::Array(data) => {
                Ok(data.data_as_booleans()?.clone())
            },
            BooleanAlgorithm::Bytes(data) => {
                Ok(Arc::new(data.data_as_bytes()?.iter().map(|x| *x!=0).collect()))
            }
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum DataAlgorithm {
    Numbers(NumberAlgorithm),
    Strings(StringAlgorithm),
    Booleans(BooleanAlgorithm),
    Empty
}

impl DataAlgorithm {
    fn new(code: &str, data: &mut Vec<ReceivedData>) -> Result<DataAlgorithm,()> {
        let mut code = code.chars();
        data.reverse();
        Ok(match code.next() {
            Some('N') => DataAlgorithm::Numbers(NumberAlgorithm::new(&mut code,data)?),
            Some('S') => DataAlgorithm::Strings(StringAlgorithm::new(&mut code,data)?),
            Some('B') => DataAlgorithm::Booleans(BooleanAlgorithm::new(&mut code,data)?),
            Some('E') => DataAlgorithm::Empty,
            _ => { return Err(()); }
        })
    }

    fn specify(code: &str) -> Result<Vec<ReceivedDataType>,()> {
        let mut code = code.chars();
        let mut spec = vec![];
        match code.next() {
            Some('N') => { NumberAlgorithm::specify(&mut code,&mut spec)?; },
            Some('S') => { StringAlgorithm::specify(&mut code,&mut spec)?; },
            Some('B') => { BooleanAlgorithm::specify(&mut code,&mut spec)?; },
            Some('E') => {},
            _ => { return Err(()); }
        }
        Ok(spec)
    }

    pub fn to_received_data(&self) -> Result<ReceivedData,()> {
        match self {
            DataAlgorithm::Numbers(n) => { 
                Ok(ReceivedData::new_arc_numbers(&n.make()?))
            },
            DataAlgorithm::Strings(s) => {
                Ok(ReceivedData::new_arc_strings(&s.make()?))
            },
            DataAlgorithm::Booleans(b) => {
                Ok(ReceivedData::new_arc_booleans(&b.make()?))
            },
            DataAlgorithm::Empty => {
                Ok(ReceivedData::new_empty())
            }
        }
    }
}

struct DataAlgorithmVisitor;

impl<'de> Visitor<'de> for DataAlgorithmVisitor {
    type Value = DataAlgorithm;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a DataAlgorithm")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let code : String = st_field("group",seq.next_element()?)?;
        let mut data = vec![];
        for variety in DataAlgorithm::specify(&code).map_err(|_| de::Error::custom("bad data format"))? {
            match variety {
                ReceivedDataType::Bytes => {
                    let value = st_field("expected bytes",seq.next_element::<ByteData>().ok().flatten())?.data;
                    data.push(ReceivedData::new_bytes(value));
                },
                ReceivedDataType::Booleans => {
                    let value = st_field("expected bools",seq.next_element::<Vec<bool>>().ok().flatten())?;
                    data.push(ReceivedData::new_booleans(value));
                },
                ReceivedDataType::Numbers => {
                    let value = st_field("expected numbers",seq.next_element::<Vec<f64>>().ok().flatten())?;
                    data.push(ReceivedData::new_numbers(value));
                },
                ReceivedDataType::Strings => {
                    let value = st_field("expected strings",seq.next_element::<Vec<String>>().ok().flatten())?;
                    data.push(ReceivedData::new_strings(value));
                },
                ReceivedDataType::Empty => {
                    let _ : IgnoredAny = st_field("expected empty",seq.next_element::<IgnoredAny>().ok().flatten())?;
                    data.push(ReceivedData::new_empty());
                }
            }
        }
        Ok(DataAlgorithm::new(&code,&mut data).map_err(|_| de::Error::custom("bad data format"))?)
    }
}

impl<'de> Deserialize<'de> for DataAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<DataAlgorithm, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(DataAlgorithmVisitor)
    }
}
