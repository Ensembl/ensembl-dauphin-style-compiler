use std::future::Future;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem::replace;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use anyhow::{ bail };
use commander::PromiseFuture;
use crate::run::pgcommander::{ PgCommander , PgCommanderTaskSpec };

struct SingleFileData<K,V> {
    commander: PgCommander,
    getter: Box<dyn Fn(&K) -> PgCommanderTaskSpec<V>>,
    promises: HashMap<K,PromiseFuture<anyhow::Result<V>>>
}

pub struct SingleFile<K,V>(Arc<Mutex<SingleFileData<K,V>>>);

fn convert_task<F,G>(a: PgCommanderTaskSpec<F>, task: Pin<Box<dyn Future<Output=anyhow::Result<G>> + 'static>>) -> PgCommanderTaskSpec<G> {
    PgCommanderTaskSpec {
        name: a.name,
        prio: a.prio,
        slot: a.slot,
        timeout: a.timeout,
        task
    }
}

async fn run_task<F,V>(cb: F, promise: PromiseFuture<V>) -> anyhow::Result<()> where F: Future<Output=V> {
    promise.satisfy(cb.await);
    Ok(())
}

impl<K,V> SingleFile<K,V> where K: Clone+Hash+PartialEq+Eq, V: 'static {
    pub fn new<F>(commander: &PgCommander, getter: F) -> SingleFile<K,V> where F: (Fn(&K) -> PgCommanderTaskSpec<V>) + 'static {
        SingleFile(Arc::new(Mutex::new(SingleFileData {
            commander: commander.clone(),
            getter: Box::new(getter),
            promises: HashMap::new()
        })))
    }

    pub async fn request(&self, key: K) -> anyhow::Result<V> {
        let mut data = self.0.lock().unwrap();
        if !data.promises.contains_key(&key) {
            let promise = PromiseFuture::new();
            let mut task = (data.getter)(&key);
            let task_task = replace(&mut task.task,Box::pin(async { bail!("placeholder") }));
            let task = convert_task(task,Box::pin(run_task(task_task,promise.clone())));
            data.promises.insert(key.clone(),promise);
        }
        data.promises.get_mut(&key).unwrap().clone().await
    }
}
