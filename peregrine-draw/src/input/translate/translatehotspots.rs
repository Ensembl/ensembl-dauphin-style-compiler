use std::{collections::{HashSet, BTreeSet}, sync::{Arc, Mutex}, iter::FromIterator};
use commander::CommanderStream;
use eachorevery::eoestruct::StructValue;
use peregrine_data::{HotspotResult, DataMessage, SingleHotspotEntry};
use peregrine_toolkit::{lock, error::Error};
use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI};

fn process_hotspot_event(api: &LockedPeregrineInnerAPI, engaged: &mut BTreeSet<SingleHotspotEntry>, x: f64, doc_y: f64, only_hover: bool) -> Result<(),Message> {
    let events = api.trainset.get_hotspot(&lock!(api.stage).read_stage(), (x,doc_y))?;
    let events = filter_events_by_depth(events);
    if only_hover {
        /* If this is only a hover event, then the only events we ware about are those which
         * have just been added to or removed from the engaged set.
         */
        let events = BTreeSet::from_iter(events.iter().cloned());
        let new_events = events.difference(engaged).cloned().collect::<Vec<_>>();
        let old_events = engaged.difference(&events).cloned().collect::<Vec<_>>();
        *engaged = events;
        process_each_hotspot_event(api,&new_events,x,doc_y,true)?;
        process_each_hotspot_event(api,&old_events,x,doc_y,false)?;
    } else {
        /* click event */
        process_each_hotspot_event(api,&events,x,doc_y,true)?;
    }
    Ok(())
}

fn event_depth(event: &SingleHotspotEntry) -> Option<i8> {
    event.value().map(|event| {
        match event {
            HotspotResult::Setting(_,_,_,d) => d,
            HotspotResult::Special(_,d) => d,
            HotspotResult::Click(_,_,d) => d
        }
    })
}

fn filter_events_by_depth(mut events: Vec<SingleHotspotEntry>) -> Vec<SingleHotspotEntry> {
    let depths = events.iter().map(|d| event_depth(d)).collect::<Vec<_>>();
    let max = depths.iter().filter_map(|x| *x).max().unwrap_or(0);
    let events = events.drain(..).zip(depths).filter_map(|(e,d)|
        if d.map(|d| d == max).unwrap_or(false) {
            Some(e)
        } else {
            None
        }
    ).collect();
    events
} 

fn process_each_hotspot_event(api: &LockedPeregrineInnerAPI, events: &[SingleHotspotEntry], x: f64, doc_y: f64, start: bool) -> Result<(),Message> {
    let mut hotspot_contents = HashSet::new();
    let mut hotspot_varieties = HashSet::new();
    for event in events {
        if let Some(event) = event.value() {
            match event {
                HotspotResult::Click(variety,_,_) => {
                    hotspot_varieties.insert(variety);
                },
                _ => {}
            }
        }
    }
    for event in events {
        if let Some(event) = event.value() {
            match event {
                HotspotResult::Special(_,_) => {},
                HotspotResult::Click(_,contents,_) => {
                    hotspot_contents.insert(contents);
                },
                HotspotResult::Setting(switch,path,value,_) => {
                    let switch = switch.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                    api.data_api.update_setting(&switch,&path,StructValue::new_expand(&value,None).map_err(|e| {
                        Message::DataError(DataMessage::XXXTransitional(Error::operr(&e)))
                    })?);
                }
            }
        }
    }
    if hotspot_contents.len() > 0 || hotspot_varieties.len() > 0 {
        api.report.hotspot_event(x,doc_y,start,&hotspot_varieties.iter().cloned().collect::<Vec<_>>(),&hotspot_contents.drain().collect::<Vec<_>>());
    }
    Ok(())
}

fn process_event(messages: &CommanderStream<Option<(f64,f64,bool)>>, event: &InputEvent, window_y: f64) -> Result<(),Message> {
    match event.details {
        InputEventKind::ZMenu => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y+window_y,false)));
        },
        InputEventKind::HoverChange => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y+window_y,true)));
        },
        _ => {}
    }
    Ok(())
}

async fn hotspots_loop(inner_api: &mut PeregrineInnerAPI, messages: CommanderStream<Option<(f64,f64,bool)>>) -> Result<(),Message> {
    let messages2 = messages.clone();
    inner_api.lock().await.dom.shutdown().add(move || {
        messages2.add(None);
    });
    let engaged = Arc::new(Mutex::new(BTreeSet::new()));
    while let Some(message) = messages.get().await {
        let api = inner_api.lock().await;
        let mut engaged = lock!(engaged);
        process_hotspot_event(&api,&mut engaged,message.0,message.1,message.2)?;
        drop(engaged);
    }
    Ok(())
}

pub(crate) fn translate_hotspots(low_level: &mut LowLevelInput, commander: &PgCommanderWeb, inner_api: &PeregrineInnerAPI) {
    let api2 = inner_api.clone();
    let messages = CommanderStream::new();
    let messages2 = messages.clone();
    low_level.distributor_mut().add(move |e| {
        let window_pos = lock!(api2.stage()).read_stage().y().position().unwrap_or(0.);
        process_event(&messages,e,window_pos).ok();
    }); // XXX error distribution
    let mut inner2 = inner_api.clone();
    commander.add("hotspot", 0, None, None, Box::pin(async move { 
        hotspots_loop(&mut inner2,messages2).await
    }));
}
