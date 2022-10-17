use std::sync::Arc;

use crate::{BackendNamespace, switch::expansion::Expansion};

#[derive(Debug)]
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

#[derive(Debug)]
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
