use std::{sync::Arc };
use serde::{Deserialize, Deserializer };
use crate::{BackendNamespace, switch::expansion::Expansion};

#[derive(Debug,serde_derive::Deserialize)]
pub struct ExpansionModelBuilder {
    name: String,
    backend_namespace: BackendNamespace,
    triggers: Vec<Vec<String>>,

}

impl ExpansionModelBuilder {
    pub fn new(backend_namespace: &BackendNamespace, name: &str) -> ExpansionModelBuilder {
        ExpansionModelBuilder {
            name: name.to_string(),
            backend_namespace: backend_namespace.clone(),
            triggers: vec![]
        }
    }

    pub fn add_trigger(&mut self, trigger: &[String]) {
        self.triggers.push(trigger.to_vec());
    }
}

#[derive(Debug,Clone)]
pub struct ExpansionModel(Arc<ExpansionModelBuilder>);

impl ExpansionModel {
    pub fn new(builder: ExpansionModelBuilder) -> ExpansionModel {
        ExpansionModel(Arc::new(builder))
    }

    pub(crate) fn to_expansion(&self) -> Expansion {
        Expansion::new(&self.0.name,&self.0.backend_namespace)
    }

    pub(crate) fn triggers(&self) -> &[Vec<String>] {
        &self.0.triggers
    }
}

impl<'de> Deserialize<'de> for ExpansionModel {
    fn deserialize<D>(deserializer: D) -> Result<ExpansionModel, D::Error>
            where D: Deserializer<'de> {
        let builder = ExpansionModelBuilder::deserialize(deserializer)?;
        Ok(ExpansionModel(Arc::new(builder)))
    }
}
