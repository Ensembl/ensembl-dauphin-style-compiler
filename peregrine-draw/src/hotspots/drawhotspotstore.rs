use peregrine_data::{SingleHotspotEntry, HotspotGroupEntry, SpaceBasePoint, AuxLeaf, SingleHotspotResult};
use peregrine_toolkit::{hotspots::hotspotstore::{HotspotStore, HotspotStoreProfile}, error::Error};
use crate::{Message};

pub(super) type PointPair = (SpaceBasePoint<f64,AuxLeaf>,SpaceBasePoint<f64,AuxLeaf>,Option<f64>);

pub(super) struct DrawHotspotStore<X> {
    store: HotspotStore<(f64,f64),PointPair,X,SingleHotspotEntry>
}

impl<X> DrawHotspotStore<X> {
    pub(super) fn new(profile: Box<dyn HotspotStoreProfile<SingleHotspotEntry,Area=PointPair,Coords=(f64,f64),Context=X>>, entries: &[HotspotGroupEntry]) -> Result<DrawHotspotStore<X>,Error> {
        let mut out = DrawHotspotStore {
            store: HotspotStore::new(profile)
        };
        out.init(entries);
        Ok(out)
    }

    fn init(&mut self, entries: &[HotspotGroupEntry]) {
        let mut ordering = 0;
        for entry in entries {
            if let Some(run) = entry.run() {
                if let Some(run) = run.iter(entry.area().len()) {
                    for (i,((top_left,bottom_right),run)) in entry.area().iter().zip(run).enumerate() {
                        let single_entry = SingleHotspotEntry::new(entry,i,ordering);
                        self.store.add(&(top_left.make(),bottom_right.make(),Some(*run)),single_entry);
                    }    
                }
            } else {
                for (i,(top_left,bottom_right)) in entry.area().iter().enumerate() {
                    let single_entry = SingleHotspotEntry::new(entry,i,ordering);
                    self.store.add(&(top_left.make(),bottom_right.make(),None),single_entry);
                }    
            }
            ordering += 1;
        }
    }

    pub(crate) fn get_hotspot(&self, context: &X, position_px: (f64,f64)) -> Result<Vec<SingleHotspotResult>,Message> {
        let mut candidates = self.store.get(context,&position_px)
            .drain(..).map(|entry| SingleHotspotResult {
                entry: entry.clone()
            }).collect::<Vec<_>>();
        candidates.sort();
        Ok(candidates)
    }
}
