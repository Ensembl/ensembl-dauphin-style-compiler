use std::{sync::{Arc, Mutex}};
use commander::{CommanderStream, cdr_tick, cdr_timer };
use eachorevery::eoestruct::StructValue;
use peregrine_data::{GlobalAllotmentMetadata, ZMenuFixed};
use peregrine_toolkit::{plumbing::oneshot::OneShot};
use peregrine_toolkit_async::sync::{needed::{Needed, NeededLock}, changed::Changed};
use crate::{Message, PgCommanderWeb, util::message::Endstop};
use super::{PgConfigKey, PgPeregrineConfig};

const TRIVIAL_PIXELS : f64 = 20000.; // if nothing would move more than 1px on a screen this size, ignore the change

fn trivial_change(x_from: f64, x_to: f64, bp_from: f64, bp_to: f64) -> bool {
    let (left_from,right_from) = to_left_right(x_from,bp_from);
    let (left_to,right_to) = to_left_right(x_to,bp_to);
    let screenful = bp_from.min(bp_to);
    let trivial_amount = screenful / TRIVIAL_PIXELS -1.;
    (left_to-left_from).abs() < trivial_amount && (right_to-right_from).abs() < trivial_amount
}

fn extract_coord(stick: &mut Changed<String>, x: &mut Changed<f64>, bp: &mut Changed<f64>) -> Option<(String,f64,f64)> {
    let nothing_changed = !x.is_changed() && !bp.is_changed() && !stick.is_changed();
    if nothing_changed { return None; }
    if let ((Some(x_from),Some(x_to)),(Some(bp_from),Some(bp_to))) = (x.peek(),bp.peek()) {
        if trivial_change(*x_from,*x_to,*bp_from,*bp_to) { return None; }
    }
    if let (Some(stick),Some(x),Some(bp)) = (stick.report(true),x.report(true),bp.report(true)) {
        Some((stick.clone(),*x,*bp))
    } else {
        None
    }
}

fn to_left_right(position: f64, scale: f64) -> (f64,f64) {
    ((position-scale/2.), (position+scale/2.))
}

struct ReportData {
    x_bp: Changed<f64>,
    bp_per_screen: Changed<f64>,
    target_x_bp: Changed<f64>,
    target_bp_per_screen: Changed<f64>,
    stick: Changed<String>,
    target_stick: Changed<String>,
    endstop: Changed<Vec<Endstop>>,
    messages: CommanderStream<Option<Message>>,
    /* If we are delaying then we need to keep the main loop alive */
    needed: Needed,
    #[allow(unused)]
    fast_lock: Option<NeededLock>
}

impl ReportData {
    fn new(messages: &CommanderStream<Option<Message>>, needed: &Needed) -> ReportData {
        ReportData {
            x_bp: Changed::new(),
            bp_per_screen: Changed::new(),
            target_x_bp: Changed::new(),
            target_bp_per_screen: Changed::new(),
            stick: Changed::new(),
            target_stick: Changed::new(),
            endstop: Changed::new(),
            messages: messages.clone(),
            needed: needed.clone(),
            fast_lock: None
        }
    }

    fn set_stick(&mut self, stick: &str) {
        self.fast_lock();
        self.stick.set(stick.to_string());
    }

    fn set_target_stick(&mut self, stick: &str) {
        self.fast_lock();
        self.target_stick.set(stick.to_string());
    }

    fn fast_lock(&mut self) {
        self.fast_lock = Some(self.needed.lock());
    }

    fn set_x_bp(&mut self, value: f64) { self.fast_lock(); self.x_bp.set(value); }
    fn set_bp_per_screen(&mut self, value: f64) { self.fast_lock(); self.bp_per_screen.set(value); }
    fn set_target_x_bp(&mut self, value: f64) { self.fast_lock(); self.target_x_bp.set(value); }
    fn set_target_bp_per_screen(&mut self, value: f64) { self.fast_lock(); self.target_bp_per_screen.set(value); }
    fn set_endstops(&mut self, value: &[Endstop]) {  self.fast_lock(); self.endstop.set(value.to_vec()); }

    fn hotspot_event(&self, x: f64, y: f64, varieties: &[StructValue], content: &[StructValue]) {
        self.messages.add(Some(Message::HotspotEvent(x,y,varieties.to_vec(),content.to_vec())));
    }

    fn zmenu_event(&self, x: f64, y: f64, event: Vec<ZMenuFixed>) {
        self.messages.add(Some(Message::ZMenuEvent(x,y,event)));
    }

