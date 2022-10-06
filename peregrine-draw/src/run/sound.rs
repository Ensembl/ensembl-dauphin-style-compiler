use peregrine_data::Asset;
use peregrine_data::Assets;
use peregrine_data::BackendNamespace;
use peregrine_toolkit::log_extra;
use peregrine_toolkit::plumbing::distributor::Distributor;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::js::promise::promise_to_future;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use js_sys::{ Uint8Array};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{ AudioContext, AudioBufferSourceNode, AudioBuffer, AudioContextState };
use commander::{CommanderStream, PromiseFuture};
use crate::{Message, PgCommanderWeb };

use super::PgConfigKey;
use super::PgPeregrineConfig;

#[derive(Clone)]
struct PromiseBgd(Arc<Mutex<Option<(Closure<dyn FnMut(JsValue)>,Closure<dyn FnMut(JsValue)>)>>>);

enum SoundQueueItem {
    Play(Option<BackendNamespace>,String),
    Shutdown
}

struct SoundState {
    assets: Assets,
    audio_context: Option<AudioContext>,
    samples: HashMap<String,Option<AudioBuffer>>
}

impl SoundState {
    fn audio_context(&mut self) -> Result<&AudioContext,JsValue> {
        Ok(self.audio_context.get_or_insert(AudioContext::new()?))
    }

    async fn make_source(&mut self, asset: &Asset) -> Result<Option<AudioBuffer>,JsValue> {
        let bytes = asset.bytes();
        if bytes.is_none() { return Ok(None); }
        let bytes = bytes.unwrap();
        let promise = self.audio_context()?.decode_audio_data(&Uint8Array::from(bytes.data().as_ref().as_ref()).buffer())?;
        let audio_buffer = promise_to_future(promise).await?.dyn_into::<AudioBuffer>()?;
        Ok(Some(audio_buffer))
    }

    async fn get_source<'a>(&'a mut self, name: &'a str, asset: &Asset) -> Result<Option<&AudioBuffer>,JsValue> {
        if !self.samples.contains_key(name) {
            let source = self.make_source(asset).await?;
            self.samples.insert(name.to_string(),source);
        }
        Ok(self.samples.get(name).unwrap().as_ref())
    }

    async fn try_play(&mut self, channel: Option<&BackendNamespace>, name: &str) -> Result<(),JsValue> {
        let asset = self.assets.get(channel,name);
        if asset.is_none() { return Ok(()); }
        let asset = asset.unwrap();
        promise_to_future(self.audio_context()?.resume()?).await.ok(); // handle autoplay-protection having stopped earlier sounds
        let source_node = AudioBufferSourceNode::new(self.audio_context()?)?;
        match self.audio_context()?.state() {
            AudioContextState::Running => {},
            _ => { return Ok(()); }
        }
        let audio_buffer = self.get_source(name,&asset).await?;
        if audio_buffer.is_none() { return Ok(()); }
        let audio_buffer = audio_buffer.unwrap();
        source_node.set_buffer(Some(&audio_buffer));
        let volume_node = self.audio_context()?.create_gain()?;
        let volume = asset.metadata("volume").map(|x| x.parse::<f32>()).transpose().ok().flatten().unwrap_or(1.);
        volume_node.gain().set_value(volume);
        source_node.connect_with_audio_node(&volume_node)?;
        volume_node.connect_with_audio_node(&self.audio_context()?.destination())?;
        let finished = PromiseFuture::new();
        let finished2 = finished.clone();
        let finished_closure = Closure::once(move || {
            finished2.satisfy(());
        });
        source_node.set_onended(Some(finished_closure.as_ref().unchecked_ref()));
        source_node.start()?;
        finished.await;
        source_node.stop()?;
        source_node.disconnect()?;
        volume_node.disconnect()?;
        promise_to_future(self.audio_context()?.suspend()?).await?;
        Ok(())
    }

    async fn play(&mut self, channel: Option<&BackendNamespace>, name: &str) {
        self.try_play(channel,name).await.ok();
    }
}

struct SoundComposer {
    sound: Sound,
    dinged: bool,
    ding_sound: Option<String>
}

impl SoundComposer {
    fn new(sound: &Sound, config: &PgPeregrineConfig) -> Result<SoundComposer,Message> {
        let ding_sound = config.get_str(&PgConfigKey::EndstopSound)?;
        let ding_sound = if ding_sound.is_empty() { None } else { Some(ding_sound.to_string()) };
        Ok(SoundComposer {
            sound: sound.clone(),
            dinged: false,
            ding_sound
        })
    }

    fn handle_message(&mut self, message: &Message) {
        match message {
            Message::HitEndstop(stops) => {
                if let Some(ding) = &self.ding_sound {
                    if stops.len() > 0 {
                        if !self.dinged {
                            self.sound.play(None,&ding);
                            self.dinged = true;
                        }
                    } else {
                        self.dinged = false;
                    }
                }
            },
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct Sound {
    queue: CommanderStream<SoundQueueItem>
}

impl Sound {
    async fn run_loop(&mut self, mut state: SoundState) -> Result<(),Message> {
        loop {
            match self.queue.get().await {
                SoundQueueItem::Play(channel,asset) => {
                    state.play(channel.as_ref(),&asset).await;
                },
                SoundQueueItem::Shutdown => { break; }
            }
        }
        log_extra!("sound queue shutdown");
        Ok(())
    }

    pub(crate) fn play(&mut self, channel: Option<&BackendNamespace>, sound: &str) {
        self.queue.add(SoundQueueItem::Play(channel.cloned(),sound.to_string()))
    }

    pub(crate) fn new(config: &PgPeregrineConfig, commander: &PgCommanderWeb, assets: &Assets, messages: &mut Distributor<Message>, shutdown: &OneShot) -> Result<Sound,Message> {
        let queue = CommanderStream::new();
        let state = SoundState {
            assets: assets.clone(),
            audio_context: None,
            samples: HashMap::new()
        };
        let queue2 = queue.clone();
        shutdown.add(move || {
            queue2.add(SoundQueueItem::Shutdown);
        });
        let out = Sound { queue };
        let mut out2 = out.clone();
        commander.add("sound",15,None,None,Box::pin(async move { out2.run_loop(state).await }));
        let mut composer = SoundComposer::new(&out,config)?;
        messages.add(move |m| composer.handle_message(m));
        Ok(out)
    }
}
