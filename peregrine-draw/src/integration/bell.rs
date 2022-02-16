use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use crate::util::message::{ Message };
use super::{timer::Timer, custom::{Custom, CustomSender}};

#[derive(Clone)]
pub struct BellSender {
    sender: CustomSender
}

impl BellSender {
    fn new(sender: CustomSender) -> Result<BellSender,Message> {
        Ok(BellSender {
            sender
        })
    }

    pub fn ring(&self) { self.sender.dispatch(); }
}

struct BellReceiverState {
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + 'static>>>>,
    custom: Custom
}

fn run_callbacks(callbacks: &Arc<Mutex<Vec<Box<dyn Fn()>>>>) {
    for cb in callbacks.lock().unwrap().iter_mut() {
        (cb)();
    }
}

impl BellReceiverState {
    fn new() -> Result<BellReceiverState,Message> {
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        let callbacks2 = callbacks.clone();
        let mut timer = Timer::new(move || { run_callbacks(&callbacks2); });
        let custom =  Custom::new(move || {
            timer.go(0);
        });
        let out = BellReceiverState {
            callbacks,
            custom
        };
        Ok(out)
    }

    fn add(&mut self, callback: Box<dyn Fn() + 'static>) {
        self.callbacks.lock().unwrap().push(callback);
    }

    pub fn sender(&self) -> CustomSender { self.custom.sender() }
}

#[derive(Clone)]
pub struct BellReceiver(Arc<Mutex<BellReceiverState>>);

impl BellReceiver {
    fn new() -> Result<BellReceiver,Message> {
        Ok(BellReceiver(Arc::new(Mutex::new(BellReceiverState::new()?))))
    }

    pub fn add<T>(&mut self, callback: T) where T: Fn() + 'static {
        self.0.lock().unwrap().add(Box::new(callback));
    }

    pub fn sender(&self) -> CustomSender { lock!(self.0).sender() }
}

pub fn make_bell() -> Result<(BellSender,BellReceiver),Message> {
    let receiver = BellReceiver::new()?;
    Ok((BellSender::new(receiver.sender())?,receiver))
}
