use std::{collections::HashMap, rc::Rc, sync::{Arc, Mutex}};
use peregrine_data::{ Scale, ZMenuGenerator, ZMenuProxy, SpaceBaseArea, SpaceBasePointRef, LeafStyle, Hotspot, SettingMode };
use peregrine_toolkit::{lock, eachorevery::EachOrEvery};
use crate::stage::{stage::{ ReadStage }, axis::UnitConverter};
use crate::util::message::Message;

const HORIZ_ZONES : u64 = 10;
const VERT_ZONE_HEIGHT : u64 = 200;

/* A major complication with using zones is dynamic rescaling and the ability for co-ordinates to include both
 * bp-scaling andpixel co-ordinates, meaning the hotspots can vary in which zones they intersect. Fortunately, as
 * an exact match is performed within the zones, it's enough to just take the _largest_ space occupiable by a hotspot.
 *
 * The zones are stored as fractions of a panel. As a panel corresponds to an exact number of bp, converting the bp
 * component to a zone is simple: divide by bp_in_carriage.
 *
 * An offset in px needs a conversion ratio of px_per_panel, or at least a maximum px_per_panel. This isn't something
 * that's stored very directly at all. We store bp_per_screen and px_per_screen in the stage. Together these can tell
 * us px_per_screen. However, that's not the same of px_per_panel: a screen can be composed of multiple or fractional
 * panels. We can obtain from scale the maximum bp_per_screen for which the drawing will be displayed. Changes to stage
 * can keep us updated in the number of px_per_screen for which we track the minimum as an order of magnitude. We
 * recompute when this changes (which should be once in a lbue moon) so for our computation can be considered
 * constant. Together, max bp_per_screen and min px_per_screen can give us max bp_per_px. We can then convert px into
 * max bp and use bp_in_carriage to convert to maximum proportion of carriage.
 */

pub struct SwitchProxy(Rc<EachOrEvery<(Vec<String>,bool)>>,usize);

impl SwitchProxy {
    pub fn value(&self) -> (Vec<String>,bool) {
        self.0.get(self.1).unwrap().clone()
    }
}

pub struct SettingProxy(Rc<EachOrEvery<(Vec<String>,SettingMode)>>,usize);

impl SettingProxy {
    pub fn value(&self) -> (Vec<String>,SettingMode) {
        self.0.get(self.1).unwrap().clone()
    }
}

#[derive(Clone)]
struct SwitchGenerator(Rc<EachOrEvery<(Vec<String>,bool)>>);

impl SwitchGenerator {
    fn new(values: &EachOrEvery<(Vec<String>,bool)>) -> SwitchGenerator {
        SwitchGenerator(Rc::new(values.clone()))
    }

    fn make_proxy(&self, index: usize) -> SwitchProxy {
        SwitchProxy(self.0.clone(),index)
    }
}

#[derive(Clone)]
struct SettingGenerator(Rc<EachOrEvery<(Vec<String>,SettingMode)>>);

impl SettingGenerator {
    fn new(values: &EachOrEvery<(Vec<String>,SettingMode)>) -> SettingGenerator {
        SettingGenerator(Rc::new(values.clone()))
    }

    fn make_proxy(&self, index: usize) -> SettingProxy {
        SettingProxy(self.0.clone(),index)
    }
}

enum HotspotUnscaledEntryDetails {
    ZMenu(ZMenuGenerator),
    Switch(SwitchGenerator),
    Setting(SettingGenerator)
}

struct HotspotUnscaledEntry {
    details: HotspotUnscaledEntryDetails,
    area: SpaceBaseArea<f64,LeafStyle>
}

impl HotspotUnscaledEntry {
    fn new(area: SpaceBaseArea<f64,LeafStyle>, hotspot: &Hotspot) -> HotspotUnscaledEntry {
        match hotspot {
            Hotspot::ZMenu(zmenu,values) => {
                let mut map_values = HashMap::new();
                for (k,v) in values.iter() {
                    map_values.insert(k.to_string(),v.clone());
                }
                let details = ZMenuGenerator::new(&zmenu,&map_values); // XXX push up
                HotspotUnscaledEntry {
                    details: HotspotUnscaledEntryDetails::ZMenu(details),
                    area
                }      
            },
            Hotspot::Switch(values) => {
                let details = SwitchGenerator::new(values);
                HotspotUnscaledEntry {
                    details: HotspotUnscaledEntryDetails::Switch(details),
                    area
                }
            },
            Hotspot::Setting(values) => {
                let details = SettingGenerator::new(values);
                HotspotUnscaledEntry {
                    details: HotspotUnscaledEntryDetails::Setting(details),
                    area
                }
            }
        }
    }
}

fn order(a: f64, b: f64) -> (f64,f64) {
    if a < b { (a,b) } else { (b,a) }
}

pub struct DrawingHotspotsBuilder {
    entries: Vec<HotspotUnscaledEntry>,
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

