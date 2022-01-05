use crate::input::translate::measure::Measure;
use super::regime::{RegimeCreator, RegimeTrait, TickResult};

pub(super) struct GotoRegimeCreator();

impl RegimeCreator for GotoRegimeCreator {
    type Object = GotoRegime;

    fn create(&self) -> Self::Object {
        GotoRegime::new()
    }
}

pub(crate) struct GotoRegime {
    goto: Option<(f64,f64)>
}

impl GotoRegime {
    pub(crate) fn new() -> GotoRegime {
        GotoRegime {
            goto: None
        }
    }

    pub(crate) fn goto(&mut self, x: f64, bp: f64) {
        self.goto = Some((x,bp));
    }
}

impl RegimeTrait for GotoRegime {
    fn set_size(&mut self, measure: &Measure, size: Option<f64>) {
    }

    fn tick(&mut self, measure: &Measure, total_dt: f64) -> TickResult {
        if let Some((centre,bp)) = self.goto.take() {
            TickResult::Update(Some(centre),Some(bp))
        } else {
            TickResult::Finished
        }
    }

    fn update_settings(&mut self, measure: &Measure) {
    }
}
