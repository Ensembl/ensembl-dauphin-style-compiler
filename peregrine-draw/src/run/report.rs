use std::{sync::{Arc, Mutex}};
use commander::{CommanderStream, cdr_tick};
use peregrine_data::{AllotterMetadata, ZMenuFixed};
use peregrine_toolkit::sync::needed::{Needed, NeededLock};
use crate::{Message, PgCommanderWeb };
use super::{PgConfigKey, PgPeregrineConfig};

struct Changed<T: PartialEq> {
    reported: Option<T>,
    unreported: Option<T>,
    #[allow(unused)]
    lock: Option<NeededLock>
}

impl<T: PartialEq+std::fmt::Debug> Changed<T> where T: PartialEq {
    fn new() -> Changed<T> {
        Changed {
            reported: None,
            unreported: None,
            lock: None
        }
    }

    fn set(&mut self, value: T, needed: &Needed) {
        self.unreported = Some(value);
        self.lock = Some(needed.lock());
    }
    fn is_changed(&mut self) -> bool { 
        let changed = self.unreported.is_some() && self.unreported != self.reported;
        if !changed { self.lock = None; }
        changed
     }
    fn peek(&self) -> Option<&T> { self.unreported.as_ref().or_else(|| self.reported.as_ref()) }

    fn report(&mut self, reuse: bool) -> Option<&T> {
        let mut update = false;
        if let Some(unreported) = self.unreported.take() {
            update = true;
            if let Some(reported) = self.reported.as_ref() {
                if reported == &unreported { update = false; }
            }
            self.reported = Some(unreported);
        }
        self.lock = None;
        if update || reuse {
            self.reported.as_ref()
        } else {
            None
        }
    }
}

fn extract_coord(stick: &mut Changed<String>, x: &mut Changed<f64>, bp: &mut Changed<f64>) -> Option<(String,f64,f64)> {
    if !x.is_changed() && !bp.is_changed() && !stick.is_changed() { return None; }
    if let (Some(stick),Some(x),Some(bp)) = (stick.report(true),x.report(true),bp.report(true)) {
        Some((stick.clone(),*x,*bp))
    } else {
        None
    }
}

fn to_left_right(position: f64, scale: f64) -> (u64,u64) {
    ((position-scale/2.).floor() as u64, (position+scale/2.).ceil() as u64)
}

struct ReportData {
    x_bp: Changed<f64>,
    bp_per_screen: Changed<f64>,
    target_x_bp: Changed<f64>,
    target_bp_per_screen: Changed<f64>,
    stick: Changed<String>,
    target_stick: Changed<String>,
    messages: CommanderStream<Message>,
    needed: Needed
}

impl ReportData {
    fn new(messages: &CommanderStream<Message>, needed: &Needed) -> ReportData {
        ReportData {
            x_bp: Changed::new(),
            bp_per_screen: Changed::new(),
            target_x_bp: Changed::new(),
            target_bp_per_screen: Changed::new(),
            stick: Changed::new(),
            target_stick: Changed::new(),
            messages: messages.clone(),
            needed: needed.clone()
        }
    }

    fn set_stick(&mut self, stick: &str) {
        self.stick.set(stick.to_string(),&self.needed);
        self.target_stick.set(stick.to_string(),&self.needed);
    }
    fn set_x_bp(&mut self, value: f64) { self.x_bp.set(value,&self.needed); }
    fn set_bp_per_screen(&mut self, value: f64) { self.bp_per_screen.set(value,&self.needed); }
    fn set_target_x_bp(&mut self, value: f64) { self.target_x_bp.set(value,&self.needed); }
    fn set_target_bp_per_screen(&mut self, value: f64) { self.target_bp_per_screen.set(value,&self.needed); }

    fn zmenu_event(&self, event: Vec<ZMenuFixed>) {
        self.messages.add(Message::ZMenuEvent(event));
    }

    fn build_messages(&mut self) -> Vec<Message> {
        let mut out = vec![];
        if let Some((stick,current_pos,current_scale)) = extract_coord(&mut self.stick,&mut self.x_bp,&mut self.bp_per_screen) {
            let (left,right) = to_left_right(current_pos,current_scale);
            out.push(Message::CurrentLocation(stick,left,right))
        }
        if let Some((stick,current_pos,current_scale)) = extract_coord(&mut self.target_stick,&mut self.target_x_bp,&mut self.target_bp_per_screen) {
            let (left,right) = to_left_right(current_pos,current_scale);
            out.push(Message::TargetLocation(stick,left,right))
        }
        out
    }

    fn set_allotter_metadata(&self, metadata: &AllotterMetadata) {
        self.messages.add(Message::AllotterMetadata(metadata.clone()));
    }

    fn report_step(&mut self) -> Result<(),Message> {
        for message in self.build_messages() {
            self.messages.add(message);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Report {
    data: Arc<Mutex<ReportData>>,
    needed: Needed,
    update_freq: f64
}

impl Report {
    async fn report_loop(&self) -> Result<(),Message> {
        loop {
            self.data.lock().unwrap().report_step()?;
            cdr_tick(self.update_freq as u64).await;
            self.needed.wait_until_needed().await;
        }

    }

    pub(crate) fn new(config: &PgPeregrineConfig, messages: &CommanderStream<Message>) -> Result<Report,Message> {
        let needed = Needed::new();
        Ok(Report {
            data: Arc::new(Mutex::new(ReportData::new(messages,&needed))),
            update_freq: config.get_f64(&PgConfigKey::ReportUpdateFrequency)?,
            needed
        })
    }

    pub(crate) fn set_stick(&self, value: &str) { self.data.lock().unwrap().set_stick(value); }
    pub(crate) fn set_x_bp(&self, value: f64) { self.data.lock().unwrap().set_x_bp(value); }
    pub(crate) fn set_bp_per_screen(&self, value: f64) { self.data.lock().unwrap().set_bp_per_screen(value); }
    pub(crate) fn set_target_x_bp(&self, value: f64) { self.data.lock().unwrap().set_target_x_bp(value); }
    pub(crate) fn set_target_bp_per_screen(&self, value: f64) { self.data.lock().unwrap().set_target_bp_per_screen(value); }

    pub(crate) fn set_allotter_metadata(&self, metadata: &AllotterMetadata) {
        self.data.lock().unwrap().set_allotter_metadata(metadata);
    }

    pub(crate) fn zmenu_event(&self, event: Vec<ZMenuFixed>) {
        self.data.lock().unwrap().zmenu_event(event);
    }

    pub(crate) fn run(&self, commander: &PgCommanderWeb) {
        let self2 = self.clone();
        commander.add("report", 0, None, None, Box::pin(async move { self2.report_loop().await }));
    }
}