    pub(crate) fn add_rectangle(&mut self, area: SpaceBaseArea<f64,LeafStyle>, hotspot: &Hotspot) {
        self.entries.push(HotspotUnscaledEntry::new(area,hotspot));
    }

    pub(crate) fn build(self) -> DrawingHotspots {
        DrawingHotspots::new(self)
    }
}

#[derive(Clone)]
pub(crate) enum HotspotEntryDetails {
    ZMenu(Rc<ZMenuProxy>),
    Switch(Rc<SwitchProxy>),
    Setting(Rc<SettingProxy>)
}

impl HotspotEntryDetails {
    fn new(unscaled: &HotspotUnscaledEntryDetails, index: usize) -> HotspotEntryDetails {
        match unscaled {
            HotspotUnscaledEntryDetails::ZMenu(generator) => {
                HotspotEntryDetails::ZMenu(Rc::new(generator.make_proxy(index)))
            },
            HotspotUnscaledEntryDetails::Switch(generator) => {
                HotspotEntryDetails::Switch(Rc::new(generator.make_proxy(index)))
            },
            HotspotUnscaledEntryDetails::Setting(generator) => {
                HotspotEntryDetails::Setting(Rc::new(generator.make_proxy(index)))
            }
        }
    }
}

#[derive(Clone)]
struct HotspotEntry {
    area: SpaceBaseArea<f64,LeafStyle>,
    index: usize,
    order: usize,
    details: HotspotEntryDetails
}

impl HotspotEntry {
    fn new(unscaled: &HotspotUnscaledEntry, index: usize, order: usize) -> HotspotEntry {
        HotspotEntry {
            area: unscaled.area.clone(),
            index,
            order,
            details: HotspotEntryDetails::new(&unscaled.details,index)
        }
    }

    fn is_hotspot(&self, x_px: f64, y_px: f64, left: f64, bp_per_carriage: f64, px_per_carriage: f64, car_px_left: f64) -> bool {
        let mut iter = self.area.iter();
        if let Some((top_left,bottom_right)) = iter.nth(self.index) {
            let top_px = top_left.normal;
            let bottom_px = bottom_right.normal;
            let left_px = (top_left.base - left) / bp_per_carriage * px_per_carriage + car_px_left + top_left.tangent;
            let right_px = (bottom_right.base - left) / bp_per_carriage * px_per_carriage + car_px_left + bottom_right.tangent;
            return x_px >= left_px && x_px <= right_px && y_px >= *top_px && y_px < *bottom_px;
        }
        false
    }
}

struct ScaledHotspots {
    min_px_per_screen: f64,
    bp_in_carriage: f64,
    left: f64,
    max_bp_per_px: f64,
    zmenus: HashMap<u64,Rc<Vec<HotspotEntry>>>
}

impl ScaledHotspots {
    fn new(min_px_per_screen: f64, unscaled: &DrawingHotspotsBuilder) -> ScaledHotspots {
        let max_bp_per_screen = unscaled.scale.as_ref().map(|s| s.bp_per_screen_range().1).unwrap_or(1) as f64;
        let max_bp_per_px = max_bp_per_screen / min_px_per_screen;
        let mut out = ScaledHotspots {
            min_px_per_screen,
            bp_in_carriage: unscaled.scale.as_ref().map(|s| s.bp_in_carriage()).unwrap_or(1) as f64,
            max_bp_per_px,
            left: unscaled.left,
            zmenus: HashMap::new()
        };
        out.build_scaled(unscaled);
        out
    }

    fn maximum_footprint(&self, top_left: &SpaceBasePointRef<f64,LeafStyle>, bottom_right: &SpaceBasePointRef<f64,LeafStyle>) -> ((f64,u64),(f64,u64)) {
        /* y-coordinate */
        let (top_px,bottom_px) = order(*top_left.normal,*bottom_right.normal);
        /* x-coordinate */
        let (mut left_bp,mut right_bp) = order(*top_left.base,*bottom_right.base);
        if *top_left.tangent < 0. { left_bp += top_left.tangent * self.max_bp_per_px; }
        if *bottom_right.tangent > 0. { right_bp += bottom_right.tangent * self.max_bp_per_px; }
        let left_scr = (left_bp - self.left) / self.bp_in_carriage;
        let right_scr = (right_bp - self.left) / self.bp_in_carriage;
        ((left_scr,top_px as u64),(right_scr,(bottom_px+1.) as u64))
    }

    // TODO no-bp zmenus
    fn get_zones(&self, top_left: &SpaceBasePointRef<f64,LeafStyle>, bottom_right: &SpaceBasePointRef<f64,LeafStyle>) -> Vec<u64> {
        let ((left_scr,top_px),(right_scr,bottom_px)) = self.maximum_footprint(top_left,bottom_right);
        let mut out = vec![];
        for v_zone in (top_px/VERT_ZONE_HEIGHT)..((bottom_px/VERT_ZONE_HEIGHT)+1) {
            let left_zone = (left_scr*(HORIZ_ZONES as f64)).floor() as u64;
            let right_zone = (right_scr*(HORIZ_ZONES as f64)).floor() as u64;
            for h_zone in left_zone..(right_zone+1).min(HORIZ_ZONES) {
                out.push(v_zone*HORIZ_ZONES+h_zone);
            }
        }
        out
    }

