use commander::{CommanderStream, FusePromise, PromiseFuture};
use peregrine_toolkit::{lock, error };
use crate::{ request::{minirequests::{bootchannelres::BootChannelRes, bootchannelreq::BootChannelReq}}, InstanceInformation, PeregrineCoreBase, shapeload::programloader::ProgramLoader, run::PgDauphinTaskSpec, DataMessage, add_task, PgCommanderTaskSpec, PacketPriority, CountingPromise, BackendNamespace};

use super::wrappedchannelsender::WrappedChannelSender;

type BootStream = CommanderStream<Option<(BackendNamespace,WrappedChannelSender,FusePromise<Result<BackendNamespace,DataMessage>>)>>;

async fn finish_bootstrap(response: &BootChannelRes, base: &PeregrineCoreBase, sender: &WrappedChannelSender, loader: &ProgramLoader) -> Result<BackendNamespace,DataMessage> {
    let info = InstanceInformation::new(
        response.namespace(),response,&base.version
    );
    base.channel_registry.register_channel(response.namespace(),sender);
    lock!(base.integration).report_instance_information(&info);
    let r = base.dauphin.run_program(&base.manager,loader,&base.channel_registry,PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        program_name: response.program_name().clone(),
        payloads: None
    }).await;
    if let Err(err) = r {
        error!("{}",err);
    }
    lock!(base.integration).set_assets(response.channel_assets());
    lock!(base.integration).set_assets(response.chrome_assets());
    base.queue.set_assets(response.channel_assets());
    base.queue.set_assets(response.chrome_assets());
    base.queue.regenerate_track_config();
    Ok(response.namespace().clone())
}

pub(super) async fn boot_channel(base: &PeregrineCoreBase, loader: &ProgramLoader, name: &BackendNamespace, sender: &WrappedChannelSender) -> Result<BackendNamespace,DataMessage> {
    let request = BootChannelReq::new();
    let response = base.manager.submit_direct(sender,&PacketPriority::RealTime,&Some(name.clone()),request, |v| {
        v.into_variety().into_boot_channel()
    }).await?;
    finish_bootstrap(&response,base,sender,loader).await
}

async fn boot_loop(stream: BootStream, base: &PeregrineCoreBase, loader: &ProgramLoader, booted: &CountingPromise) -> Result<(),DataMessage> {
    loop {
        if let Some((name,sender,promise)) = stream.get().await {
            promise.fuse(boot_channel(base,loader,&name,&sender).await);
        } else {
            booted.unlock();
        }
    }
}

#[derive(Clone)]
pub(crate) struct ChannelBoot {
    bootable: BootStream,
    booted: CountingPromise
}

impl ChannelBoot {
    pub(crate) fn new(booted: &CountingPromise) -> ChannelBoot {
        ChannelBoot {
            bootable: CommanderStream::new(),
            booted: booted.clone()
        }
    }

    pub(crate) async fn boot(&self, name: &BackendNamespace, sender: &WrappedChannelSender) -> Result<BackendNamespace,DataMessage> {
        let promise = FusePromise::new();
        self.bootable.add(Some((name.clone(),sender.clone(),promise.clone())));
        let p = PromiseFuture::new();
        promise.add(p.clone());
        p.await
    }

    pub(crate) async fn ready(&self) -> Result<(),DataMessage> {
        self.bootable.add(None);
        self.booted.wait().await;
        Ok(())
    }

    pub(crate) fn run_boot_loop(&self, base: &PeregrineCoreBase, loader: &ProgramLoader) {
        let base2 = base.clone();
        let loader = loader.clone();
        let booted = self.booted.clone();
        let stream = self.bootable.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: "bootstrap".to_string(),
            prio: 4,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                boot_loop(stream,&base2,&loader,&booted).await;
                Ok(())
            }),
            stats: false
        });    
    }
}