    fn build_messages(&mut self, fast: bool) -> Vec<Message> {
        let mut out = vec![];
        if !fast {
            if let Some((stick,current_pos,current_scale)) = extract_coord(&mut self.stick,&mut self.x_bp,&mut self.bp_per_screen) {
                let (left,right) = to_left_right(current_pos,current_scale);
                out.push(Message::CurrentLocation(stick,left,right));
            }
            if let Some((stick,current_pos,current_scale)) = extract_coord(&mut self.target_stick,&mut self.target_x_bp,&mut self.target_bp_per_screen) {
                let (left,right) = to_left_right(current_pos,current_scale);
                out.push(Message::TargetLocation(stick,left,right));
            }
            self.fast_lock = None;
        }
        if let Some(endstops) = self.endstop.report(false) {
            out.push(Message::HitEndstop(endstops.to_vec()));
        }
        out
    }

    fn set_allotter_metadata(&self, metadata: &GlobalAllotmentMetadata) {
        self.messages.add(Some(Message::AllotmentMetadataReport(metadata.clone())));
    }

    fn report_step(&mut self, fast: bool) -> Result<(),Message> {
        for message in self.build_messages(fast) {
            self.messages.add(Some(message));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Report {
    data: Arc<Mutex<ReportData>>,
    needed: Needed,
    update_freq: f64,
    shutdown: OneShot
}

impl Report {
    async fn report_loop(&self) -> Result<(),Message> {
        while !self.shutdown.poll() {
            self.data.lock().unwrap().report_step(false)?;
            cdr_timer(self.update_freq).await;
            self.needed.wait_until_needed().await;
        }
        self.needed.set(); // make sure other thread wakes up
        Ok(())
    }

    async fn fast_report_loop(&self) -> Result<(),Message> {
        while !self.shutdown.poll() {
            self.data.lock().unwrap().report_step(true)?;
            cdr_tick(1).await;
            self.needed.wait_until_needed().await;
        }
        self.needed.set(); // make sure other thread wakes up
        Ok(())
    }

    pub(crate) fn new(config: &PgPeregrineConfig, messages: &CommanderStream<Option<Message>>, shutdown: &OneShot) -> Result<Report,Message> {
        let needed = Needed::new();
        let needed2 = needed.clone();
        shutdown.add(move || {
            needed2.set();
        });
        Ok(Report {
            data: Arc::new(Mutex::new(ReportData::new(messages,&needed))),
            update_freq: config.get_f64(&PgConfigKey::ReportUpdateFrequency)?,
            shutdown: shutdown.clone(),
            needed
        })
    }

    pub(crate) fn set_stick(&self, value: &str) { self.data.lock().unwrap().set_stick(value); }
    pub(crate) fn set_x_bp(&self, value: f64) { self.data.lock().unwrap().set_x_bp(value); }
    pub(crate) fn set_bp_per_screen(&self, value: f64) { self.data.lock().unwrap().set_bp_per_screen(value); }
    pub(crate) fn set_target_stick(&self, value: &str) { self.data.lock().unwrap().set_target_stick(value); }
    pub(crate) fn set_target_x_bp(&self, value: f64) { self.data.lock().unwrap().set_target_x_bp(value); }
    pub(crate) fn set_target_bp_per_screen(&self, value: f64) { self.data.lock().unwrap().set_target_bp_per_screen(value); }
    pub(crate) fn set_endstops(&self,value: &[Endstop]) { self.data.lock().unwrap().set_endstops(value); }

    pub(crate) fn set_allotter_metadata(&self, metadata: &GlobalAllotmentMetadata) {
        self.data.lock().unwrap().set_allotter_metadata(metadata);
    }

    pub(crate) fn hotspot_event(&self, x: f64, y: f64, varieties: &[StructValue], content: &[StructValue]) {
        self.data.lock().unwrap().hotspot_event(x,y,varieties,content);
    }

    pub(crate) fn zmenu_event(&self, x: f64, y: f64, event: Vec<ZMenuFixed>) {
        self.data.lock().unwrap().zmenu_event(x,y,event);
    }

    pub(crate) fn run(&self, commander: &PgCommanderWeb, shutdown: &OneShot) {
        let self2 = self.clone();
        commander.add("report", 5, None, None, Box::pin(async move { self2.report_loop().await }));
        let self2 = self.clone();
        commander.add("report-fast", 5, None, None, Box::pin(async move { self2.fast_report_loop().await }));
    }
}