    fn build_scaled(&mut self, unscaled: &DrawingHotspotsBuilder) -> Result<(),Message> {
        let mut order = 0;
        let mut building_zmenus = HashMap::new();
        for entry in &unscaled.entries {
            for (i,(top_left,bottom_right)) in entry.area.iter().enumerate() {
                let entry = HotspotEntry::new(entry,i,order);
                for zone in self.get_zones(&top_left,&bottom_right) {
                    building_zmenus.entry(zone).or_insert_with(|| vec![]).push(entry.clone());
                }
            }
            order += 1;
        }
        self.zmenus = building_zmenus.drain().map(|(k,v)| (k,Rc::new(v))).collect();
        Ok(())
    }
}

fn rounded_px_per_screen(px_per_screen: f64) -> f64 {
    let px_per_screen = px_per_screen.round() as u64;
    (px_per_screen.next_power_of_two() >> 1) as f64
}

pub struct DrawingHotspots {
    unscaled: Arc<DrawingHotspotsBuilder>,
    last_lookup: Mutex<Option<(u64,Rc<Vec<HotspotEntry>>)>>,
    min_px_per_screen: Option<f64>,
    scaled: Option<ScaledHotspots>,
    bp_in_carriage: u64,
    left: f64
}

// TODO mouse move needed set on screen resize
impl DrawingHotspots {
    fn new(builder: DrawingHotspotsBuilder) -> DrawingHotspots {
        DrawingHotspots {
            left: builder.left,
            bp_in_carriage: builder.scale.as_ref().map(|s| s.bp_in_carriage()).unwrap_or(1), // XXX unwrap
            unscaled: Arc::new(builder),
            min_px_per_screen: None,
            scaled: None,
            last_lookup: Mutex::new(None)
        }
    }

    pub(super) fn set_px_per_screen(&mut self, px_per_screen: f64) {
        let new_min_px_per_screen = rounded_px_per_screen(px_per_screen);
        let min_px_per_screen = self.min_px_per_screen
            .map(|old| old.min(new_min_px_per_screen))
            .unwrap_or(new_min_px_per_screen);    
        self.min_px_per_screen = Some(min_px_per_screen);    
        if let Some(scaled) = &self.scaled {
            if scaled.min_px_per_screen == min_px_per_screen { return; }
        }
        self.scaled = Some(ScaledHotspots::new(min_px_per_screen,&self.unscaled));
    }

    fn calculate_zone(&self, converter: &UnitConverter, position_px: (f64,f64)) -> Result<Option<u64>,Message> {
        let position_x_bp = converter.px_pos_to_bp(position_px.0);
        let bp_from_left = position_x_bp - self.left;
        if bp_from_left < 0. || bp_from_left >= self.bp_in_carriage as f64 { return Ok(None); }
        let carriage_prop = bp_from_left / self.bp_in_carriage as f64;
        let h_zone = (carriage_prop * HORIZ_ZONES as f64).floor() as u64;
        let v_zone = (position_px.1 / VERT_ZONE_HEIGHT as f64).floor() as u64;
        Ok(Some(h_zone + (v_zone * HORIZ_ZONES)))
    }

    fn get_zone_data(&self, zone: u64) -> Option<Rc<Vec<HotspotEntry>>> {
        let mut zone_data = None;
        let mut last_lookup = lock!(self.last_lookup);
        if let Some((last_zone,last_zone_data)) = last_lookup.as_ref() {
            if *last_zone == zone { zone_data = Some(last_zone_data.clone()); }
        }
        if zone_data.is_none() {
            zone_data = self.scaled.as_ref().and_then(|scaled| scaled.zmenus.get(&zone).cloned());
            if let Some(zone_data) = &zone_data {
                *last_lookup = Some((zone,zone_data.clone()));
            }
        }
        zone_data
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position_px: (f64,f64)) -> Result<Vec<HotspotEntryDetails>,Message> {
        let converter = stage.x().unit_converter()?;
        let zone = self.calculate_zone(&converter,position_px)?;
        let zone = if let Some(zone) = zone { zone } else { return Ok(vec![]); };
        let bp_per_px = converter.px_delta_to_bp(1.);
        let px_per_carriage = self.bp_in_carriage as f64 / bp_per_px;
        let left_px = converter.bp_to_pos_px(self.left)?;
        let zone_data = self.get_zone_data(zone);
        let mut out = vec![];
        if let Some(zone_data) = &zone_data {
            for entry in zone_data.iter() {
                if entry.is_hotspot(position_px.0,position_px.1,self.left,self.bp_in_carriage as f64,px_per_carriage,left_px as f64) {
                    out.push((entry.order,entry.details.clone()));
                }
            }
        }
        out.sort_by_cached_key(|v| v.0);
        Ok(out.drain(..).map(|x| x.1).collect())
    }
}
