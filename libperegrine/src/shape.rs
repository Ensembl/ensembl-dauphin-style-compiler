use std::sync::{Arc, Mutex};
use eard_interp::{GlobalContext, GlobalBuildContext, Return, HandleStore};
use peregrine_data::{ProgramShapesBuilder, SpaceBaseArea, PartialSpaceBase, SpaceBase, LeafRequest, Patina};
use peregrine_toolkit::{lock, eachorevery::EachOrEvery};

fn leaf_from_handle(ctx: &GlobalContext, leafs: &HandleStore<LeafRequest>, reg: usize) -> Result<EachOrEvery<LeafRequest>,String> {
    Ok(if !ctx.is_finite(reg)? {
        EachOrEvery::every(leafs.get(ctx.force_infinite_number(reg)? as usize)?.clone())
    } else if ctx.is_atomic(reg)? {
        EachOrEvery::every(leafs.get(ctx.force_number(reg)? as usize)?.clone())
    } else {
        EachOrEvery::each(
            ctx.force_finite_number(reg)?.iter().map(|h| {
                leafs.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?
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
        let leafs = leaf_from_handle(ctx,leafs,regs[3])?.index(|a| a.name().clone());;
        let area = area.replace_allotments(leafs);
        let paint = paints.get(ctx.force_number(regs[2])? as usize)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        shapes.as_mut().unwrap().add_rectangle(area,paint,None).map_err(|e| {
            format!("cannot add rectangle")
        })?;
        Ok(Return::Sync)
    }))
}
