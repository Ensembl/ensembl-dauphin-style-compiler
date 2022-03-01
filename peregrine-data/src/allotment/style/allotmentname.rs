use std::sync::Arc;

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct AllotmentName {
    name: Arc<Vec<String>>,
    container: bool
}

impl AllotmentName {
    pub(super) fn new(spec: &str) -> AllotmentName {
        let mut name = spec.split("/").map(|x| x.to_string()).collect::<Vec<_>>();
        let mut container = false;
        if let Some("") = name.last().map(|x| x.as_str()) {
            name.pop();
            container = true;
        }
        AllotmentName {
            name: Arc::new(name),
            container
        }
    }

    pub(super) fn sequence(&self) -> &[String] { &self.name }
    pub(super) fn is_container(&self) -> bool { self.container }
}

#[derive(Clone)]
pub struct AllotmentNamePart {
    name: AllotmentName,
    end: usize
}

impl AllotmentNamePart {
    pub(super) fn new(name: AllotmentName) -> AllotmentNamePart {
        AllotmentNamePart {
            end: name.name.len(), name
        }
    }

    pub(super) fn sequence(&self) -> &[String] { &self.name.name[0..self.end] }

    pub(super) fn pop(&self) -> Option<AllotmentNamePart> {
        if self.end > 0 {
            let mut part = self.clone();
            part.end -= 1;
            Some(part)
        } else {
            None
        }
    }
}
