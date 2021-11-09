use std::sync::{ Arc };
use commander::PromiseFuture;

use crate::{DataMessage, PgCommanderTaskSpec, AuthorityStore, add_task, async_complete_task, core::stick::{ StickId }};
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };

async fn get_jump(stick_authority_store: &AuthorityStore, location: &str) -> Result<Option<(String,u64,u64)>,DataMessage> {
    stick_authority_store.try_location(location.clone()).await
}

async fn query_jump(stick_authority_store: AuthorityStore, location: &str) -> Result<Arc<(String,u64,u64)>,DataMessage> {
    let jump = get_jump(&stick_authority_store,location).await?.ok_or_else(||
        DataMessage::NoSuchJump(location.to_string())
    )?;
    Ok(Arc::new(jump))
}

fn make_jump_cache(stick_authority_store: &AuthorityStore) -> Memoized<String,Result<Arc<(String,u64,u64)>,DataMessage>> {
    let stick_authority_store = stick_authority_store.clone();
    Memoized::new(MemoizedType::Cache(128),move |_,location: &String| {
        let stick_authority_store = stick_authority_store.clone();
        let location = location.clone();
        Box::pin(async move { query_jump(stick_authority_store.clone(),&location).await })
    })   
}

#[derive(Clone)]
pub struct JumpStore(Memoized<String,Result<Arc<(String,u64,u64)>,DataMessage>>,PeregrineCoreBase);

impl JumpStore {
    pub fn new(base: &PeregrineCoreBase, stick_authority_store: &AuthorityStore) -> JumpStore {
        JumpStore(make_jump_cache(stick_authority_store),base.clone())
    }

    pub async fn get(&self, location: &String) -> Result<Arc<(String,u64,u64)>,DataMessage> {
        self.1.booted.wait().await;
        self.0.get(location).await.as_ref().clone()
    }

    pub(crate) fn jump(&self, location: &str, promise: PromiseFuture<Option<(StickId,f64,f64)>>) {
        let self2 = self.clone();
        let location = location.to_string();
        let handle = add_task(&self.1.commander,PgCommanderTaskSpec {
            name: "jump".to_string(),
            prio: 4,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                let (stick,left,right) = self2.get(&location).await?.as_ref().clone();
                let left = left as f64;
                let right = right as f64;
                promise.satisfy(Some((StickId::new(&stick),(left+right)/2.,right-left)));
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&self.1.commander,&self.1.messages,handle, |e| (e,false));

    }
}
