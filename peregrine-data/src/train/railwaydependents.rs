use std::sync::Mutex;

use peregrine_toolkit::{lock, plumbing::onchange::MutexOnChange, sync::{blocker::{Blocker, Lockout}, needed::Needed}};

use crate::{CarriageExtent, ShapeStore, PeregrineCoreBase, allotment::{core::{allotmentmetadata::AllotmentMetadataReport, trainstate::TrainState}}, PlayingField, DrawingCarriage};

use super::{anticipate::Anticipate, railwayevent::RailwayEvents, train::Train};

pub struct RailwayDependents {
    anticipate: Anticipate,
    playing_field: MutexOnChange<PlayingField>,
    metadata: MutexOnChange<AllotmentMetadataReport>,
    visual_blocker: Blocker,
    #[allow(unused)]
    visual_lockout: Mutex<Option<Lockout>>
}

impl RailwayDependents {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker) -> RailwayDependents {
        RailwayDependents {
            anticipate: Anticipate::new(base,result_store),
            playing_field: MutexOnChange::new(),
            metadata: MutexOnChange::new(),
            visual_blocker: visual_blocker.clone(),
            visual_lockout: Mutex::new(None),
        }
    }

    fn draw_update_playingfield(&self, carriages: &[DrawingCarriage], events: &mut RailwayEvents) {
        let mut playing_field = PlayingField::empty();
        for carriage in carriages {
            playing_field = playing_field.union(&carriage.solution().playing_field());
        }
        self.playing_field.update(playing_field, |playing_field| {
            events.draw_notify_playingfield(playing_field.clone());
        });
    }

    fn draw_update_allotment_metadata(&self, quiescent: Option<&Train>, events: &mut RailwayEvents) {
        if let Some(train) = quiescent {
            if train.is_active() {
                if let Some(metadata) = train.allotter_metadata() {
                    self.metadata.update(metadata,|metadata| {
                        events.draw_send_allotment_metadata(&metadata);
                    });
                }
            }
        }
    }

    fn update_visual_lock(&self, busy: bool) {
        let mut lockout = lock!(self.visual_lockout);
        if busy {
            if lockout.is_none() {
                *lockout = Some(self.visual_blocker.lock());
            }
        } else {
            *lockout = None;
        }
    }

    pub(super) fn position_was_updated(&self, carriage_extent: &CarriageExtent) {
        self.anticipate.anticipate(carriage_extent);
    }

    pub(super) fn carriages_loaded(&self, quiescent: Option<&Train>, carriages: &[DrawingCarriage], events: &mut RailwayEvents) {
        self.draw_update_allotment_metadata(quiescent,events);
        self.draw_update_playingfield(carriages,events);
    }

    pub(super) fn busy(&self, busy: bool) {
        self.update_visual_lock(busy);
    }
}
