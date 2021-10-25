use crate::{input::translate::{measure::Measure}};
use super::regime::{TickResult, RegimeCreator, RegimeTrait};

pub(super) struct SetRegimeCreator();

impl RegimeCreator for SetRegimeCreator {
    type Object = SetRegime;

    fn create(&self) -> Self::Object {
        SetRegime::new()
    }
}

pub(crate) struct SetRegime {
    target: Option<(f64,f64)>
}

impl SetRegime {
    pub(super) fn new() -> SetRegime {
        SetRegime {
            target: None
        }
    }

    pub(crate) fn set(&mut self, pos: f64, bp: f64) {
        self.target = Some((pos,bp));
    }
}

impl RegimeTrait for SetRegime {
    fn set_size(&mut self, _measure: &Measure, _size: Option<f64>) {}

    fn update_settings(&mut self, _measure: &Measure) {}

    fn tick(&mut self, _measure: &Measure, _total_dt: f64) -> TickResult {
        if let Some((x,bp)) = self.target.take() {
            TickResult::Update(Some(x),Some(bp))
        } else {
            TickResult::Finished
        }
    }
}
