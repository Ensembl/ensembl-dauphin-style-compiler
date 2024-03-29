use commander::{ CommanderStream, cdr_tick };
use eard_interp::{LibcoreTemplate, InterpreterBuilder, build_libcore, Interpreter, LibcoreBuilder, RunContext, prepare_libcore};
use eard_libeoe::{ build_libeoe, LibEoEBuilder, prepare_libeoe };
use peregrine_data::{ 
    PgCommander, PgCommanderTaskSpec, PeregrineCore, add_task, DataStore, SmallValuesStore
};
use peregrine_dauphin_queue::{ PgDauphinTaskSpec, PgEardoLoadTaskSpec, PgEardoRunTaskSpec };
use eard_libperegrine::{build_libperegrine, prepare_libperegrine, LibPeregrineBuilder};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::time::now;
use peregrine_toolkit::{log_extra, log, lock};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

fn load_eardo(interp: &mut Interpreter, spec: PgEardoLoadTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    stream.add(interp.load(&spec.data).map_err(|e|
        Error::operr(&format!("Cannot load eardo: {}",e))
    ));
}

macro_rules! result {
    ($value:expr, $stream:expr, $out:expr) => {
        match $value {
            Ok(x) => x,
            Err(e) => {
                $stream.add(Err(Error::operr(&format!("running eardo: {}",e))));
                return $out;
            }
        }
    };
}

fn run_eardo(interp: &mut Interpreter, data_store: &DataStore, small_values_store: &SmallValuesStore, 
    libcore_builder: &LibcoreBuilder, libperegrine_builder: &LibPeregrineBuilder, libeoe_builder: &LibEoEBuilder,
    commander: &PgCommander, spec: PgEardoRunTaskSpec, stream: CommanderStream<Result<(),Error>>) {
    /* run */
    let stream = stream.clone();
    let program = result!(interp.get(&spec.name,"main"),stream,()).clone();
    let libcore_builder = libcore_builder.clone();
    let libperegrine_builder = libperegrine_builder.clone();
    let libeoe_builder = libeoe_builder.clone();
    let data_store = data_store.clone();
    let small_values_store = small_values_store.clone();
    let task = PgCommanderTaskSpec {
        name: format!("eard: {:?}",spec.name),
        prio: spec.prio,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let mut context = RunContext::new();
            prepare_libcore(&mut context,&libcore_builder,LibcoreBrowser::new());
            result!(prepare_libperegrine(
                &mut context,&libperegrine_builder,&data_store,&small_values_store,
                spec.payloads
            ),stream,Ok(()));
            result!(prepare_libeoe(&mut context,&libeoe_builder),stream,Ok(()));
            result!(program.run(context).await,stream,Ok(()));
            stream.add(Ok(()));
            Ok(())
        }),
        stats: true
    };
    add_task(&commander,task);
}

async fn call_up_async(last_bounce: Arc<Mutex<Option<f64>>>) -> Result<(),String> {
    let now = now().round();
    let prev = *lock!(last_bounce);
    if let Some(prev) = prev {
        if prev != now {
            cdr_tick(0).await;
        }
    }
    *lock!(last_bounce) = Some(now);
    Ok(())
}

struct LibcoreBrowser {
    last_bounce: Arc<Mutex<Option<f64>>>
}

impl LibcoreBrowser {
    fn new() -> LibcoreBrowser {
        LibcoreBrowser {
            last_bounce: Arc::new(Mutex::new(None))
        }
    }
}

impl LibcoreTemplate for LibcoreBrowser {
    fn print(&self, s: &str) {
        log!("{}",s);
    }

    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>> {
        let last_bounce = self.last_bounce.clone();
        Box::pin(call_up_async(last_bounce))
    }    
}

fn eard_interp() -> Result<(Interpreter,LibcoreBuilder,LibPeregrineBuilder,LibEoEBuilder),String> {
    let mut builder = InterpreterBuilder::new();
    let libcore_builder = build_libcore(&mut builder)?;
    let libperegrine_builder = build_libperegrine(&mut builder)?;
    let libeoe_builder = build_libeoe(&mut builder)?;
    Ok((Interpreter::new(builder),libcore_builder,libperegrine_builder,libeoe_builder))
}

async fn main_loop(core: PeregrineCore) -> Result<(),Error> {
    let data_store = core.agent_store.data_store.clone();
    let small_values_store = core.agent_store.small_values_store.clone();
    let (mut interp,libcore_builder,libperegrine_builder,libeoe_builder) = eard_interp().map_err(|e| Error::operr(&e))?;
    loop {
        let e = core.base.dauphin_queue.get().await;
        match e.task {
            PgDauphinTaskSpec::LoadEardo(p) => load_eardo(&mut interp,p,e.channel),
            PgDauphinTaskSpec::RunEardo(r) => run_eardo(&mut interp,&data_store,&small_values_store,&libcore_builder,&libperegrine_builder,&libeoe_builder,&core.base.commander,r,e.channel),
            PgDauphinTaskSpec::Quit => { break; }
        }
    }
    log_extra!("dauphin runner quit");
    Ok(())
}

pub fn peregrine_dauphin(core: &PeregrineCore) {
    add_task(&core.base.commander,PgCommanderTaskSpec {
        name: "dauphin runner".to_string(),
        prio: 2,
        slot: None,
        timeout: None,
        task: Box::pin(main_loop(core.clone())),
        stats: false
    });
}
