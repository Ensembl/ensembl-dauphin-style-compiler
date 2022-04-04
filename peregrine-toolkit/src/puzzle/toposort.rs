use std::{hash::Hash, collections::HashMap};

use crate::log;

pub struct TopoSort<T: Hash+Eq+Clone> {
    clearable: Vec<T>,
    depends_on: HashMap<T,Vec<(T,Vec<T>)>>
}

impl<T: Hash+Eq+Clone + std::fmt::Debug> TopoSort<T> {
    pub fn new() -> TopoSort<T> {
        TopoSort {
            clearable: vec![],
            depends_on: HashMap::new()
        }
    }

    pub fn add(&mut self, value: T, mut dependencies: Vec<T>) {
        for dependency in &dependencies {
            if !self.depends_on.contains_key(&dependency) {
                self.depends_on.insert(dependency.clone(),vec![]);
            }
        }
        if let Some(dep) = dependencies.pop() {
            self.depends_on.get_mut(&dep).unwrap().push((value,dependencies));
        } else {
            self.clearable.push(value);
        }
    }

    pub fn sort(&mut self) -> Vec<T> {
        let mut out = vec![];
        while let Some(ready) = self.clearable.pop() {
            if let Some(mut refiles) = self.depends_on.remove(&ready) {
                for (refile_name,mut refile_deps) in refiles.drain(..) {
                    let mut refile_here = None;
                    while let Some(refile_dep) = refile_deps.pop() {
                        if self.depends_on.contains_key(&refile_dep) {
                            refile_here = Some(refile_dep);
                            break;
                        }
                    }
                    if let Some(refile_dep) = refile_here {
                        self.depends_on.entry(refile_dep).or_insert_with(|| vec![]).push((refile_name,refile_deps));
                    } else {
                        self.clearable.push(refile_name);
                    }  
                }
            }
            out.push(ready);
        }
        out
    }
}
