use std::sync::{ Arc };
use crate::{DataMessage, PgCommanderTaskSpec, StickAuthorityStore, add_task, api::ApiMessage, async_complete_task, core::stick::{ StickId }};
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };

async fn get_jump(stick_authority_store: &StickAuthorityStore, location: &str) -> Result<Option<(String,u64,u64)>,DataMessage> {
    stick_authority_store.try_location(location.clone()).await
}

async fn query_jump(stick_authority_store: StickAuthorityStore, location: &str) -> Result<Arc<(String,u64,u64)>,DataMessage> {
    let jump = get_jump(&stick_authority_store,location).await?.ok_or_else(||
        DataMessage::NoSuchJump(location.to_string())
    )?;
    Ok(Arc::new(jump))
}

fn make_jump_cache(stick_authority_store: &StickAuthorityStore) -> Memoized<String,Result<Arc<(String,u64,u64)>,DataMessage>> {
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
    pub fn new(base: &PeregrineCoreBase, stick_authority_store: &StickAuthorityStore) -> JumpStore {
        JumpStore(make_jump_cache(stick_authority_store),base.clone())
    }

    pub async fn get(&self, location: &String) -> Result<Arc<(String,u64,u64)>,DataMessage> {
        self.1.booted.wait().await;
        self.0.get(location).await.as_ref().clone()
    }

    pub(crate) fn jump(&self, location: &str) {
        let self2 = self.clone();
        let location = location.to_string();
        let queue = self.1.queue.clone();
        let handle = add_task(&self.1.commander,PgCommanderTaskSpec {
            name: "jump".to_string(),
            prio: 4,
            timeout: None,
            slot: None,
            task: Box::pin(async move { 
                let (stick,left,right) = self2.get(&location).await?.as_ref().clone();
                let left = left as f64;
                let right = right as f64;
                queue.push(ApiMessage::SetStick(StickId::new(&stick)));
                queue.push(ApiMessage::SetPosition((left+right)/2.));
                queue.push(ApiMessage::SetBpPerScreen(right-left));
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&self.1.commander,&self.1.messages,handle, |e| (e,false));

    }
}