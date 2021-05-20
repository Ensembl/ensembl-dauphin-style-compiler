mod agent {
    pub(crate) mod laneprogramlookup;

    pub use self::laneprogramlookup::LaneProgramLookup;
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

    pub use self::config::{ PgdPeregrineConfig, ConfigKey };
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
    mod shaperequest;
    pub(crate) mod programregion;
    mod resultstore;
    pub(crate) mod programdata;
    pub(crate) mod programname;

    pub use self::datastore::DataStore;
    pub use self::shaperequest::{ Region, ShapeRequest };
    pub use self::programdata::ProgramData;
    pub use self::programname::ProgramName;
    pub use self::programregion::{ ProgramRegion, ProgramRegionBuilder };
    pub use self::resultstore::{ LaneStore };
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
    mod spacebase;

    pub use self::core::{ 
        Patina, Pen, Colour, DirectColour, Plotter 
    };
    pub use self::shape::Shape;
    pub use self::spacebase::{ SpaceBase, SpaceBaseArea };
    pub use self::zmenu::ZMenu;
    pub use self::shapelist::{ ShapeListBuilder, ShapeList };
    pub use self::zmenufixed::{ ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator };
}

pub(crate) mod switch {
    pub(crate) mod allotment;
    pub(crate) mod track;
    pub(crate) mod switch;
    pub(crate) mod trackconfig;
    pub(crate) mod trackconfiglist;

    pub use self::allotment::AllotmentRequest;
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
    pub mod builder;
    pub mod cbor;
    pub mod memoized;
    pub mod message;
    pub mod miscpromises;
    pub mod ringarray;
    pub mod unlock;

    pub use self::builder::Builder;
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

pub use self::agent::{ LaneProgramLookup };
pub use self::api::{ PeregrineCore, PeregrineCoreBase, PeregrineIntegration, PeregrineApiQueue, CarriageSpeed, AgentStore };
pub use self::core::{ PgdPeregrineConfig, ConfigKey, Stick, StickId, StickTopology, Scale, Focus, Viewport };
pub use self::index::{ StickStore, StickAuthorityStore };
pub use self::lane::{ Region, ProgramName, ProgramRegion, LaneStore, DataStore, ProgramData, ProgramRegionBuilder, ShapeRequest };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload, add_task, complete_task, async_complete_task };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager, issue_stick_request };
pub use self::shape::{ 
    Patina, Colour, DirectColour,
    ZMenu, Pen, Plotter, Shape, ZMenuFixed, ZMenuFixedSequence, ZMenuFixedBlock, ZMenuFixedItem, ZMenuGenerator, ShapeListBuilder,
    SpaceBase, SpaceBaseArea
};
pub use self::switch::allotment::{ 
    AllotmentRequest, AllotmentPetitioner, AllotmentHandle, Allotter, Allotment, OffsetSize, AllotmentPositionKind,
    PositionVariant, AllotmentPosition
};
pub use self::switch::switch::{ Switches };
pub use self::switch::track::Track;
pub use self::train::{ Carriage, CarriageId };
pub use self::util::{ CountingPromise, DataMessage, Builder };
pub use self::util::ringarray::DataFilter;
