use std::{collections::HashMap, sync::Arc};

fn parse_report_value(input: &str) -> (Arc<HashMap<String,String>>,Vec<String>) {
    let parts = input.split(";").collect::<Vec<_>>();
    let mut values = HashMap::new();
    let mut reports = vec![];
    for item in parts {
        if let Some(eq_at) = item.find("=") {
            let (k,v) = item.split_at(eq_at);
            values.insert(k.to_string(),v[1..].to_string());
        } else if item.starts_with("!") {
            reports.push(item[1..].to_string());
        } else {
            values.insert("type".to_string(),item.to_string());
        }
    }
    (Arc::new(values),reports)
}


#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub(crate) struct MetadataStyle {
    values: Arc<HashMap<String,String>>,
    report: Vec<String>
}

impl MetadataStyle {
    pub(crate) fn new(spec: &str) -> MetadataStyle {
        let (values,report) = parse_report_value(spec);
        MetadataStyle { values,report }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item=(&String,&String)> {
        self.values.iter()
    }

    pub(crate) fn reporting(&self) -> &[String] { &self.report }
}
