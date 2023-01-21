use eachorevery::EachOrEvery;
use peregrine_data::{ Colour, DrawnType, Patina, RectangleShape, Shape, ShapeDemerge, HollowEdge2, AuxLeaf, DrawingShape, CoordinateSystem, PolygonShape };
use peregrine_toolkit::error::Error;
use crate::shape::canvasitem::heraldry::{HeraldryCanvasesUsed, Heraldry};
use crate::shape::canvasitem::text::prepare_text;
use crate::shape::core::drawshape::{SimpleShapePatina};
use crate::shape::layers::drawingtools::{DrawingToolsBuilder, CanvasType};
use crate::shape::triangles::drawgroup::{DrawGroup, ShapeCategory};
use super::drawshape::{ GLShape };

fn split_spacebaserect(tools: &mut DrawingToolsBuilder, shape: &RectangleShape<AuxLeaf>, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Error> {
    let mut out = vec![];
    let depth = shape.area().top_left().allotments().map(|x| x.depth);
    let wobble = shape.wobble().clone();
    match shape.patina() {
        Patina::Drawn(drawn_variety,_) => {
            let width = match drawn_variety {
                DrawnType::Stroke(w) => Some(*w),
                DrawnType::Fill => None
            };
            match draw_group.shape_category() {
                ShapeCategory::SolidColour | ShapeCategory::Other => {
                    out.push(GLShape::SpaceBaseRect(shape.area().clone(),shape.run().clone(),SimpleShapePatina::from_patina(shape.patina())?,depth,draw_group.clone(),wobble));
                },
                ShapeCategory::Heraldry(HeraldryCanvasesUsed::Solid(heraldry_canvas),scale) => {
                    let heraldry = make_heraldry(shape.patina())?;
                    let handles = heraldry.map_results(|x| x.add(tools))?;
                    out.push(GLShape::Heraldry(shape.area().clone(),shape.run().clone(),handles,depth,draw_group.clone(),heraldry_canvas.clone(),scale.clone(),None,wobble));
                },
                ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale) => {
                    let width = width.unwrap_or(0.);
                    let heraldry = make_heraldry(shape.patina())?;
                    let handles = heraldry.map_results(|x| x.add(tools))?;
                    // XXX too much cloning, at least Arc them
                    let area = shape.area();
                    let run = shape.run();
                    out.push(GLShape::Heraldry(area.clone(),run.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge2::Left(width)),wobble.clone()));
                    out.push(GLShape::Heraldry(area.clone(),run.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge2::Right(width)),wobble.clone()));
                    out.push(GLShape::Heraldry(area.clone(),run.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge2::Top(width)),wobble.clone()));
                    out.push(GLShape::Heraldry(area.clone(),run.clone(),handles,depth,draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge2::Bottom(width)),wobble.clone()));
                }
            }
        },
        Patina::Hotspot(hotspot) => {
            out.push(GLShape::SpaceBaseRect(shape.area().clone(),shape.run().clone(),SimpleShapePatina::Hotspot(hotspot.clone()),depth,draw_group.clone(),None));
        },
        Patina::Metadata(_,_) => {}
    }
    Ok(out)
}

fn split_polygon(shape: &PolygonShape<AuxLeaf>, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Error> {
    let mut out = vec![];
    let wobble = shape.wobble().clone();
    match shape.patina() {
        Patina::Drawn(_,_) => {
            out.push(GLShape::Polygon(shape.position().clone(),shape.radius().clone(),draw_group.depth(),shape.points(),shape.angle(),SimpleShapePatina::from_patina(shape.patina())?,draw_group.clone(),wobble));
        },
        Patina::Hotspot(hotspot) => {
            out.push(GLShape::Polygon(shape.position().clone(),shape.radius().clone(),draw_group.depth(),shape.points(),shape.angle(),SimpleShapePatina::Hotspot(hotspot.clone()),draw_group.clone(),None));
        },
        Patina::Metadata(_,_) => {}
    }
    Ok(out)
}

fn colour_to_heraldry(colour: &Colour, _hollow: bool) -> Option<Heraldry> {
    match colour {
        Colour::Stripe(a,b,c,_prop) => {
            Some(Heraldry::Stripe(a.clone(),b.clone(),50,*c))
        },
        Colour::Bar(a,b,c,prop) => {
            Some(Heraldry::new_dots(a,b,(prop*100.) as u32,*c,false))
        },
        _ => None
    }
}

fn make_heraldry(patina: &Patina) -> Result<EachOrEvery<Heraldry>,Error> {
    let (colours,hollow) = match patina {
        Patina::Drawn(DrawnType::Fill,c) => (c,false),
        Patina::Drawn(DrawnType::Stroke(_),c) => (c,true),
        _ => Err(Error::fatal("heraldry attempted on non filled/hollow"))?
    };
    colours.map_results(|colour| {
        colour_to_heraldry(colour,hollow)
            .ok_or_else(|| Error::fatal("heraldry attempted on non-heraldic colour"))
    })
}

pub struct GLCategoriser();

impl ShapeDemerge for GLCategoriser {
    type X = DrawGroup;

    fn categorise(&self, coord_system: &CoordinateSystem, depth: i8) -> Self::X {
        DrawGroup::new(coord_system,depth,&ShapeCategory::Other)
    }

    fn categorise_with_colour(&self, coord_system: &CoordinateSystem, depth: i8, drawn_variety: &DrawnType, colour: &Colour) -> Self::X {
        let is_fill = match drawn_variety {
            DrawnType::Fill => false,
            DrawnType::Stroke(_) => true
        };
        let category = if let Some(heraldry) = colour_to_heraldry(colour,is_fill) {
            ShapeCategory::Heraldry(heraldry.canvases_used(),heraldry.scale())                                
        } else {
            ShapeCategory::SolidColour
        };
        DrawGroup::new(&coord_system,depth,&category)        
    }
}

pub(crate) fn prepare_shape_in_layer(tools: &mut DrawingToolsBuilder, shape: DrawingShape) -> Result<Vec<GLShape>,Error> {
    let mut out = vec![];
    let demerge = shape.demerge(&GLCategoriser());
    for (draw_group,shape) in demerge {
        if draw_group.coord_system().is_dustbin() { continue; }
        match shape {
            Shape::Empty(_) => {},
            Shape::Wiggle(shape) => {
                out.push(GLShape::Wiggle(shape.range(),shape.values(),shape.plotter().clone(),shape.get_style().depth));
            },
            Shape::Text(shape) => {
                prepare_text(&mut out,tools,&shape,&draw_group);
            },
            Shape::Image(shape) => {
                let depth = shape.position().allotments().map(|x| x.depth);
                let drawing_bitmap = tools.bitmap();
                let names = shape.iter_names().collect::<Vec<_>>();
                let mut all_bitmaps = names.iter().map(|asset| drawing_bitmap.make(&shape.channel(),asset)).collect::<Result<Vec<_>,_>>()?;
                let manager = tools.composition_builder(&CanvasType::Crisp);
                let handles = all_bitmaps.drain(..).map(|x| manager.add(x)).collect::<Result<_,_>>()?;
                out.push(GLShape::Image(shape.position().clone(),handles,depth,draw_group));
            },
            Shape::SpaceBaseRect(shape) => {
                out.append(&mut split_spacebaserect(tools,&shape,&draw_group)?);
            },
            Shape::Polygon(shape) => {
                out.append(&mut split_polygon(&shape,&draw_group)?);
            }
        }
    }
    Ok(out)
}
