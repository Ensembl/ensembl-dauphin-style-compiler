use std::{sync::Arc, hash::{Hash, Hasher, BuildHasher}, collections::{hash_map::{DefaultHasher}, HashMap}, fmt};

const TOKEN_ALL : &str = "**";

#[derive(Clone)]
pub struct AllotmentName {
    hash: Arc<u64>,
    name: Arc<Vec<String>>,
    container: bool
}

impl Hash for AllotmentName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentName {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentName {}

impl PartialOrd for AllotmentName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for AllotmentName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl AllotmentName {
    pub(crate) fn do_new(name: Vec<String>, container: bool) -> AllotmentName {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        container.hash(&mut hasher);
        AllotmentName {
            hash: Arc::new(hasher.finish()),
            name: Arc::new(name),
            container
        }
    }

    pub(crate) fn new(spec: &str) -> AllotmentName {
        let mut name = spec.split("/").map(|x| x.to_string()).collect::<Vec<_>>();
        let mut container = false;
        if let Some("") = name.last().map(|x| x.as_str()) {
            name.pop();
            container = true;
        }
        Self::do_new(name,container)
    }

    pub(crate) fn from_part(part: &AllotmentNamePart) -> AllotmentName {
        Self::do_new(part.sequence().to_vec(),part.name.container)
    }

    pub fn name(&self) -> &[String] { &self.name }

    pub fn hash_value(&self) -> u64 { *self.hash }
    pub fn sequence(&self) -> &[String] { &self.name }
    pub(crate) fn is_container(&self) -> bool { self.container }
    pub fn is_dustbin(&self) -> bool { self.name.len() == 0 }
}

#[cfg(debug_assertions)]
impl fmt::Debug for AllotmentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}{}",self.sequence().join("/"),if self.container { "/" } else { "" })
    }
}

pub struct PassThroughHasher(u64);

impl Hasher for PassThroughHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0<<8) | (byte as u64);
        }
    }
    
    fn finish(&self) -> u64 { self.0 }
}

#[derive(Clone)]
pub struct BuildPassThroughHasher;

impl BuildHasher for BuildPassThroughHasher {
    type Hasher = PassThroughHasher;
    fn build_hasher(&self) -> PassThroughHasher {
        PassThroughHasher(0)
    }
}

pub type AllotmentNameHashMap<T> = HashMap<AllotmentName,T,BuildPassThroughHasher>;

pub fn allotmentname_hashmap<T>() -> AllotmentNameHashMap<T> {
    HashMap::<_,_,BuildPassThroughHasher>::with_hasher(BuildPassThroughHasher)
}

pub struct AllotmentNamePrefixes<'a> {
    name: &'a AllotmentNamePart,
    end: usize
}

impl<'a> Iterator for AllotmentNamePrefixes<'a> {
    type Item=AllotmentNamePart;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end < self.name.end {
            let mut out = self.name.clone();
            out.end = self.end;
            self.end += 1;
            Some(out)
        } else {
            None
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
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

    pub(crate) fn iter_prefixes(&self) -> impl Iterator<Item=AllotmentNamePart> + '_ {
        AllotmentNamePrefixes {
            name: self,
            end: 0
        }
    }

    pub(crate) fn full(&self) -> &AllotmentName { &self.name }
    pub(crate) fn empty(&self) -> bool { self.end == self.start }
    pub(crate) fn sequence(&self) -> &[String] { &self.name.name[self.start..self.end] }
    pub(crate) fn removed_head(&self) -> AllotmentNamePart {
        let mut out = self.clone();
        out.start = 0;
        out.end = self.start;
        out
    }

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
