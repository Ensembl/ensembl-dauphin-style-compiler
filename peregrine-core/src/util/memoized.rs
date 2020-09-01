use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{ Arc, Mutex };
use crate::lock;
use varea::Cache;
use commander::PromiseFuture;
use super::fuse::FusePromise;

enum MemoizedStore<K,V> {
    Complete(HashMap<K,Arc<V>>),
    LruCache(Cache<K,Arc<V>>)
}

impl<K,V> MemoizedStore<K,V> where K: Clone+Eq+Hash {
    fn insert(&mut self, k: K, v: Arc<V>) {
        match self {
            MemoizedStore::Complete(hm) => { hm.insert(k,v); },
            MemoizedStore::LruCache(c) => { c.put(&k,v); }
        }
    }

    fn get(&mut self, k: &K) -> Option<&Arc<V>> {
        match self {
            MemoizedStore::Complete(hm) => { hm.get(k) },
            MemoizedStore::LruCache(c) => { c.get(k) }
        }
    }

    fn guaranteed(&self, k: &K) -> bool {
        match self {
            MemoizedStore::Complete(hm) => { hm.contains_key(k) },
            MemoizedStore::LruCache(_) => { false }
        }
    }
}

pub struct MemoizedData<K,V> {
    known: MemoizedStore<K,V>,
    pending: HashMap<K,FusePromise<Arc<V>>>
}

pub struct MemoizedDataResult<K,V> {
    memoized: Memoized<K,V>,
    key: K
}

impl<K,V> MemoizedDataResult<K,V> where K: Clone+Eq+Hash {
    pub fn resolve(self, value: V) {
        self.memoized.add(self.key,value)
    }
}

pub struct Memoized<K,V> {
    data: Arc<Mutex<MemoizedData<K,V>>>,
    resolver: Arc<Box<dyn Fn(&K,MemoizedDataResult<K,V>)>>
}

// Rust bug means can't derive Clone on polymorphic types
impl<K,V> Clone for Memoized<K,V> {
    fn clone(&self) -> Self {
        Memoized {
            data: self.data.clone(),
            resolver: self.resolver.clone()
        }
    }
}

impl<K,V> MemoizedData<K,V> where K: Clone+Eq+Hash {
    pub fn new() -> MemoizedData<K,V> {
        MemoizedData {
            known: MemoizedStore::Complete(HashMap::new()),
            pending: HashMap::new(),
        }
    }

    pub fn new_cache(size: usize) -> MemoizedData<K,V> {
        MemoizedData {
            known: MemoizedStore::LruCache(Cache::new(size)),
            pending: HashMap::new(),
        }
    }

    fn add(&mut self, key: K, value: V) {
        if self.known.guaranteed(&key) { return; }
        let v = Arc::new(value);
        self.known.insert(key.clone(),v.clone());
        if let Some(mut fuse) = self.pending.remove(&key) {
            fuse.fuse(v);
        }
    }

    fn promise(&mut self, key: &K) -> (PromiseFuture<Arc<V>>,bool) {
        let p = PromiseFuture::new();
        let request = if let Some(value) = self.known.get(key) {
            p.satisfy(value.clone());
            false
        } else if let Some(fuse) = self.pending.get_mut(key) {
            fuse.add(p.clone());
            false
        } else {
            let mut fuse = FusePromise::new();
            fuse.add(p.clone());
            self.pending.insert(key.clone(),fuse);
            true
        };
        (p,request)
    }
}

impl<K,V> Memoized<K,V> where K: Clone+Eq+Hash {
    pub fn new<F>(f: F) -> Memoized<K,V> where F: Fn(&K,MemoizedDataResult<K,V>) + 'static {
        Memoized {
            data: Arc::new(Mutex::new(MemoizedData::new())),
            resolver: Arc::new(Box::new(f))
        }
    }

    pub fn new_cache<F>(size: usize, f: F) -> Memoized<K,V> where F: Fn(&K,MemoizedDataResult<K,V>) + 'static {
        Memoized {
            data: Arc::new(Mutex::new(MemoizedData::new_cache(size))),
            resolver: Arc::new(Box::new(f))
        }
    }

    pub fn add(&self, key: K, value: V) {
        lock!(self.data).add(key,value);
    }

    pub fn get_no_wait(&self, key: &K) -> anyhow::Result<()> {
        let mut data = lock!(self.data);
        let (_,request) = data.promise(key);
        drop(data);
        if request {
            (self.resolver)(key,MemoizedDataResult {
                memoized: self.clone(),
                key: key.clone()
            });
        }
        Ok(())
    }

    pub async fn get(&self, key: &K) -> anyhow::Result<Arc<V>> {
        let mut data = lock!(self.data);
        let (promise,request) = data.promise(key);
        drop(data);
        if request {
            (self.resolver)(key,MemoizedDataResult {
                memoized: self.clone(),
                key: key.clone()
            });
        }
        Ok(promise.await)
    }
}
