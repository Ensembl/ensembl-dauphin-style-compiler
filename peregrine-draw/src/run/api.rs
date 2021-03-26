use crate::util::message::{ Message };
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use peregrine_data::{ Channel, Track, StickId };
use super::progress::Progress;

pub trait PeregrineDrawApi {
    fn set_message_reporter<F>(&mut self,callback: F) where F: FnMut(Message) + 'static;
    fn setup_blackbox(&self, url: &str) -> Result<(),Message>;
    fn x(&self) -> Result<f64,Message>;
    fn y(&self) -> Result<f64,Message>;
    fn size(&self) -> Result<(f64,f64),Message>;
    fn bp_per_screen(&self) -> Result<f64,Message>;
    fn set_x(&mut self, x: f64) -> Progress;
    fn set_y(&mut self, y: f64);
    fn set_size(&mut self, x: f64, y: f64);
    fn set_bp_per_screen(&mut self, z: f64) -> Progress;
    fn bootstrap(&self, channel: Channel);
    fn add_track(&self, track: Track) -> Progress;
    fn remove_track(&self, track: Track) -> Progress;
    fn set_stick(&self, stick: &StickId) -> Progress;
}
