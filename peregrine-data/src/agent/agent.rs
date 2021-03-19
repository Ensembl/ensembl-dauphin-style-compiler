use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::sync::{ Arc };
use crate::run::{ PgCommanderTaskSpec, add_task };
use crate::util::memoized::{ Memoized, MemoizedType, MemoizedDataResult };
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

#[derive(Clone)]
pub struct Agent<K,V> {
    store: Memoized<K,Result<V,DataMessage>>
}

impl<K,V> Agent<K,V> where K: Clone+Eq+Hash+Debug + 'static, V: 'static {
    pub fn new<F,W>(kind: MemoizedType, name: &str, prio: i8, base: &PeregrineCoreBase, agent_store: &AgentStore, callback: F) -> Agent<K,V>
                where F: Fn(PeregrineCoreBase,AgentStore,K) -> W+ 'static, W: Future<Output=Result<V,DataMessage>> {
        let agent_store = agent_store.clone();
        let base = base.clone();
        let name = name.to_string();
        let callback = Arc::new(callback);
        Agent {
            store: Memoized::new(kind, move |key: &K, result: MemoizedDataResult<K,Result<V,DataMessage>>| {
                let agent_store = agent_store.clone();
                let base = base.clone();
                let base2 = base.clone();
                let name = name.to_string();
                let key = key.clone();
                let callback = callback.clone();
                add_task(&base2.commander,PgCommanderTaskSpec {
                    name: format!("agent {} {:?}",name,key),
                    prio,
                    slot: None,
                    timeout: None,
                    task: Box::pin(async move {
                        let r = callback(base,agent_store,key).await;
                        result.resolve(r);
                        Ok(())
                    })
                });
            })
        }
    }

    pub async fn get(&self, k: &K) -> Arc<Result<V,DataMessage>> {
        self.store.get(k).await
    }

    pub fn get_no_wait(&self, k: &K) -> Result<(),DataMessage> {
        self.store.get_no_wait(k).map_err(|x| DataMessage::XXXTmp(x.to_string()))
    }
}
