use std::sync::{ Arc };

#[derive(Debug)]
struct MarchingAntsState {
    tlbr: (f64,f64,f64,f64)
}

#[derive(Clone,Debug)]
pub(crate) struct MarchingAnts(Arc<MarchingAntsState>);

impl MarchingAnts {
    pub(crate) fn new(tlbr: (f64,f64,f64,f64)) -> MarchingAnts {
        let tlbr = (
            tlbr.0.min(tlbr.2),
            tlbr.1.min(tlbr.3),
            tlbr.0.max(tlbr.2),
            tlbr.1.max(tlbr.3),
        );
        MarchingAnts(Arc::new(MarchingAntsState {
            tlbr
        }))
    }
}

#[derive(Clone,Debug)]
pub(crate) enum Spectre {
    MarchingAnts(MarchingAnts)
}
