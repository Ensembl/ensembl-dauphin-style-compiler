use crate::{Region, switch::trackconfig::TrackConfig, core::pixelsize::PixelSize, ShapeRequest, allotment::core::rangeused::RangeUsed};

#[derive(Clone)]
pub struct BpPxConverter {
    bp_per_carriage: f64,
    min_px_per_carriage: f64,
    bp_start: f64
}

impl BpPxConverter {
    fn new(region: &Region, pixel_size: &PixelSize) -> BpPxConverter {
        BpPxConverter {
            bp_per_carriage: region.scale().bp_in_carriage() as f64,
            min_px_per_carriage: pixel_size.min_px_per_carriage() as f64,
            bp_start: region.min_value() as f64
        }
    }

    pub(crate) fn full_carriage_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>) -> RangeUsed<f64> {
        base_range.plus_scalar(-self.bp_start).carriage_range(pixel_range,self.min_px_per_carriage,self.bp_per_carriage)
    }
}

#[derive(Clone)]
pub struct ShapeRequestGroup {
    region: Region,
    tracks: Vec<TrackConfig>,
    pixel_size: PixelSize,
    warm: bool,
    bp_px_converter: BpPxConverter
}

impl ShapeRequestGroup {
    pub fn new(region: &Region, tracks: &[TrackConfig], pixel_size: &PixelSize, warm: bool) -> ShapeRequestGroup {
        ShapeRequestGroup {
            bp_px_converter: BpPxConverter::new(region,pixel_size),
            region: region.clone(),
            tracks: tracks.to_vec(),
            pixel_size: pixel_size.clone(),
            warm,
        }
    }

    pub fn region(&self) -> &Region { &self.region }
    pub fn pixel_size(&self) -> &PixelSize { &self.pixel_size }
    pub fn warm(&self) -> bool { self.warm }

    pub fn full_carriage_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>) -> RangeUsed<f64> {
        self.bp_px_converter.full_carriage_range(base_range,pixel_range)
    }

    pub fn iter(&self) -> impl Iterator<Item=ShapeRequest> + '_ {
        let self2 = self.clone();
        self.tracks.iter().map(move |track| {
            ShapeRequest::new(&self2.region,track,&self2.pixel_size,self2.warm)
        })
    }
}
