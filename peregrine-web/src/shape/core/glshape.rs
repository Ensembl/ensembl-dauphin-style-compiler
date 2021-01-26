use peregrine_core::{ Shape, SingleAnchor, SeaEnd, Patina, Colour };
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::geometry::GeometryProcessName;
use crate::webgl::AccumulatorCampaign;

pub(crate) struct GLShape(Shape);

impl GLShape {
    pub fn new(shape: Shape) -> GLShape {
        GLShape(shape)
    }
}

fn colour_to_patina(colour: Colour) -> PatinaProcessName {
    match colour {
        Colour::Direct(_) => PatinaProcessName::Direct,
        Colour::Spot(c) => PatinaProcessName::Spot(c)
    }
}

fn add_rectangle<'a>(layer: &'a mut Layer, anchor: SingleAnchor, skin: &PatinaProcessName, allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) -> anyhow::Result<(AccumulatorCampaign,GeometryProcessName)> {
    match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok((layer.get_pin(skin)?.add_solid_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size)?,GeometryProcessName::Pin))
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok((layer.get_fix(skin)?.add_solid_rectangles(layer,sea_x,sea_y,ship_x,ship_y,x_size,y_size)?,GeometryProcessName::Fix))     
        },
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok((layer.get_tape(skin)?.add_solid_rectangles(layer,xx,sea_y,ship_x,ship_y,x_size,y_size)?,GeometryProcessName::Tape))         
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok((layer.get_page(skin)?.add_solid_rectangles(layer,sea_x,yy,ship_x,ship_y,x_size,y_size)?,GeometryProcessName::Page))
        }
    }
}

fn add_colour(campaign: &mut AccumulatorCampaign, layer: &mut Layer, geometry: &GeometryProcessName, colour: &Colour, vertexes: usize) -> anyhow::Result<()> {
    match colour {
        Colour::Direct(d) => {
            let direct = layer.get_direct(geometry)?;
            direct.direct(campaign,d,vertexes)?;
        },
        Colour::Spot(colour) => {
            let spot = layer.get_spot(geometry,colour)?;
            let mut process = layer.get_process_mut(geometry,&PatinaProcessName::Spot(colour.clone()))?;
            spot.spot(&mut process)?;
        },
        _ => {}
    }
    Ok(())
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, shape: Shape) -> anyhow::Result<()> {
    match shape {
        Shape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            match patina {
                Patina::Filled(colour) => {
                    let patina = colour_to_patina(colour.clone());
                    let (mut campaign,geometry) = add_rectangle(layer,anchor,&patina,allotment,x_size,y_size)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                },
                _ => {}
            }
        },
        _ => {}
    }
    Ok(())
}
