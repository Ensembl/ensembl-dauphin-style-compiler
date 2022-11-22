use peregrine_data::{LeafStyle, SingleHotspotEntry, HotspotGroupEntry, SpaceBasePoint};
use peregrine_toolkit::{hotspots::hotspotstore::{HotSpotStore, HotspotStoreProfile}, log, error::Error};
use crate::{Message};

pub(super) type PointPair = (SpaceBasePoint<f64,LeafStyle>,SpaceBasePoint<f64,LeafStyle>);

pub(super) struct DrawHotspotStore<X> {
    store: HotSpotStore<(f64,f64),PointPair,X,SingleHotspotEntry>
}

impl<X> DrawHotspotStore<X> {
    pub(super) fn new(profile: Box<dyn HotspotStoreProfile<SingleHotspotEntry,Area=PointPair,Coords=(f64,f64),Context=X>>, entries: &[HotspotGroupEntry]) -> Result<DrawHotspotStore<X>,Error> {
        let mut out = DrawHotspotStore {
            store: HotSpotStore::new(profile)
        };
        out.init(entries);
        Ok(out)
    }

    fn init(&mut self, entries: &[HotspotGroupEntry]) {
        let mut ordering = 0;
        for entry in entries {
            for (i,(top_left,bottom_right)) in entry.area().iter().enumerate() {
                let entry = SingleHotspotEntry::new(entry,i,ordering);
                self.store.add(&(top_left.make(),bottom_right.make()),entry);
            }
            ordering += 1;
        }
    }

    pub(crate) fn get_hotspot(&self, context: &X, position_px: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        let mut candidates = self.store.get(context,&position_px)
            .drain(..).cloned().collect::<Vec<_>>();
        candidates.sort();
        Ok(candidates)
    }
}
