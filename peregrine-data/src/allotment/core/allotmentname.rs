use std::{sync::Arc, hash::{Hash, Hasher, BuildHasher}, collections::{hash_map::{DefaultHasher}, HashMap}, fmt};

#[derive(Clone)]
pub struct AllotmentName {
    hash: Arc<u64>,
    name: Arc<Vec<String>>
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
        Some(self.cmp(other))
    }
}

impl Ord for AllotmentName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.name.len().cmp(&other.name.len()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => self.hash.cmp(&other.hash),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater
        }
    }
}

impl AllotmentName {
    pub(crate) fn do_new(name: Vec<String>) -> AllotmentName {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        AllotmentName {
            hash: Arc::new(hasher.finish()),
            name: Arc::new(name),
        }
    }

    pub(crate) fn new(spec: &str) -> AllotmentName {
        let mut name = spec.split("/").map(|x| x.to_string()).collect::<Vec<_>>();
        if let Some("") = name.last().map(|x| x.as_str()) {
            name.pop();
        }
        Self::do_new(name)
    }

    pub fn name(&self) -> &[String] { &self.name }
    pub fn hash_value(&self) -> u64 { *self.hash }
    pub fn sequence(&self) -> &[String] { &self.name }
    pub fn is_dustbin(&self) -> bool { self.name.len() == 0 }
}

#[cfg(debug_assertions)]
impl fmt::Debug for AllotmentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.sequence().join("/"))
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
