use std::collections::HashMap;

use serde::{Serializer, ser::SerializeSeq};

use super::metricutil::FactoredValueBuilder;

#[cfg_attr(debug_assertions,derive(Debug))]
struct GeneralDatapoint {
    tags: Vec<(usize,usize)>,
    values: Vec<(usize,f64)>
}

impl serde::Serialize for GeneralDatapoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut tag_keys = vec![];
        let mut tag_values = vec![];
        let mut value_keys = vec![];
        let mut value_values = vec![];
        for (key,value) in &self.tags {
            tag_keys.push(*key);
            tag_values.push(*value);
        }
        for (key,value) in &self.values {
            value_keys.push(*key);
            value_values.push(*value);
        }
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&tag_keys)?;
        seq.serialize_element(&tag_values)?;
        seq.serialize_element(&value_keys)?;
        seq.serialize_element(&value_values)?;
        seq.end()
    }
}

struct GeneralMetricSeriesBuilder {
    tag_keys: FactoredValueBuilder<String>,
    tag_values: FactoredValueBuilder<String>,
    value_keys: FactoredValueBuilder<String>,
    datapoints: Vec<GeneralDatapoint>
}

impl GeneralMetricSeriesBuilder {
    pub fn new() -> GeneralMetricSeriesBuilder {
        GeneralMetricSeriesBuilder {
            tag_keys: FactoredValueBuilder::new(),
            tag_values: FactoredValueBuilder::new(),
            value_keys: FactoredValueBuilder::new(),
            datapoints: vec![]
        }
    }

    pub fn add(&mut self, tags: &[(String,String)], values: &[(String,f64)]) {
        let tags = tags.iter().map(|(k,v)| (self.tag_keys.lookup(k), self.tag_values.lookup(v))).collect::<Vec<_>>();
        let values = values.iter().map(|(k,v)| (self.value_keys.lookup(k), *v)).collect::<Vec<_>>();
        self.datapoints.push(GeneralDatapoint { tags, values })
    }
}

pub struct GeneralMetricBuilder {
    series: HashMap<String,GeneralMetricSeriesBuilder>
}

impl GeneralMetricBuilder {
    pub fn new() -> GeneralMetricBuilder {
        GeneralMetricBuilder {
            series: HashMap::new()
        }
    }

    pub fn add(&mut self, name: &str, tags: &[(String,String)], values: &[(String,f64)]) {
        if !self.series.contains_key(name) {
            self.series.insert(name.to_string(),GeneralMetricSeriesBuilder::new());
        }
        self.series.get_mut(name).unwrap().add(tags,values);
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct GeneralMetricSeries {
    name: String,
    tag_keys: Vec<String>,
    tag_values: Vec<String>,
    value_keys: Vec<String>,
    datapoints: Vec<GeneralDatapoint>
}

impl GeneralMetricSeries {
    fn new(name: &str, mut builder: GeneralMetricSeriesBuilder) -> GeneralMetricSeries {
        GeneralMetricSeries {
            name: name.to_string(),
            tag_keys: builder.tag_keys.build(),
            tag_values: builder.tag_values.build(),
            value_keys: builder.value_keys.build(),
            datapoints: builder.datapoints
        }
    }
}

impl serde::Serialize for GeneralMetricSeries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(5))?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.tag_keys)?;
        seq.serialize_element(&self.tag_values)?;
        seq.serialize_element(&self.value_keys)?;
        seq.serialize_element(&self.datapoints)?;
        seq.end()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct GeneralMetricData(Vec<GeneralMetricSeries>);

impl GeneralMetricData {
    pub fn new(builder: &mut GeneralMetricBuilder) -> GeneralMetricData {
        GeneralMetricData(builder.series.drain().map(|(k,v)| {
            GeneralMetricSeries::new(&k,v)
        }).collect())
    }
}

impl serde::Serialize for GeneralMetricData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for series in &self.0 {
            seq.serialize_element(series)?;
        }
        seq.end()
    }
}
