use std::sync::{Arc, Mutex};
use eard_interp::{ GlobalContext, GlobalBuildContext, HandleStore, Return, Value };
use peregrine_data::{ProgramShapesBuilder, LeafRequest};
use peregrine_toolkit::lock;

pub(crate) fn op_leaf_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let spec = ctx.force_finite_string(regs[1])?.to_vec();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        let mut leaf_list = spec.iter().map(|spec| {
            shapes.as_mut().unwrap().use_allotment(&spec).clone()
        }).collect::<Vec<_>>();
        drop(shapes);
        let leafs = ctx.context.get_mut(&leafs);
        let h = leaf_list.drain(..).map(|leaf| leafs.push(leaf) as f64).collect();
        ctx.set(regs[0],Value::FiniteNumber(h))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_leaf(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let leafs = gctx.patterns.lookup::<HandleStore<LeafRequest>>("leaf")?;
    Ok(Box::new(move |ctx,regs| {
        let spec = ctx.force_string(regs[1])?.to_string();
        let shapes = ctx.context.get_mut(&shapes);
        let mut shapes = lock!(shapes);
        let leaf = shapes.as_mut().unwrap().use_allotment(&spec).clone();
        drop(shapes);
        let leafs = ctx.context.get_mut(&leafs);
        let h = leafs.push(leaf);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}
