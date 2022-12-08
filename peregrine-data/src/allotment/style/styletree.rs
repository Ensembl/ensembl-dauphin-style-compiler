use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::{lock};
use crate::{allotment::{style::{containerstyle::ContainerStyle, leafstyle::{InheritableLeafStyle, UninheritableLeafStyle, LeafStyle}}, core::allotmentname::{AllotmentName, AllotmentNameHashMap, allotmentname_hashmap}}};
use super::{pathtree::{PathTree, PathKey}};

struct StyleTreeInternal {
    container: PathTree<Vec<(String,String)>,String,HashMap<String,String>>,    
    leaf: PathTree<Vec<(String,String)>,String,HashMap<String,String>>
}

impl StyleTreeInternal {
    fn new()-> StyleTreeInternal {
        let merge_add = |map: &mut Option<HashMap<String,String>>, mut values: Vec<_>| {
            map.get_or_insert_with(|| HashMap::new()).extend(&mut values.drain(..));
        };
        let merge_lookup = |map: &mut Option<HashMap<String,String>>, values: &HashMap<String,String>| {
            let map = map.get_or_insert_with(|| HashMap::new());
            for (k,v) in values {
                if !map.contains_key(k) {
                    map.insert(k.to_string(),v.to_string());
                }
            }
        };
        StyleTreeInternal {
            container: PathTree::new(merge_add,merge_lookup),
            leaf: PathTree::new(merge_add,merge_lookup)
        }
    }

    fn add(&mut self, spec: &str, values: Vec<(String,String)>) {
        let mut parts = spec.split("/").collect::<Vec<_>>();
        let container = if let Some(last) = parts.last() { *last == "" } else { false };
        if container { parts.pop(); }
        let tree = if container { &mut self.container } else { &mut self.leaf };
        let mut path = parts.drain(..).map(|p| 
            match p {
                "*" => PathKey::AnyOne,
                "**" => PathKey::AnyMany,
                x => PathKey::Fixed(x.to_string())
             }
        ).collect::<Vec<_>>();
        path.reverse();
        tree.add(&path,values);
    }

    fn lookup(&self, path: &[String], container: bool) -> HashMap<String,String> {
        let mut path = path.to_vec();
        path.reverse();
        let tree = if container { &self.container } else { &self.leaf };
        tree.lookup(&path).unwrap_or_else(|| HashMap::new())
    }

    fn lookup_container(&self, path: &[String]) -> HashMap<String,String> {
        self.lookup(path,true)
    }

    fn lookup_leaf(&self, path: &[String]) -> HashMap<String,String> {
        self.lookup(path,false)
    }
}

struct StyleTreeState {
    internal: StyleTreeInternal,
    container_cache: HashMap<Vec<String>,ContainerStyle>,
    leaf_cache: AllotmentNameHashMap<LeafStyle>
}

#[derive(Clone)]
pub(crate) struct StyleTree(Arc<Mutex<StyleTreeState>>);

impl StyleTree {
    pub(crate) fn new() -> StyleTree {
        StyleTree(Arc::new(Mutex::new(
            StyleTreeState {
                internal: StyleTreeInternal::new(),
                container_cache: HashMap::new(),
                leaf_cache: allotmentname_hashmap()
            }
        )))
    }

    pub(crate) fn add(&mut self, spec: &str, values: Vec<(String,String)>) {
        lock!(self.0).internal.add(spec,values);
    }

    pub(crate) fn lookup_container(&self, allotment: &AllotmentName) -> ContainerStyle {
        let mut state = lock!(self.0);
        if !state.container_cache.contains_key(allotment.name()) {
            let container = state.internal.lookup_container(allotment.name());
            state.container_cache.insert(allotment.name().to_vec(),ContainerStyle::build(&container));
        }
        state.container_cache.get(allotment.name()).unwrap().clone()
    }

    pub(crate) fn lookup_leaf(&self, allotment: &AllotmentName) -> LeafStyle {
        let mut state = lock!(self.0);
        if !state.leaf_cache.contains_key(allotment) {
            let mut inherit = InheritableLeafStyle::empty();
            let name = allotment.name();
            for index in 0..name.len() {
                let prefix = &name[0..index];
                if !state.container_cache.contains_key(prefix) {
                    let container = state.internal.lookup_container(prefix);
                    state.container_cache.insert(prefix.to_vec(),ContainerStyle::build(&container));
                }
                let style = state.container_cache.get(prefix).unwrap();
                inherit.override_style(&style.leaf);
            }
            let leaf = state.internal.lookup_leaf(allotment.name());
            let specified = UninheritableLeafStyle::build(&leaf);
            inherit.override_style(&InheritableLeafStyle::new(&leaf));
            let style = inherit.make(&specified);
            state.leaf_cache.insert(allotment.clone(),style);
        }
        state.leaf_cache.get(allotment).unwrap().clone()
    }
}
