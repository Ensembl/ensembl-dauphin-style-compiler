use std::sync::{Arc, Mutex};
use peregrine_data::{Assets, Scale};
use peregrine_toolkit::error::Error;
use crate::{shape::{core::{text::DrawingText, bitmap::DrawingBitmap, flatdrawing::FlatDrawingManager}, heraldry::heraldry::DrawingHeraldry}, webgl::{global::WebGlGlobal, CanvasWeave, DrawingCanvasesBuilder, canvas::{tessellate::canvastessellator::CanvasTessellator, imagecache::ImageCache}}, util::fonts::Fonts, hotspots::drawinghotspots::{DrawingHotspots, DrawingHotspotsBuilder}, Message};

pub(crate) struct ToolPreparations {
    crisp: CanvasTessellator,
    heraldry_h: CanvasTessellator,
    heraldry_v: CanvasTessellator
}

impl ToolPreparations {
    fn new() -> ToolPreparations {
        ToolPreparations {
            crisp: CanvasTessellator::new(&CanvasWeave::Crisp,"uSampler"),
            heraldry_h: CanvasTessellator::new(&CanvasWeave::HorizStack,"uSampler"),
            heraldry_v: CanvasTessellator::new(&CanvasWeave::VertStack,"uSampler"),
        }
    }

    pub(crate) fn crisp_manager(&mut self) -> &mut CanvasTessellator { &mut self.crisp }
    pub(crate) fn heraldry_h_manager(&mut self) -> &mut CanvasTessellator { &mut self.heraldry_h }
    pub(crate) fn heraldry_v_manager(&mut self) -> &mut CanvasTessellator { &mut self.heraldry_v }

    pub(super) fn allocate(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        self.crisp.make(gl,drawable)?;
        self.heraldry_h.make(gl,drawable)?;
        self.heraldry_v.make(gl,drawable)?;
        Ok(())
    }
}

pub(crate) struct DrawingTools {
    pub hotspots: DrawingHotspots
}

pub(crate) struct DrawingToolsBuilder {
    text: DrawingText,
    bitmap: DrawingBitmap,
    heraldry: DrawingHeraldry,
    hotspots: DrawingHotspotsBuilder
}

impl DrawingToolsBuilder {
    pub(super) fn new(fonts: &Fonts, assets: &Assets, image_cache: &ImageCache, scale: Option<&Scale>, left: f64, bitmap_multiplier: f64) -> DrawingToolsBuilder {
        let manager = FlatDrawingManager::new();
        DrawingToolsBuilder {
            text: DrawingText::new(fonts,bitmap_multiplier),
            bitmap: DrawingBitmap::new(assets,image_cache),
            heraldry: DrawingHeraldry::new(),
            hotspots: DrawingHotspotsBuilder::new(scale, left)
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn bitmap(&mut self) -> &mut DrawingBitmap { &mut self.bitmap }
    pub(crate) fn heraldry(&mut self) -> &mut DrawingHeraldry { &mut self.heraldry }
    pub(crate) fn hotspots(&mut self) -> &mut DrawingHotspotsBuilder { &mut self.hotspots }

    pub(crate) fn build(self) -> Result<DrawingTools,Message> {
        Ok(DrawingTools {
            hotspots: self.hotspots.build()?
        })
    }

    pub(crate) async fn start_preparation(&mut self, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<ToolPreparations,Error> {
        let mut preparations = ToolPreparations::new();
        self.text.calculate_requirements(gl,&mut preparations.crisp).await?;
        self.bitmap.calculate_requirements(gl, &mut preparations.crisp).await?;
        self.heraldry.calculate_requirements(gl,&mut preparations).await?;
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, mut preparations: ToolPreparations) -> Result<(),Error> {
        self.text.manager().draw_at_locations(&mut preparations.crisp)?;
        self.bitmap.manager().draw_at_locations(&mut preparations.crisp)?;
        self.heraldry.draw_at_locations(&mut preparations)?;
        Ok(())
    }
}
