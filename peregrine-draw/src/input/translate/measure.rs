use crate::{Message, PeregrineInnerAPI, stage::axis::ReadStageAxis};

pub(super) struct Measure {
    pub bp_per_screen: f64,
    pub px_per_screen: f64,
    pub x_bp: f64
}

impl Measure {
    pub(super) fn new(inner: &PeregrineInnerAPI) -> Result<Option<Measure>,Message> {
        let stage = inner.stage().lock().unwrap();
        if !stage.ready() { return Ok(None); }
        Ok(Some(Measure {
            bp_per_screen: stage.x().bp_per_screen()?,
            px_per_screen: stage.x().size()?,
            x_bp: stage.x().position()?
        }))
    }
}