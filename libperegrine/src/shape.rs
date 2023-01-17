use std::sync::{Arc, Mutex};
use eachorevery::EachOrEvery;
use eard_interp::{GlobalContext, GlobalBuildContext, Return, HandleStore, AsyncReturn };
use peregrine_data::{ProgramShapesBuilder, SpaceBaseArea, PartialSpaceBase, SpaceBase, LeafRequest, Patina, Plotter, Pen, AccessorResolver, BackendNamespace};
use peregrine_toolkit::{lock};
use crate::util::eoe_from_handle;

fn eoe_from_string_reg(ctx: &GlobalContext, reg: usize) -> Result<EachOrEvery<String>,String> {
    Ok(if !ctx.is_finite(reg)? {
        EachOrEvery::every(ctx.force_infinite_string(reg)?.to_string())
    } else if ctx.is_atomic(reg)? {
        EachOrEvery::every(ctx.force_string(reg)?.to_string())
    } else {
        EachOrEvery::each(
            ctx.force_finite_string(reg)?.iter().map(|h| {
                h.to_string()
            }).collect::<Vec<_>>()
        )
    })
}

pub(crate) fn op_rectangle(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let paints = ctx.context.get(&paints);
        let nw = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1] as usize)? as usize)?.clone();
        let area = SpaceBaseArea::new(
            PartialSpaceBase::from_spacebase(nw),
            PartialSpaceBase::from_spacebase(se)).ok_or_else(|| {
                format!("coordinates differ in size when drawing rectangle")
            })?;
        let leafs = eoe_from_handle(ctx,leafs,regs[3])?.index(|a| a.name().clone());
        let area = area.replace_allotments(leafs);
        let paint = paints.get(ctx.force_number(regs[2])? as usize)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_rectangle(area,paint,None).map_err(|e| {
            format!("cannot add rectangle: {}",e.to_string())
        })?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_empty(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let nw = coords.get(ctx.force_number(regs[0] as usize)? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1] as usize)? as usize)?.clone();
        let area = SpaceBaseArea::new(
            PartialSpaceBase::from_spacebase(nw),
            PartialSpaceBase::from_spacebase(se)).ok_or_else(|| {
                format!("coordinates differ in size when drawing rectangle")
            })?;
        let leafs = eoe_from_handle(ctx,leafs,regs[2])?.index(|a| a.name().clone());
        let area = area.replace_allotments(leafs);
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_empty(area).map_err(|e| {
            format!("cannot add empty: {}",e.to_string())
        })?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_wiggle(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let graph_types = gctx.patterns.lookup::<HandleStore<Plotter>>("graph-types")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let bp_left = ctx.force_number(regs[0])?;
        let bp_right = ctx.force_number(regs[1])?;
        let graph_type = ctx.force_number(regs[2])? as usize;
        let values = ctx.force_finite_number(regs[3])?;
        let full_values = if ctx.is_finite(regs[4])? {
            let present = ctx.force_finite_boolean(regs[4])?;
            values.iter().zip(present.iter().cycle()).map(|(v,p)| {
                if *p { Some(*v) } else { None }
            }).collect()
        } else {
            let present = ctx.force_infinite_boolean(regs[4])?;
            if present {
                values.iter().map(|x| Some(*x)).collect()
            } else {
                vec![None;values.len()]
            }
        };
        let leafs = ctx.context.get(&leafs);
        let leaf = ctx.force_number(regs[5])? as usize;
        let leaf = leafs.get(leaf)?.clone();
        let graph_types = ctx.context.get(&graph_types);
        let graph_type = graph_types.get(graph_type)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_wiggle(bp_left,bp_right,graph_type,full_values,leaf).map_err(|e|
            format!("Cannot add wiggle: {}",e)
        )?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_text(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let pens = ctx.context.get(&pens);
        let coords = coords.get(ctx.force_number(regs[0])? as usize)?.clone();
        let pen = pens.get(ctx.force_number(regs[1])? as usize)?.clone();
        let text = eoe_from_string_reg(ctx,regs[2])?;
        let leafs = eoe_from_handle(ctx,leafs,regs[3])?;
        let coords = coords.replace_allotments(leafs);
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_text(coords,pen,text).map_err(|e| {
            format!("cannot add text: {}",e.to_string())
        })?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_running_text(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let coords = ctx.context.get(&coords);
        let leafs = ctx.context.get(&leafs);
        let pens = ctx.context.get(&pens);
        let nw = coords.get(ctx.force_number(regs[0])? as usize)?.clone();
        let se = coords.get(ctx.force_number(regs[1])? as usize)?.clone();
        let pen = pens.get(ctx.force_number(regs[2])? as usize)?.clone();
        let text = eoe_from_string_reg(ctx,regs[3])?;
        let leafs = eoe_from_handle(ctx,leafs,regs[4])?;
        let area = SpaceBaseArea::new(
            PartialSpaceBase::from_spacebase(nw),
            PartialSpaceBase::from_spacebase(se)).ok_or_else(|| format!("lengths don't match in running text"))?;
        let area = area.replace_allotments(leafs);
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_running_text(area,pen,text).map_err(|e| {
            format!("cannot add text: {}",e.to_string())
        })?;
        Ok(Return::Sync)
    }))
}

async fn image_channel(resolver: AccessorResolver) -> Result<BackendNamespace,String> {
    resolver.resolve("source://").await.map_err(|e| e.message.to_string())
}

pub(crate) fn op_image(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let resolver = gctx.patterns.lookup::<AccessorResolver>("channel-resolver")?;
    Ok(Box::new(move |ctx,regs| {
        let resolver = ctx.context.get(&resolver);
        let coords = ctx.context.get(&coords);
        let coords = coords.get(ctx.force_number(regs[0])? as usize)?.clone();
        let leafs = ctx.context.get(&leafs);
        let leafs = eoe_from_handle(ctx,leafs,regs[2])?;
        let shapes = ctx.context.get(&shapes).clone();
        Ok(Return::Async(AsyncReturn::new(
            Box::pin(image_channel(resolver.clone())),
            move |ctx,regs,channel| {
                let image = eoe_from_string_reg(ctx,regs[1])?;
                let coords = coords.replace_allotments(leafs.clone());
                let mut shapes = lock!(shapes);
                shapes.as_mut().unwrap().add_image(&channel,coords,image).map_err(|e| {
                    format!("cannot add text: {}",e.to_string())
                })?;        
                Ok(())
            }
        )))
    }))
}
