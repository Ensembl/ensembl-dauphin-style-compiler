use peregrine_data::Asset;
use peregrine_data::Assets;
use peregrine_toolkit::plumbing::distributor::Distributor;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use js_sys::{ Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{ AudioContext, AudioBufferSourceNode, AudioBuffer, AudioContextState };
use commander::{CommanderStream, PromiseFuture};

use crate::{Message, PgCommanderWeb };

use super::PgConfigKey;
use super::PgPeregrineConfig;

async fn wrap_js_promise(promise: Promise) -> Result<JsValue,JsValue> {
    let result = PromiseFuture::new();
    let result2 = result.clone();
    let result3 = result.clone();
    let success = Closure::once(move |v| result2.satisfy(Ok(v)));
    let failure = Closure::once(move |v| result3.satisfy(Err(v)));
    let closure = promise.then2(&success,&failure);
    let out = result.await;
    drop(success);
    drop(failure);
    drop(closure);
    out
}

#[derive(Clone)]
struct PromiseBgd(Arc<Mutex<Option<(Closure<dyn FnMut(JsValue)>,Closure<dyn FnMut(JsValue)>)>>>);

fn wrap_js_promise_bgd(promise: Promise) {
    let holder = PromiseBgd(Arc::new(Mutex::new(None)));
    let holder2 = holder.clone();
    let success = Closure::once(move |v| drop(holder2));
    let holder2 = holder.clone();
    let failure = Closure::once(move |v| drop(holder2));
    let _ = promise.then2(&success,&failure);
    holder.0.lock().unwrap().replace((success,failure));
}

enum SoundQueueItem {
    Play(String)
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
        let promise = self.audio_context()?.decode_audio_data(&Uint8Array::from(bytes.as_ref().as_ref()).buffer())?;
        let audio_buffer = wrap_js_promise(promise).await?.dyn_into::<AudioBuffer>()?;
        Ok(Some(audio_buffer))
    }

    async fn get_source(&mut self, name: &str, asset: &Asset) -> Result<Option<&AudioBuffer>,JsValue> {
        if !self.samples.contains_key(name) {
            let source = self.make_source(asset).await?;
            self.samples.insert(name.to_string(),source);
        }
        Ok(self.samples.get(name).unwrap().as_ref())
    }

    async fn try_play(&mut self, name: &str) -> Result<(),JsValue> {
        let asset = self.assets.get(name);
        if asset.is_none() { return Ok(()); }
        let asset = asset.unwrap();
        wrap_js_promise(self.audio_context()?.resume()?).await.ok(); // handle autoplay-protection having stopped earlier sounds
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
        wrap_js_promise(self.audio_context()?.suspend()?).await?;
        Ok(())
    }

    async fn play(&mut self, name: &str) {
        self.try_play(name).await.ok();
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
                            self.sound.play(ding);
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
                SoundQueueItem::Play(asset) => {
                    state.play(&asset).await;
                }
            }
        }
    }

    pub(crate) fn play(&mut self, sound: &str) {
        self.queue.add(SoundQueueItem::Play(sound.to_string()))
    }

    pub(crate) fn new(config: &PgPeregrineConfig, commander: &PgCommanderWeb, assets: &Assets, messages: &mut Distributor<Message>) -> Result<Sound,Message> {
        let queue = CommanderStream::new();
        let state = SoundState {
            assets: assets.clone(),
            audio_context: None,
            samples: HashMap::new()
        };
        let out = Sound { queue };
        let mut out2 = out.clone();
        commander.add("sound",15,None,None,Box::pin(async move { out2.run_loop(state).await }));
        let mut composer = SoundComposer::new(&out,config)?;
        messages.add(move |m| composer.handle_message(m));
        Ok(out)
    }
}