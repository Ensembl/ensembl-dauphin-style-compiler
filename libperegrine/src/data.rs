use std::sync::{Arc, Mutex};
use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Value, Return, AsyncReturn };
use peregrine_data::{DataRequest, PacketPriority, DataStore, DataResponse, LoadMode, RunReport, ShapeRequest, AccessorResolver, BackendNamespace };

async fn resolve(resolver: AccessorResolver, accessor: String) -> Result<BackendNamespace,String> {
    resolver.resolve(&accessor).await.map_err(|e| e.message.to_string())
}

pub(crate) fn op_request(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    let requests = gctx.patterns.lookup::<HandleStore<DataRequest>>("requests")?;
    let resolver = gctx.patterns.lookup::<AccessorResolver>("channel-resolver")?;
    Ok(Box::new(move |ctx,regs| {
        let shape_request = ctx.context.get(&shape_request);
        let region = shape_request.region().clone();
        let backend = ctx.force_string(regs[1])?.to_string();
        let endpoint = ctx.force_string(regs[2])?.to_string();
        let resolver = ctx.context.get(&resolver);
        let requests = requests.clone();
        Ok(Return::Async(AsyncReturn::new(
            Box::pin(resolve(resolver.clone(),backend)),
            move |ctx,regs,backend| {
                let req = DataRequest::new(&backend,&endpoint,&region);
                let requests = ctx.context.get_mut(&requests);
                let h = requests.push(req);
                ctx.set(regs[0],Value::Number(h as f64))?;
                Ok(())
            }
        )))
    }))
}

pub(crate) fn op_scope(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<DataRequest>>("requests")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[0])? as usize;
        let k = ctx.force_string(regs[1])?.to_string();
        let v = ctx.force_string(regs[2])?.to_string();
        let requests = ctx.context.get_mut(&requests);
        let req = requests.get_mut(h)?;
        req.add_scope(&k,&vec![v]);
        Ok(Return::Sync)
    }))
}

async fn get_data(data_store: DataStore, request: DataRequest, priority: PacketPriority) -> Result<(DataResponse,f64),String> {
    data_store.get(&request,&priority).await.map_err(|e| e.message.to_string())
}

pub(crate) fn op_get_data(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<DataRequest>>("requests")?;
    let responses = gctx.patterns.lookup::<HandleStore<DataResponse>>("responses")?;
    let report = gctx.patterns.lookup::<Arc<Mutex<RunReport>>>("report")?;
    let data_store = gctx.patterns.lookup::<DataStore>("store")?;
    let mode = gctx.patterns.lookup::<LoadMode>("mode")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let requests = ctx.context.get(&requests);
        let req = requests.get(h)?.clone();
        let mode = ctx.context.get(&mode);
        let priority = if mode.high_priority() { PacketPriority::RealTime } else { PacketPriority::Batch };
        let responses = responses.clone();
        let report = report.clone();
        let data_store = ctx.context.get(&data_store).clone();
        Ok(Return::Async(AsyncReturn::new(
            Box::pin(get_data(data_store,req,priority)),
            move |ctx,regs,(res,took_ms)| {
                let responses = ctx.context.get_mut(&responses);
                let h = responses.push(res);
                ctx.set(regs[0],Value::Number(h as f64))?;
                let report = ctx.context.get(&report);
                report.lock().unwrap().net_ms += took_ms;
                Ok(())
            }
        )))
    }))
}

pub(crate) fn op_data_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<DataResponse>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = res.get2(stream).map_err(|e|
            format!("cannot get data stream {}: {}",stream,e)
        )?;
        let value = value.data_as_booleans().map_err(|e|
            format!("data stream {} was not booleans",stream)
        )?;
        ctx.set(regs[0],Value::FiniteBoolean(value.as_ref().to_vec()))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<DataResponse>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = res.get2(stream).map_err(|e|
            format!("cannot get data stream {}: {}",stream,e)
        )?;
        let value = value.data_as_numbers().map_err(|e|
            format!("data stream {} was not numbers",stream)
        )?;
        ctx.set(regs[0],Value::FiniteNumber(value.as_ref().to_vec()))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<DataResponse>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = res.get2(stream).map_err(|e|
            format!("cannot get data stream {}: {}",stream,e)
        )?;
        let value = value.data_as_strings().map_err(|e|
            format!("data stream {} was not strings",stream)
        )?;
        ctx.set(regs[0],Value::FiniteString(value.as_ref().to_vec()))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_bp_range(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let shape_request = ctx.context.get(&shape_request);
        let min = shape_request.region().min_value();
        let max = shape_request.region().max_value();
        ctx.set(regs[0],Value::Number(min as f64))?;
        ctx.set(regs[1],Value::Number(max as f64))?;
        Ok(Return::Sync)
    }))
}
