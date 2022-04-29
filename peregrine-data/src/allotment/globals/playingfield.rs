use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer, commute}};

use crate::{CoordinateSystemVariety, CoordinateSystem};

use super::globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,Hash,PartialEq,Eq)]
pub enum PlayingFieldEdge { Top, Bottom, Left, Right }

pub struct LocalPlayingFieldBuilder(LocalValueBuilder<PlayingFieldEdge,f64,f64>);

impl LocalPlayingFieldBuilder {
    pub(crate) fn new() -> LocalPlayingFieldBuilder {
        LocalPlayingFieldBuilder(LocalValueBuilder::new())
    }

    pub(crate) fn set(&mut self, coord_system: &CoordinateSystem, value: StaticValue<f64>) {
        let edge = match (&coord_system.0,coord_system.1) {
            (CoordinateSystemVariety::Tracking, false) => PlayingFieldEdge::Top,
            (CoordinateSystemVariety::Tracking, true) => PlayingFieldEdge::Bottom,
            (CoordinateSystemVariety::TrackingWindow, false) => PlayingFieldEdge::Top,
            (CoordinateSystemVariety::TrackingWindow, true) => PlayingFieldEdge::Bottom,
            (CoordinateSystemVariety::Window, false) => PlayingFieldEdge::Top,
            (CoordinateSystemVariety::Window, true) => PlayingFieldEdge::Bottom,
            (CoordinateSystemVariety::Sideways, false) => PlayingFieldEdge::Left,
            (CoordinateSystemVariety::Sideways, true) => PlayingFieldEdge::Right,
            _ => { return; }
        };
        self.0.entry(edge).add_local(value);
    }

    pub(crate) fn global(&mut self, name: &PlayingFieldEdge) -> &StaticValue<f64> {
        self.0.entry(name.clone()).get_global()
    }
}

pub struct LocalPlayingField(LocalValueSpec<PlayingFieldEdge,f64,f64>);

impl LocalPlayingField {
    pub(crate) fn new(builder: &LocalPlayingFieldBuilder) -> LocalPlayingField {
        let out = LocalPlayingField(LocalValueSpec::new(&builder.0,|x| {
            commute(x,0.,|x,y| x.max(*y)).dearc()
        }));
        out
    }

    pub(crate) fn add(&self, global: &mut GlobalPlayingFieldBuilder) {
        global.0.add(&self.0);
    }
}

pub struct GlobalPlayingFieldBuilder(GlobalValueBuilder<PlayingFieldEdge,f64,f64>);

impl GlobalPlayingFieldBuilder {
    pub(crate) fn new() -> GlobalPlayingFieldBuilder {
        GlobalPlayingFieldBuilder(GlobalValueBuilder::new())
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct GlobalPlayingField(GlobalValueSpec<PlayingFieldEdge,f64>);

impl GlobalPlayingField {
    pub(crate) fn new(builder: GlobalPlayingFieldBuilder, answer: &mut StaticAnswer) -> GlobalPlayingField {
        GlobalPlayingField(GlobalValueSpec::new(builder.0,move |_,x,answer| {
            let v = x.iter().map(|x| x.call(&answer)).fold(0.,f64::max);
            (v,(v*100000.).round() as i64)
        },answer))
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct PlayingField {
    pub height: f64,
    pub squeeze: (f64,f64),
}

impl PlayingField {
    pub fn new(global: &GlobalPlayingField) -> PlayingField {
        let top = global.0.get(&PlayingFieldEdge::Top).unwrap_or(&0.).clone();
        let bottom = global.0.get(&PlayingFieldEdge::Bottom).unwrap_or(&0.).clone();
        let left = global.0.get(&PlayingFieldEdge::Left).unwrap_or(&0.).clone();
        let right = global.0.get(&PlayingFieldEdge::Right).unwrap_or(&0.).clone();
        PlayingField {
            height: top+bottom,
            squeeze: (left,right)
        }
    }
}
