use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use crate::lock;
use commander::PromiseFuture;
use super::fuse::FusePromise;

pub struct MemoizedData<K,V> {
    known: HashMap<K,Arc<V>>,
    pending: HashMap<K,FusePromise>
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
            known: HashMap::new(),
            pending: HashMap::new(),
        }
    }

    fn add(&mut self, key: K, value: V) {
        if self.known.contains_key(&key) { return; }
        self.known.insert(key.clone(),Arc::new(value));
        if let Some(mut fuse) = self.pending.remove(&key) {
            fuse.fuse();
        }
    }

    fn promise(&mut self, key: &K) -> (PromiseFuture<()>,bool) {
        let p = PromiseFuture::new();
        let request = if self.known.contains_key(key) {
            p.satisfy(());
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

    fn get(&self, key: &K) -> Option<Arc<V>> { self.known.get(key).cloned() }
}

impl<K,V> Memoized<K,V> where K: Clone+Eq+Hash {
    pub fn new<F>(f: F) -> Memoized<K,V> where F: Fn(&K,MemoizedDataResult<K,V>) + 'static {
        Memoized {
            data: Arc::new(Mutex::new(MemoizedData::new())),
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
        loop {
            let mut data = lock!(self.data);
            let (promise,request) = data.promise(key);
            drop(data);
            if request {
                (self.resolver)(key,MemoizedDataResult {
                    memoized: self.clone(),
                    key: key.clone()
                });
            }
            promise.await;
            let data = lock!(self.data);
            if let Some(value) = data.get(key) {
                return Ok(value);
            }
            drop(data);
        }
    }
}
