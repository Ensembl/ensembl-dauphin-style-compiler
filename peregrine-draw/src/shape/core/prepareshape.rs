use peregrine_data::{Allotment, Colour, DrawnType, EachOrEvery, HollowEdge, Patina, RectangleShape, Shape, ShapeCommon, ShapeDemerge, ShapeDetails, CoordinateSystem, HollowEdge2};
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::shape::core::drawshape::{SimpleShapePatina};
use crate::shape::heraldry::heraldry::{Heraldry, HeraldryCanvasesUsed};
use crate::shape::triangles::drawgroup::{DrawGroup, ShapeCategory};
use crate::util::message::Message;
use super::drawshape::{ GLShape };

fn split_spacebaserect(tools: &mut DrawingTools, common: &ShapeCommon, shape: &RectangleShape<Allotment>, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Message> {
    let mut out = vec![];
    let depth = common.depth().clone();
    match shape.patina() {
        Patina::Drawn(drawn_variety,_) => {
            let width = match drawn_variety {
                DrawnType::Stroke(w) => Some(*w as f64),
                DrawnType::Fill => None
            };
            match draw_group.shape_category() {
                ShapeCategory::SolidColour | ShapeCategory::Other => {
                    out.push(GLShape::SpaceBaseRect(shape.holey_area().clone(),SimpleShapePatina::from_patina(shape.patina())?,depth,draw_group.clone()));
                },
                ShapeCategory::SpotColour(c) => {
                    out.push(GLShape::SpaceBaseRect(shape.holey_area().clone(),SimpleShapePatina::spot_from_patina(c,shape.patina())?,depth,draw_group.clone()));
                },
                ShapeCategory::Heraldry(HeraldryCanvasesUsed::Solid(heraldry_canvas),scale) => {
                    let heraldry_tool = tools.heraldry();
                    let heraldry = make_heraldry(shape.patina())?;
                    let handles = heraldry.map(|x| heraldry_tool.add(x.clone()));
                    out.push(GLShape::Heraldry(shape.holey_area().clone(),handles,depth,draw_group.clone(),heraldry_canvas.clone(),scale.clone(),None));
                },
                ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale) => {
                    let width = width.unwrap_or(0.);
                    let heraldry_tool = tools.heraldry();
                    let heraldry = make_heraldry(shape.patina())?;
                    let handles = heraldry.map(|x| heraldry_tool.add(x.clone()));
                    // XXX too much cloning, at least Arc them
                    let area = shape.holey_area();
                    out.push(GLShape::Heraldry(area.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge2::Left(width))));
                    out.push(GLShape::Heraldry(area.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge2::Right(width))));
                    out.push(GLShape::Heraldry(area.clone(),handles.clone(),depth.clone(),draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge2::Top(width))));
                    out.push(GLShape::Heraldry(area.clone(),handles,depth,draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge2::Bottom(width))));
                }
            }
        },
        Patina::ZMenu(zmenu,values) => {
            out.push(GLShape::SpaceBaseRect(shape.holey_area().clone(),SimpleShapePatina::ZMenu(zmenu.clone(),values.clone()),depth,draw_group.clone()));
        }
    }
    Ok(out)
}

fn colour_to_heraldry(colour: &Colour, hollow: bool) -> Option<Heraldry> {
    match colour {
        Colour::Stripe(a,b,c,_prop) => {
            Some(Heraldry::Stripe(a.clone(),b.clone(),50,*c))
        },
        Colour::Bar(a,b,c,prop) => {
            if hollow {
                Some(Heraldry::new_dots(a,b,(prop*100.) as u32,*c,false))
            } else {
                Some(Heraldry::new_bar(a,b,(prop*100.) as u32,*c,false))
            }
        },
        _ => None
    }
}

fn make_heraldry(patina: &Patina) -> Result<EachOrEvery<Heraldry>,Message> {
    let (colours,hollow) = match patina {
        Patina::Drawn(DrawnType::Fill,c) => (c,false),
        Patina::Drawn(DrawnType::Stroke(_),c) => (c,true),
        _ => Err(Message::CodeInvariantFailed(format!("heraldry attempted on non filled/hollow")))?
    };
    colours.map_results(|colour| {
        colour_to_heraldry(colour,hollow)
            .ok_or_else(|| Message::CodeInvariantFailed(format!("heraldry attempted on non-heraldic colour")))
    })
}

pub struct GLCategoriser();

impl ShapeDemerge for GLCategoriser {
    type X = DrawGroup;

    fn categorise(&self, coord_system: &CoordinateSystem) -> Self::X {
        DrawGroup::new(coord_system,&ShapeCategory::Other)
    }

    fn categorise_with_colour(&self, coord_system: &CoordinateSystem, drawn_variety: &DrawnType, colour: &Colour) -> Self::X {
        let is_fill = match drawn_variety {
            DrawnType::Fill => false,
            DrawnType::Stroke(_) => true
        };
        let category = if let Some(heraldry) = colour_to_heraldry(colour,is_fill) {
            ShapeCategory::Heraldry(heraldry.canvases_used(),heraldry.scale())                                
        } else if let Colour::Spot(spot) = colour {
            ShapeCategory::SpotColour(spot.clone())
        } else {
            ShapeCategory::SolidColour
        };
        DrawGroup::new(&coord_system,&category)        
    }
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape<Allotment>) -> Result<Vec<GLShape>,Message> {
    let mut out = vec![];
    let demerge = shape.demerge(&GLCategoriser());
    for (draw_group,shape) in demerge {
        let common = shape.common();
        let depth = common.depth().clone();
        match shape.details() {
            ShapeDetails::Wiggle(shape) => {
                if let Some(depth) = depth.get(0) {
                    out.push(GLShape::Wiggle(shape.range(),shape.values(),shape.plotter().clone(),*depth));
                }
            },
            ShapeDetails::Text(shape) => {
                let drawing_text = tools.text();
                let colours_iter = shape.pen().colours().iter().cycle();
                let background = shape.pen().background();
                let texts = shape.iter_texts().collect::<Vec<_>>();
                let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| {
                    drawing_text.add_text(&shape.pen(),text,colour,background)
                }).collect();
                out.push(GLShape::Text(shape.holey_position().clone(),handles,depth,draw_group));
            },
            ShapeDetails::Image(shape) => {
                let drawing_bitmap = tools.bitmap();
                let names = shape.iter_names().collect::<Vec<_>>();
                let handles = names.iter().map(|asset| drawing_bitmap.add_bitmap(asset)).collect::<Result<Vec<_>,_>>()?;
                out.push(GLShape::Image(shape.holey_position().clone(),handles,depth,draw_group));
            },
            ShapeDetails::SpaceBaseRect(shape) => {
                out.append(&mut split_spacebaserect(tools,&common,&shape,&draw_group)?);
            }
        }
    }
    Ok(out)
}
