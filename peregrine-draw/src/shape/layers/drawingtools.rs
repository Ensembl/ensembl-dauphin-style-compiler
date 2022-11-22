use std::sync::{Arc, Mutex};
use peregrine_data::{Assets, Scale};
use peregrine_toolkit::{error::Error, lock};
use crate::{shape::{core::{text::DrawingText, bitmap::DrawingBitmap, flatdrawing::FlatDrawingManager}}, webgl::{global::WebGlGlobal, CanvasWeave, DrawingCanvasesBuilder, canvas::{tessellate::canvastessellator::CanvasTessellator, imagecache::ImageCache}}, util::fonts::Fonts, hotspots::drawinghotspots::{DrawingHotspots, DrawingHotspotsBuilder}, Message};

const CANVAS_TYPE_LEN : usize = 3;

#[derive(Debug)]
pub(crate) enum CanvasType {
    Crisp ,
    HeraldryHoriz, // Horizontal lines
    HeraldryVert // Vertical Lines
}

impl CanvasType {
    fn from_index(index: usize) -> CanvasType {
        match index {
            0 => CanvasType::Crisp,
            1 => CanvasType::HeraldryHoriz,
            2 => CanvasType::HeraldryVert ,
            _ => panic!("bad canvastype index")           
        }
    }

    fn index(&self) -> usize {
        match self {
            CanvasType::Crisp => 0,
            CanvasType::HeraldryHoriz => 1,
            CanvasType::HeraldryVert => 2
        }
    }

    fn to_weave(&self) -> CanvasWeave {
        match self {
            CanvasType::Crisp => CanvasWeave::Crisp,
            CanvasType::HeraldryHoriz => CanvasWeave::VertStack, /* gets horizontal lines */
            CanvasType::HeraldryVert => CanvasWeave::HorizStack, /* gets vertical lines */
        }
    }
}

pub(crate) struct ToolPreparations {
    tessellators: Vec<CanvasTessellator>,
}

impl ToolPreparations {
    fn new() -> ToolPreparations {
        ToolPreparations {
            tessellators: (0..CANVAS_TYPE_LEN).map(|x| CanvasTessellator::new(&CanvasType::from_index(x).to_weave(),"uSampler")).collect(),
        }
    }

    pub(super) fn allocate(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        for i in 0..CANVAS_TYPE_LEN {
            self.tessellators[i].make(gl,drawable)?;
        }
        Ok(())
    }
}

pub(crate) struct DrawingTools {
    pub hotspots: DrawingHotspots
}

pub(crate) struct DrawingToolsBuilder {
    text: DrawingText,
    bitmap: DrawingBitmap,
    manager: Vec<FlatDrawingManager>,
    hotspots: DrawingHotspotsBuilder
}

impl DrawingToolsBuilder {
    pub(super) fn new(fonts: &Fonts, assets: &Assets, image_cache: &ImageCache, scale: Option<&Scale>, left: f64, bitmap_multiplier: f64) -> DrawingToolsBuilder {
        DrawingToolsBuilder {
            manager: (0..CANVAS_TYPE_LEN).map(|_| FlatDrawingManager::new()).collect(),
            text: DrawingText::new(fonts,bitmap_multiplier),
            bitmap: DrawingBitmap::new(assets,image_cache),
            hotspots: DrawingHotspotsBuilder::new(scale, left)
        }
    }

    pub(crate) fn manager(&mut self, type_: &CanvasType) -> &mut FlatDrawingManager { &mut self.manager[type_.index()] }
    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn bitmap(&mut self) -> &mut DrawingBitmap { &mut self.bitmap }
    pub(crate) fn hotspots(&mut self) -> &mut DrawingHotspotsBuilder { &mut self.hotspots }

    pub(crate) fn build(self) -> Result<DrawingTools,Message> {
        Ok(DrawingTools {
            hotspots: self.hotspots.build()?
        })
    }

    pub(crate) async fn start_preparation(&mut self, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<ToolPreparations,Error> {
        let mut preparations = ToolPreparations::new();
        self.text.prepare_for_allocation().await?;
        for i in 0..CANVAS_TYPE_LEN {
            self.manager[i].calculate_requirements(&mut *lock!(gl),&mut preparations.tessellators[i])?;
        }
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, mut preparations: ToolPreparations) -> Result<(),Error> {
        for i in 0..CANVAS_TYPE_LEN {
            self.manager[i].draw_at_locations(&mut preparations.tessellators[i])?;
        }
        Ok(())
    }
}
