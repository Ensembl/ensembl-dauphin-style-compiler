use crate::core::{ Focus, PeregrineData, StickId, Track, Viewport };
use crate::PgCommanderTaskSpec;
use commander::CommanderStream;

pub enum ApiMessage {
    TransitionComplete,
    AddTrack(Track),
    RemoveTrack(Track),
    SetPosition(f64),
    SetScale(f64),
    SetFocus(Focus),
    SetStick(StickId)
}

#[derive(Clone)]
pub struct PeregrineApiQueue {
    queue: CommanderStream<ApiMessage>,
    data: PeregrineData
}

impl PeregrineApiQueue {
    pub fn new(data: &PeregrineData) -> PeregrineApiQueue {
        PeregrineApiQueue {
            queue: CommanderStream::new(),
            data: data.clone()
        }
    }

    fn update_train_set(&mut self) {
        let viewport = self.data.viewport.clone();
        let train_set = self.data.train_set.clone();
        train_set.set(&mut self.data, &viewport);
    }

    fn update_viewport(&mut self, new_viewport: Viewport) {
        if new_viewport != self.data.viewport {
            self.data.viewport = new_viewport;
            self.update_train_set();
        }
    }

    fn run_message(&mut self, message: ApiMessage) {
        match message {
            ApiMessage::TransitionComplete => {
                let train_set = self.data.train_set.clone();
                train_set.transition_complete(&mut self.data);
            },
            ApiMessage::AddTrack(track) => {
                self.update_viewport(self.data.viewport.track_on(&track,true));
            },
            ApiMessage::RemoveTrack(track) => {
                self.update_viewport(self.data.viewport.track_on(&track,false));
            },
            ApiMessage::SetPosition(pos) =>{
                self.update_viewport(self.data.viewport.set_position(pos));
            },
            ApiMessage::SetScale(scale) => {
                self.update_viewport(self.data.viewport.set_scale(scale));
            },
            ApiMessage::SetFocus(focus) => {
                self.update_viewport(self.data.viewport.set_focus(&focus));
            },
            ApiMessage::SetStick(stick) => {
                self.update_viewport(self.data.viewport.set_stick(&stick));
            }
        }
    }

    pub fn run(&self) {
        let mut self2 = self.clone();
        self.data.commander.add_task(PgCommanderTaskSpec {
            name: format!("api message runner"),
            prio: 5,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                loop {
                    let message = self2.queue.get().await;
                    self2.run_message(message);
                }
            })
        });
    }

    pub fn push(&self, message: ApiMessage) {
        self.queue.add(message);
    }
}
