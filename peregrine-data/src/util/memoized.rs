use std::collections::HashMap;
use std::future::Future;
use std::sync::{ Arc, Mutex };
use std::hash::Hash;
use std::pin::Pin;
use commander::{ FusePromise, PromiseFuture };
use varea::Cache;
use crate::lock;

pub enum MemoizedType {
    Store,
    Cache(usize),
    None
}

enum MemoizedStore<K,V> {
    Complete(HashMap<K,Arc<V>>),
    LruCache(Cache<K,Arc<V>>),
    None
}

impl<K,V> MemoizedStore<K,V> where K: Clone+Eq+Hash {
    fn new(kind: MemoizedType) -> MemoizedStore<K,V> {
        match kind {
            MemoizedType::Store => MemoizedStore::Complete(HashMap::new()),
            MemoizedType::Cache(size) => MemoizedStore::LruCache(Cache::new(size)),
            MemoizedType::None => MemoizedStore::None
        }
    }

    fn insert(&mut self, k: K, v: Arc<V>) {
        match self {
            MemoizedStore::Complete(hm) => { hm.insert(k,v); },
            MemoizedStore::LruCache(c) => { c.put(&k,v); },
            MemoizedStore::None => {},
        }
    }

    fn get(&mut self, k: &K) -> Option<&Arc<V>> {
        match self {
            MemoizedStore::Complete(hm) => { hm.get(k) },
            MemoizedStore::LruCache(c) => { c.get(k) },
            MemoizedStore::None => { None }
        }
    }
}

struct MemoizedState<K: Clone+Eq+Hash,V> {
    known: MemoizedStore<K,V>,
    pending: HashMap<K,FusePromise<Arc<V>>>
}

impl<K: Clone+Eq+Hash,V> MemoizedState<K,V> {
    fn new(kind: MemoizedType) -> MemoizedState<K,V> {
        MemoizedState {
            known: MemoizedStore::new(kind),
            pending: HashMap::new()
        }
    }

    pub fn insert(&mut self, key: &K, value: Arc<V>) {
        self.known.insert(key.clone(),value)
    }

    fn get_promise(&mut self, key: &K) -> (PromiseFuture<Arc<V>>,Option<FusePromise<Arc<V>>>) {
        let p = PromiseFuture::new();
        let fuse = if let Some(value) = self.known.get(key) {
            /* already known: satisfy immediately; don't run future */
            p.satisfy(value.clone());
            None
        } else if let Some(fuse) = self.pending.get_mut(key) {
            /* already pending: add to list; don't run future */
            fuse.add(p.clone());
            None
        } else {
            /* not known: create a future and tell caller to run */
            let fuse = FusePromise::new();
            fuse.add(p.clone());
            self.pending.insert(key.clone(),fuse.clone());
            Some(fuse)
        };
        (p,fuse)
    }

}

#[derive(Clone)]
pub struct Memoized<K: Clone+Hash+Eq,V> {
    resolver: Arc<Box<dyn (Fn(&Memoized<K,V>,&K) -> Pin<Box<dyn Future<Output=V>>>) + 'static>>,
    state: Arc<Mutex<MemoizedState<K,V>>>
}

impl<K: Clone+Hash+Eq,V> Memoized<K,V> {
    pub fn new<F>(kind: MemoizedType, cb: F) -> Memoized<K,V> where F: Fn(&Memoized<K,V>,&K) -> Pin<Box<dyn Future<Output=V>>> + 'static {
        Memoized {
            state: Arc::new(Mutex::new(MemoizedState::new(kind))),
            resolver: Arc::new(Box::new(cb))
        }
    }

    pub fn warm(&self, key: &K, value: V) {
        lock!(self.state).insert(key,Arc::new(value));
    }

    pub async fn get(&self, key: &K) -> Arc<V> {
        let mut state = lock!(self.state);
        let (promise,fuse) = state.get_promise(key);
        drop(state);
        if let Some(fuse) = fuse {
            let twin = self.clone();
            let value = Arc::new((self.resolver)(twin,key).await);
            lock!(self.state).insert(key,value.clone());
            fuse.fuse(value);
        }
        promise.await
    }
}
