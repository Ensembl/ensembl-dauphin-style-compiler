use peregrine_core::{ Shape, SingleAnchor, SeaEnd, Patina, Colour };
use super::super::layers::layer::Layer;
use super::paintskin::PaintSkin;
use super::paintgeometry::PaintGeometry;
use crate::webgl::AccumulatorCampaign;

pub(crate) struct GLShape(Shape);

impl GLShape {
    pub fn new(shape: Shape) -> GLShape {
        GLShape(shape)
    }
}

fn colour_to_paintskin(colour: &Colour) -> PaintSkin {
    match colour {
        Colour::Direct(_) => PaintSkin::Colour,
        _ => PaintSkin::Colour // XXX
    }
}

fn add_rectangle<'a>(layer: &'a mut Layer, anchor: SingleAnchor, skin: &PaintSkin, allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
    match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok(layer.get_pin(skin)?.add_solid_rectangles(layer,skin,xx,yy,ship_x,ship_y,x_size,y_size)?)
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok(layer.get_fix(skin)?.add_solid_rectangles(layer,skin,sea_x,sea_y,ship_x,ship_y,x_size,y_size)?)          
        },
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok(layer.get_tape(skin)?.add_solid_rectangles(layer,skin,xx,sea_y,ship_x,ship_y,x_size,y_size)?)          
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok(layer.get_page(skin)?.add_solid_rectangles(layer,skin,sea_x,yy,ship_x,ship_y,x_size,y_size)?)
        }
    }
}

fn add_colour(campaign: &mut AccumulatorCampaign, layer: &mut Layer, geometry: &PaintGeometry, colour: &Colour, vertexes: usize) -> anyhow::Result<()> {
    match colour {
        Colour::Direct(d) => {
            let direct = layer.get_direct(geometry)?;
            direct.block_colour(campaign,d,vertexes)?;
        },
        _ => {}
    }
    Ok(())
}

pub fn add_shape_to_layer(layer: &mut Layer, shape: Shape) -> anyhow::Result<()> {
    match shape {
        Shape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            match patina {
                Patina::Filled(colour) => { // XXX patina
                    let paintskin = colour_to_paintskin(&colour);
                    let mut campaign = add_rectangle(layer,anchor,&paintskin,allotment,x_size,y_size)?;
                    add_colour(&mut campaign,layer,&PaintGeometry::Pin,&colour,4)?;
                },
                _ => {}
            }
        },
        _ => {}
    }
    Ok(())
}