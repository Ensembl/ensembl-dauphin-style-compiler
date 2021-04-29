mod agent {
    pub mod agent;
    pub(crate) mod laneprogramlookup;
    pub(crate) mod lanescaler;
    pub(crate) mod shapeprogramrunagent;

    pub use self::shapeprogramrunagent::{ ShapeProgramRunAgent };
    pub use self::laneprogramlookup::LaneProgramLookup;
    pub use self::lanescaler::LaneScaler;
}

mod api {
    mod api;
    mod agentstore;
    mod pgcore;
    mod queue;

    pub use agentstore::AgentStore;
    pub use api::{ PeregrineIntegration, CarriageSpeed };
    pub use self::pgcore::{ PeregrineCore, MessageSender, PeregrineCoreBase };
    pub use queue::{ ApiMessage, PeregrineApiQueue };
}

mod core {
    mod config;
    pub mod focus;
    mod layout;
    mod scale;
    pub mod stick;
    mod viewport;

    pub use self::config::{ PeregrineConfig, ConfigValue };
    pub use self::focus::Focus;
    pub use self::layout::Layout;
    pub use self::scale::Scale;
    pub use stick::{ StickId, Stick, StickTopology };
    pub use viewport::Viewport;
}

mod index {
    pub(crate) mod stickstore;
    pub(crate) mod stickauthority;
    pub(crate) mod stickauthoritystore;
    pub use self::stickstore::StickStore;
    pub use self::stickauthoritystore::StickAuthorityStore;
}

mod lane {
    mod datastore;
    mod lane;
    pub(crate) mod programregion;
    mod lanestore;
    pub(crate)mod programdata;
    pub(crate) mod programname;

    pub use self::datastore::DataStore;
    pub use self::lane::{ Lane, Region };
    pub use self::programdata::ProgramData;
    pub use self::programname::ProgramName;
    pub use self::programregion::{ ProgramRegion, ProgramRegionBuilder };
    pub use self::lanestore::{ LaneStore };
}

mod request {
    pub(crate) mod backoff;
    pub(crate) mod bootstrap;
    pub(crate) mod channel;
    pub(crate) mod data;
    pub(crate) mod failure;
    pub(crate) mod manager;
    pub(crate) mod packet;
    pub(crate) mod queue;
    pub(crate) mod program;
    pub(crate) mod request;
    pub(crate) mod stick;
    pub(crate) mod stickauthority;
    pub use self::channel::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority };
    pub use self::program::{ ProgramLoader };
    pub use self::manager::RequestManager;
    pub use self::stick::issue_stick_request;
}

mod run {
    pub mod instancepayload;
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec, add_task, complete_task, async_complete_task };
    pub use self::pgdauphin::{ PgDauphin, PgDauphinTaskSpec };
    pub use self::instancepayload::InstancePayload;
}

mod shape {
    mod core;
    mod shape;
    mod shapelist;
    mod zmenu;
    mod zmenufixed;
    mod shapeoutput;

    pub use self::core::{ 
        ScreenEdge, SeaEnd, SeaEndPair, ShipEnd, AnchorPair, SingleAnchor, Patina, Pen, Colour, AnchorPairAxis, DirectColour, SingleAnchorAxis, Plotter 
    };
    pub use self::shape::Shape;
    pub use self::zmenu::ZMenu;
    pub use self::shapelist::ShapeList;
    pub use self::shapeoutput::ShapeOutput;
    pub use self::zmenufixed::{ ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator };
}

pub(crate) mod switch {
    pub(crate) mod track;
    pub(crate) mod switch;
    pub(crate) mod trackconfig;
}

mod train {
    mod carriage;
    mod carriageevent;
    mod carriageset;
    mod train;
    mod trainset;

    pub use carriage::{ CarriageId, Carriage };
    pub use trainset::TrainSet;
}

mod util {
    pub mod cbor;
    pub mod indirectanswer;
    pub mod memoized;
    pub mod message;
    pub mod miscpromises;
    pub mod unlock;

    pub use self::miscpromises::CountingPromise;
    pub use self::message::DataMessage;
}

#[cfg(test)]
mod test {
    pub(crate) mod integrations {
        mod channel;
        mod commander;
        mod dauphin;
        pub(crate) use self::channel::{ TestChannelIntegration, cbor_matches, cbor_matches_print };
        pub(crate) use self::commander::{ TestCommander };
        pub(crate) use self::dauphin::FakeDauphinReceiver;
    }
    pub(crate) mod helpers;
}

pub use self::agent::{ LaneProgramLookup, LaneScaler, ShapeProgramRunAgent };
pub use self::api::{ PeregrineCore, PeregrineCoreBase, PeregrineIntegration, PeregrineApiQueue, CarriageSpeed, AgentStore };
pub use self::core::{ PeregrineConfig, Stick, StickId, StickTopology, Scale, Focus };
pub use self::index::{ StickStore, StickAuthorityStore };
pub use self::lane::{ Lane, Region, ProgramName, ProgramRegion, LaneStore, DataStore, ProgramData, ProgramRegionBuilder };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload, add_task, complete_task, async_complete_task };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager, issue_stick_request };
pub use self::shape::{ 
    ScreenEdge, SeaEnd, SeaEndPair, ShipEnd, AnchorPair, SingleAnchor, Patina, Colour, AnchorPairAxis, DirectColour, SingleAnchorAxis,
    ZMenu, Pen, Plotter, Shape, ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ShapeOutput
};
pub use self::switch::switch::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ Carriage, CarriageId };
pub use self::util::{ CountingPromise, DataMessage };
