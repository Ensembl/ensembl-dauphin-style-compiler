use crate::{CarriageSpeed, Scale, core::{Layout, pixelsize::PixelSize}};

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct TrainExtent {
    layout: Layout,
    scale: Scale,
    pixel_size: PixelSize
}

//#[cfg(debug_assertions)]
impl std::fmt::Debug for TrainExtent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}/{}...",self.layout().stick().get_id(),self.scale().get_index())
    }
}

impl TrainExtent {
    pub fn new(layout: &Layout, scale: &Scale, pixel_size: &PixelSize) -> TrainExtent {
        TrainExtent {
            layout: layout.clone(),
            scale: scale.clone(),
            pixel_size: pixel_size.clone()
        }
    }

    pub fn layout(&self) -> &Layout { &self.layout }
    pub fn scale(&self) -> &Scale { &self.scale }
    pub fn pixel_size(&self) -> &PixelSize { &self.pixel_size }

    pub(super) fn speed_limit(&self, other: &TrainExtent) -> CarriageSpeed {
        let same_stick = self.layout().stick() == other.layout().stick();
        if same_stick {
            let same_layout = self.layout() == other.layout();
            if same_layout {
                CarriageSpeed::Quick
            } else {
                CarriageSpeed::SlowCrossFade
            }
        } else {
            CarriageSpeed::Slow
        }
    }
}
