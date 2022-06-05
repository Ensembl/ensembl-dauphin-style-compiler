use std::sync::{Mutex, Arc};
use peregrine_toolkit::lock;
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit_async::sync::needed::Needed;
use crate::train::drawing::drawingtrainset::DrawingTrainSet;
use crate::train::graphics::Graphics;
use crate::train::model::trainextent::TrainExtent;
use crate::{Stick, CarriageSpeed, Viewport, DataMessage, DrawingCarriage, PeregrineApiQueue};
use crate::api::MessageSender;
use crate::switch::trackconfiglist::TrainTrackConfigList;

#[cfg(debug_trains)]
use peregrine_toolkit::debug_log;

use super::abstracttrain::AbstractTrain;

pub(crate) enum StickData {
    Pending,
    Ready(Arc<Stick>),
    Unavailable
}

impl StickData {
    fn is_broken(&self) -> bool { match self { StickData::Unavailable => true, _ => false } }
    fn is_ready(&self) -> bool { match self { StickData::Ready(_) => true, _ => false } }
}

pub(crate) struct Train {
    stick_data: Arc<Mutex<StickData>>,
    train_extent: TrainExtent,
    drawing_train_set: DrawingTrainSet,
    abstract_train: AbstractTrain,
    epoch: u64
}

impl Train {
    pub(crate) fn new(data_api: &PeregrineApiQueue, train_extent: &TrainExtent, answer_allocator: &Arc<Mutex<AnswerAllocator>>, configs: &TrainTrackConfigList, graphics: &Graphics, messages: &MessageSender, epoch: u64) -> Train {
        let abstract_train = AbstractTrain::new(data_api,train_extent,answer_allocator,configs,graphics,messages);
        let out = Train {
            train_extent: train_extent.clone(),
            stick_data: Arc::new(Mutex::new(StickData::Pending)),
            drawing_train_set: DrawingTrainSet::new(graphics),
            abstract_train, epoch
        };
        data_api.load_stick(&out.train_extent(),&out.stick_data);
        out
    }

    pub(crate) fn epoch(&self) -> u64 { self.epoch }

    pub(crate) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        self.train_extent().speed_limit(&other.train_extent())
    }

    pub(crate) fn train_extent(&self) -> &TrainExtent { &self.train_extent }

    pub(crate) fn train_half_ready(&self) -> bool {
        self.central_drawing_carriage().is_some() && lock!(self.stick_data).is_ready()
    }

    pub(crate) fn train_ready(&self) -> bool {
        self.train_half_ready() && self.all_ready() 
    }

    pub(crate) fn train_broken(&self) -> bool { lock!(self.stick_data).is_broken() }

    pub(crate) fn mute(&mut self, yn: bool) {
        self.abstract_train.mute(yn);
        if yn {
            self.drawing_train_set.set_mute();
        }
    }

    pub(crate) fn activate(&mut self, stick: &Stick) {
        self.abstract_train.active();
        self.drawing_train_set.set_active(stick);
        self.ping();
    }

    pub(crate) fn update_centre(&mut self, centre: u64) {
        self.abstract_train.update_centre(centre);
    }

    pub(crate) fn central_drawing_carriage(&self) -> Option<&DrawingCarriage> {
        self.drawing_train_set.central_carriage()
    }

    pub(crate) fn all_ready(&self) -> bool {
        self.abstract_train.is_ready() && self.drawing_train_set.can_be_made_active()
    }

    pub(crate) fn set_active(&mut self, speed: CarriageSpeed) {
        let stick_data = match &*lock!(self.stick_data) {
            StickData::Ready(stick_data) => stick_data.clone(),
            _ => { panic!("set_active() called on non-ready train") }
        };
        self.mute(false);
        self.activate(&stick_data);
    }

    pub(crate) fn set_position(&mut self, viewport: &Viewport) -> Result<(),DataMessage> {
        let centre_carriage_index = self.train_extent().scale().carriage(viewport.position()?);
        self.update_centre(centre_carriage_index);
        Ok(())
    }

    /* We need to do a lot in a ping but also make sure we don't customarily do too much!
     *
     * We need to update the CarriageBuilder party to see if any more processes are
     * ready. If they are then partystate will have been updated. In this case we need to
     * transfer copied of these into the DrawingCarriageSwitcher.
     * 
     * We should always ping DrawingCarriageSwitcher anyway to update anything that's
     * changed there.
     * 
     * So we need to make sure ping is scheduled:
     * 1. After anything which could cause a readiness change in CarriageBuilder. This is
     *    ensured by init and dtor in CarriageBuilder and the set at the end of the
     *    CarriageBuilder async method itself.
     * 2. After anything which could cause a readiness change in DrawingCarriageSwitcher.
     *    This would be caused by anything which could cause a readiness change in any of
     *    the underlying DrawingTrains. This is triggered by the set in set_ready
     *    and dtor. Switcher pings respond to changes in ready() in any of their objects.
     *    In this case, this is caused by calls to quiet(). Which gets called by ping in
     *    party.
     */
    pub(crate) fn ping(&mut self) {
        if let Some((state,wanted)) = self.abstract_train.ping() {
            self.drawing_train_set.set(&state,&wanted);
        }
        self.drawing_train_set.ping();
    }
}

// TODO add logs
// TODO test
// TODO check for excess
// TODO lifecycle enum
