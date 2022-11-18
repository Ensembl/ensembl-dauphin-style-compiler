use std::{sync::{Arc}};
use peregrine_data::{ Scale, HotspotGroupEntry, SingleHotspotEntry };
use crate::stage::{stage::{ ReadStage }};
use crate::util::message::Message;

use super::drawscreenhotspots::DrawScreenHotspots;

/* A major complication with using zones is dynamic rescaling and the ability for co-ordinates to include both
 * bp-scaling andpixel co-ordinates, meaning the hotspots can vary in which zones they intersect. Fortunately, as
 * an exact match is performed within the zones, it's enough to just take the _largest_ space occupiable by a hotspot.
 *
 * The zones are stored as fractions of a panel. As a panel corresponds to an exact number of bp, converting the bp
 * component to a zone is simple: divide by bp_in_carriage.
 */

pub struct DrawingHotspotsBuilder {
    entries: Vec<HotspotGroupEntry>,
    scale: Option<Scale>,
    left: f64
}

impl DrawingHotspotsBuilder {
    pub(crate) fn new(scale: Option<&Scale>, left: f64) -> DrawingHotspotsBuilder {
        DrawingHotspotsBuilder {
            entries: vec![],
            scale: scale.cloned(),
            left
        }
    }

    pub(crate) fn add_rectangle(&mut self, entry: HotspotGroupEntry) {
        self.entries.push(entry);
    }

    pub(crate) fn build(self) -> DrawingHotspots {
        DrawingHotspots::new(self)
    }
}

/* We are in the awkward position of having to take into account two coordinate systems with a
 * varying scale relationship to each other, at least in the x-axis -- pixels and base pair.
 * 
 * We maintain the current scale by recording max_bp_per_px. We account for pixels by converting
 * them to base-pairs at this "exchange rate". This is never an under-estimate, as required by our
 * general approach.
 * 
 * Calculating max_bp_per_px is tricky as it depends on the scale (indirectly giving the number of
 * base-pairs per screen) and the number of pixels per screen. Scale::bp_per_screen_range gives a
 * range for when a given scale will be drawn under normal circumstances, and we take its upper
 * limit as we are looking to maximise max_bp_per_px. We therefore need pixels per screen. This
 * can vary, perhaps continuously, as a user scales, but this will be a rare operation. To avoid
 * excessive recalculation, we bucket the number of base-pairs on the screen and only regenerate
 * when leaving that range. As we are maximising max_bp_per_px we need to use the minimum pixels
 * per screen within a given bucketed range.
 */

fn rounded_px_per_screen(px_per_screen: f64) -> f64 {
    let px_per_screen = px_per_screen.round() as u64;
    (px_per_screen.next_power_of_two() >> 1) as f64
}

pub struct DrawingHotspots {
    unscaled: Arc<DrawingHotspotsBuilder>,
    scaled: Option<(f64,DrawScreenHotspots)>
}

impl DrawingHotspots {
    fn new(builder: DrawingHotspotsBuilder) -> DrawingHotspots {
        DrawingHotspots {
            unscaled: Arc::new(builder),
            scaled: None
        }
    }

    pub(crate) fn set_px_per_screen(&mut self, px_per_screen: f64) {
        /* round into bucket */
        let mut new_min_pps = rounded_px_per_screen(px_per_screen);
        if let Some((old_min_pps,_)) = &self.scaled {
            if *old_min_pps == new_min_pps { return; }
            /* only let value decrease */
            new_min_pps = new_min_pps.min(*old_min_pps);
        }
        let screen = DrawScreenHotspots::new(new_min_pps,&self.unscaled.scale,self.unscaled.left,&self.unscaled.entries);
        self.scaled = screen.ok().map(|s| (new_min_pps,s));
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position_px: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        Ok(self.scaled.as_ref()
            .map(|scaled| scaled.1.get_hotspot(stage,position_px))
            .transpose()?.unwrap_or(vec![]))
    }
}
