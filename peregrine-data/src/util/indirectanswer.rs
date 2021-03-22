use commander::PromiseFuture;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::{ Arc, Mutex };
use super::fuse::FusePromise;

#[derive(Clone)]
pub struct IndirectData<K,V>(Arc<Mutex<HashMap<K,FusePromise<V>>>>,bool);

impl<K,V> IndirectData<K,V> where K: Eq+Hash+Clone, V: Clone {
    pub(crate) fn new_keep() -> IndirectData<K,V> {
        IndirectData(Arc::new(Mutex::new(HashMap::new())),false)
    }

    pub(crate) fn new_pending() -> IndirectData<K,V> {
        IndirectData(Arc::new(Mutex::new(HashMap::new())),true)
    }

    pub(crate) fn get_future(&self, k: &K) -> impl Future<Output=V> {
        let mut obj = self.0.lock().unwrap();
        if !obj.contains_key(k) {
            obj.insert(k.clone(),FusePromise::new());
        }
        let p = PromiseFuture::new();
        obj.get(k).unwrap().add(p.clone());
        p
    }

    pub(crate) async fn get(&self, k: &K) -> V {
        self.get_future(k).await
    }

    pub(crate) fn set(&self, k: &K, v: V) {
        let mut obj = self.0.lock().unwrap();
        if self.1 {
            if let Some(fuse) = obj.remove(k) {
                fuse.fuse(v);
            }
        } else {
            if let Some(fuse) = obj.get(k) {
                fuse.fuse(v);
            } else {
                let mut fuse = FusePromise::new();
                obj.insert(k.clone(),fuse.clone());
                fuse.fuse(v);
            }
        }
    }
}
