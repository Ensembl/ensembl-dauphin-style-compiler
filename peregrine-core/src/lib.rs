mod api {
    mod api;
    mod objects;
    mod queue;

    pub use api::{ PeregrineApi, PeregrineIntegration, CarriageSpeed };
    pub use self::objects::PeregrineObjects;
    pub use queue::PeregrineApiQueue;
}

mod core {
    mod config;
    pub mod focus;
    mod layout;
    mod scale;
    pub mod stick;
    pub mod track;
    mod viewport;

    pub use self::config::{ PeregrineConfig, ConfigValue };
    pub use self::focus::Focus;
    pub use self::layout::Layout;
    pub use self::scale::Scale;
    pub use stick::{ StickId, Stick, StickTopology };
    pub use track::Track;
    pub use viewport::Viewport;
}

mod index {
    pub(crate) mod stickstore;
    pub(crate) mod stickauthority;
    pub(crate) mod stickauthoritystore;
    pub use self::stickstore::StickStore;
    pub use self::stickauthoritystore::StickAuthorityStore;
}

mod panel {
    mod datastore;
    mod panel;
    mod panelprogramstore;
    mod programregion;
    mod panelrunstore;
    mod panelstore;
    mod programdata;
    pub use self::datastore::DataStore;
    pub use self::panel::{ Panel };
    pub use self::programdata::ProgramData;
    pub use self::programregion::ProgramRegion;
    pub use self::panelrunstore::{ PanelRunStore, PanelRunOutput };
    pub use self::panelprogramstore::PanelProgramStore;
    pub use self::panelstore::PanelStore;
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
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec };
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
    pub mod fuse;
    pub mod memoized;
    pub mod miscpromises;
    pub mod unlock;

    pub use self::miscpromises::CountingPromise;
}

#[cfg(test)]
mod test {
    pub(crate) mod integrations {
        mod channel;
        mod commander;
        mod console;
        mod dauphin;
        pub(crate) use self::console::TestConsole;
        pub(crate) use self::channel::{ TestChannelIntegration, cbor_matches, cbor_matches_print };
        pub(crate) use self::commander::{ TestCommander };
        pub(crate) use self::dauphin::FakeDauphinReceiver;
    }
    pub(crate) mod helpers;
}

pub use self::api::{ PeregrineObjects, PeregrineIntegration, PeregrineApi, PeregrineApiQueue, CarriageSpeed };
pub use self::core::{ PeregrineConfig, Stick, StickId, StickTopology, Track, Scale, Focus };
pub use self::index::{ StickStore, StickAuthorityStore };
pub use self::panel::{ Panel, PanelProgramStore, PanelRunStore, ProgramRegion, PanelRunOutput, PanelStore, DataStore, ProgramData };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgDauphin, Commander, InstancePayload };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager, issue_stick_request };
pub use self::shape::{ 
    ScreenEdge, SeaEnd, SeaEndPair, ShipEnd, AnchorPair, SingleAnchor, Patina, Colour, AnchorPairAxis, DirectColour, SingleAnchorAxis,
    ZMenu, Pen, Plotter, Shape
};
pub use self::train::{ Carriage, CarriageId };
pub use self::util::CountingPromise;
