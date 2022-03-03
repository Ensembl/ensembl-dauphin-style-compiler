use std::sync::Arc;

const TOKEN_ALL : &str = "**";


#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct AllotmentName {
    name: Arc<Vec<String>>,
    container: bool
}

impl AllotmentName {
    pub(crate) fn new(spec: &str) -> AllotmentName {
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

    pub(crate) fn sequence(&self) -> &[String] { &self.name }
    pub(crate) fn is_container(&self) -> bool { self.container }
}

#[derive(Clone,Debug)]
pub struct AllotmentNamePart {
    name: AllotmentName,
    start: usize,
    end: usize
}

impl AllotmentNamePart {
    pub(crate) fn new(name: AllotmentName) -> AllotmentNamePart {
        AllotmentNamePart {
            start: 0,
            end: name.name.len(),
            name
        }
    }

    pub(crate) fn full(&self) -> &AllotmentName { &self.name }
    pub(crate) fn empty(&self) -> bool { self.end == self.start }
    pub(crate) fn sequence(&self) -> &[String] { &self.name.name[self.start..self.end] }

    pub(crate) fn shift(&self) -> Option<(String,AllotmentNamePart)> {
        if !self.empty() {
            let mut part = self.clone();
            part.start += 1;
            Some((part.name.name[part.start-1].to_string(),part))
        } else {
            None
        }
    }

    pub(crate) fn pop(&self) -> Option<(String,AllotmentNamePart)> {
        if !self.empty() {
            let mut part = self.clone();
            part.end -= 1;
            Some((part.name.name[part.end].to_string(),part))
        } else {
            None
        }
    }

    pub(crate) fn remove_all(&mut self) -> bool {
        if !self.empty() {
            if self.name.name[self.start] == TOKEN_ALL {
                self.start += 1;
                return true;
            }
        }
        false
    }
}
