use commander::CommanderStream;
use peregrine_toolkit::{log, eachorevery::eoestruct::StructTemplate};

use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI, shape::layers::drawingzmenus::HotspotEntryDetails};

fn process_hotspot_event(api: &LockedPeregrineInnerAPI, x: f64, y: f64) -> Result<(),Message> {
    let events = api.trainset.get_hotspot(&api.stage.lock().unwrap().read_stage(), (x,y))?;
    let mut zmenus = vec![];
    for event in &events {
        match event {
            HotspotEntryDetails::ZMenu(z) => {
                zmenus.push(z.value());
            },
            HotspotEntryDetails::Switch(value) => {
                let (path,yn) = value.value();
                let path = path.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                log!("switch hotspot {:?}",value.value());
                if yn {
                    let tmpl_true = StructTemplate::new_boolean(true).build().ok().unwrap();
                    api.data_api.switch(&path,tmpl_true);
                } else {
                    let tmpl_false = StructTemplate::new_boolean(false).build().ok().unwrap();
                    api.data_api.switch(&path,tmpl_false);
                }
            }
        }
    }
    if zmenus.len() > 0 {
        api.report.zmenu_event(x,y,zmenus);
    }
    Ok(())
}

fn process_event(messages: &CommanderStream<Option<(f64,f64)>>, event: &InputEvent) -> Result<(),Message> {
    match event.details {
        InputEventKind::ZMenu => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y)));
        },
        _ => {}
    }
    Ok(())
}

async fn hotspots_loop(inner_api: &mut PeregrineInnerAPI, messages: CommanderStream<Option<(f64,f64)>>) -> Result<(),Message> {
    let messages2 = messages.clone();
    inner_api.lock().await.dom.shutdown().add(move || {
        messages2.add(None);
    });
    while let Some(message) = messages.get().await {
        process_hotspot_event(&inner_api.lock().await,message.0,message.1)?;
    }
    Ok(())
}

pub(crate) fn translate_hotspots(low_level: &mut LowLevelInput, commander: &PgCommanderWeb, inner_api: &PeregrineInnerAPI) {
    let messages = CommanderStream::new();
    let messages2 = messages.clone();
    low_level.distributor_mut().add(move |e| { process_event(&messages,e).ok(); }); // XXX error distribution
    let mut inner2 = inner_api.clone();
    commander.add("hotspot", 0, None, None, Box::pin(async move { hotspots_loop(&mut inner2,messages2).await }));
}
