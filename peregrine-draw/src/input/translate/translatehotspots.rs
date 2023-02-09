use std::{collections::{HashSet, BTreeSet}, sync::{Arc, Mutex}, iter::FromIterator};
use commander::CommanderStream;
use eachorevery::eoestruct::StructValue;
use peregrine_data::{DataMessage, HotspotResultVariety, SingleHotspotResult};
use peregrine_toolkit::{lock, error::Error, hotspots::hotspotstore::HotspotPosition};
use crate::{Message, PeregrineInnerAPI, PgCommanderWeb, input::{InputEvent, InputEventKind, low::lowlevel::LowLevelInput}, run::inner::LockedPeregrineInnerAPI};

fn process_hotspot_event(api: &LockedPeregrineInnerAPI, engaged: &mut BTreeSet<SingleHotspotResult>, x: f64, doc_y: f64, win_y: f64, only_hover: bool) -> Result<(),Message> {
    let events = api.trainset.get_hotspot(&lock!(api.stage).read_stage(), (x,doc_y))?;
    let events = filter_events(events,only_hover);
    if only_hover {
        /* If this is only a hover event, then the only events we ware about are those which
         * have just been added to or removed from the engaged set.
         */
        let events = BTreeSet::from_iter(events.iter().cloned());
        let new_events = events.difference(engaged).cloned().collect::<Vec<_>>();
        let old_events = engaged.difference(&events).cloned().collect::<Vec<_>>();
        *engaged = events;
        process_each_hotspot_event(api,&new_events,x,doc_y,win_y,true)?;
        process_each_hotspot_event(api,&old_events,x,doc_y,win_y,false)?;
    } else {
        /* click event */
        process_each_hotspot_event(api,&events,x,doc_y,win_y,true)?;
    }
    Ok(())
}

fn event_depth(event: &SingleHotspotResult) -> Option<i8> {
    event.entry.value().map(|r| r.depth)
}

fn filter_events(mut events: Vec<SingleHotspotResult>, only_hover: bool) -> Vec<SingleHotspotResult> {
    let depths = events.iter().map(|d| event_depth(d)).collect::<Vec<_>>();
    let max = depths.iter().filter_map(|x| *x).max().unwrap_or(0);
    let events = events.drain(..).zip(depths).filter_map(|(e,d)|
        if d.map(|d| d == max).unwrap_or(false) {
            Some(e)
        } else {
            None
        }
    ).filter(|e| e.entry.is_hover() || !only_hover)
    .collect();
    events
} 

fn merge_area(a: Option<HotspotPosition>, b: &HotspotPosition) -> HotspotPosition {
    if let Some(a) = a {
        HotspotPosition {
            top: a.top.min(b.top),
            bottom: a.bottom.max(b.bottom),
            left: a.left.min(b.left),
            right: a.right.max(b.right)
        } 
    } else {
        b.clone()
    }
}

fn process_each_hotspot_event(api: &LockedPeregrineInnerAPI, events: &[SingleHotspotResult], x: f64, doc_y: f64, win_y: f64, start: bool) -> Result<(),Message> {
    let mut hotspot_contents = HashSet::new();
    let mut hotspot_varieties = HashSet::new();
    for event in events {
        if let Some(event) = event.entry.value() {
            match event.variety {
                HotspotResultVariety::Click(variety,_) => {
                    hotspot_varieties.insert(variety);
                },
                _ => {}
            }
        }
    }
    let mut area = None;
    for result in events {
        if let Some(event) = result.entry.value() {
            area = Some(merge_area(area,&result.position));
            match event.variety {
                HotspotResultVariety::Special(_) => {},
                HotspotResultVariety::Click(_,contents) => {
                    hotspot_contents.insert(contents);
                },
                HotspotResultVariety::Setting(switch,path,value) => {
                    let switch = switch.iter().map(|x| x.as_str()).collect::<Vec<_>>();
                    api.data_api.update_setting(&switch,&path,StructValue::new_expand(&value,None).map_err(|e| {
                        Message::DataError(DataMessage::XXXTransitional(Error::operr(&e)))
                    })?);
                }
            }
        }
    }
    let mut area = area.unwrap_or_else(|| HotspotPosition { top: 0., bottom: 0., left: 0., right: 0. });
    area.top -= win_y;
    area.bottom -= win_y;
    if hotspot_contents.len() > 0 || hotspot_varieties.len() > 0 {
        api.report.hotspot_event(x,doc_y,area,start,&hotspot_varieties.iter().cloned().collect::<Vec<_>>(),&hotspot_contents.drain().collect::<Vec<_>>());
    }
    Ok(())
}

fn process_event(messages: &CommanderStream<Option<(f64,f64,f64,bool)>>, event: &InputEvent, window_y: f64) -> Result<(),Message> {
    match event.details {
        InputEventKind::ZMenu => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y+window_y,window_y,false)));
        },
        InputEventKind::HoverChange => {
            let x = *event.amount.get(0).unwrap_or(&0.);
            let y = *event.amount.get(1).unwrap_or(&0.);
            messages.add(Some((x,y+window_y,window_y,true)));
        },
        _ => {}
    }
    Ok(())
}

async fn hotspots_loop(inner_api: &mut PeregrineInnerAPI, messages: CommanderStream<Option<(f64,f64,f64,bool)>>) -> Result<(),Message> {
    let messages2 = messages.clone();
    inner_api.lock().await.dom.shutdown().add(move || {
        messages2.add(None);
    });
    let engaged = Arc::new(Mutex::new(BTreeSet::new()));
    while let Some(message) = messages.get().await {
        let api = inner_api.lock().await;
        let mut engaged = lock!(engaged);
        process_hotspot_event(&api,&mut engaged,message.0,message.1,message.2,message.3)?;
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
