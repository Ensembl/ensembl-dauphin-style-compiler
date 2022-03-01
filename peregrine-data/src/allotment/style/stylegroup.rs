use std::{sync::{Mutex, Arc}, collections::HashMap};

use peregrine_toolkit::lock;

use super::{style::{ContainerAllotmentStyle, LeafAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}};

// XXX excessive cloning: does this really need to be mutex?
pub struct AllotmentStyleGroup {
    container_style: Arc<Mutex<HashMap<Vec<String>,Arc<ContainerAllotmentStyle>>>>,
    container_empty: Arc<ContainerAllotmentStyle>,
    leaf_style: Arc<Mutex<HashMap<Vec<String>,Arc<LeafAllotmentStyle>>>>,
    leaf_empty: Arc<LeafAllotmentStyle>
}

impl AllotmentStyleGroup {
    pub fn empty() -> AllotmentStyleGroup {
        AllotmentStyleGroup {
            container_style: Arc::new(Mutex::new(HashMap::new())),
            container_empty: Arc::new(ContainerAllotmentStyle::empty()),
            leaf_style: Arc::new(Mutex::new(HashMap::new())),
            leaf_empty: Arc::new(LeafAllotmentStyle::empty())
        }
    }

    pub fn add_container(&mut self, name: &AllotmentName, style: ContainerAllotmentStyle) {
        lock!(self.container_style).insert(name.sequence().to_vec(),Arc::new(style));
    }

    pub fn add_leaf(&mut self, name: &AllotmentName, style: LeafAllotmentStyle) {
        lock!(self.leaf_style).insert(name.sequence().to_vec(),Arc::new(style));
    }

    pub fn add(&mut self, spec: &str, values: HashMap<String,String>) {
        let name = AllotmentName::new(spec);
        if name.is_container() {
            let style = ContainerAllotmentStyle::build(&values);
            self.add_container(&name,style);
        } else {
            let style = LeafAllotmentStyle::build(&values);
            self.add_leaf(&name,style);
        }
    }

    pub(super) fn get_container(&self, name: &AllotmentNamePart) -> Arc<ContainerAllotmentStyle> {
        lock!(self.container_style).get(&name.sequence().to_vec()).cloned().unwrap_or(self.container_empty.clone())
    }

    pub(super) fn get_leaf(&self, name: &AllotmentNamePart) -> Arc<LeafAllotmentStyle> {
        lock!(self.leaf_style).get(&name.sequence().to_vec()).cloned().unwrap_or(self.leaf_empty.clone())
    }
}
