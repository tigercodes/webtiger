//! What every other crate of the core stands on: ids, addresses, the
//! navigation policy and clocks. No I/O, no async, no platform code.

pub mod address;
pub mod clock;
pub mod download;
pub mod id;
pub mod policy;
mod search;
mod space;
pub mod uuid;
pub mod zoom;

pub use address::{Address, AddressError, Input, Security, classify, search_address};
pub use clock::{Clock, ManualClock, SystemClock, Timestamp};
pub use download::download_file_name;
pub use id::{IdMint, LoginSetId, SpaceId, SubscriptionId, TabId, WebViewId, WindowId};
pub use policy::{NavigationAction, check_navigation, opens_outside};
pub use search::{
    SEARCH_ENGINES, SEARCHED_LENGTH, STAND_IN_SEARCH_ENGINE, SearchEngine, search_engine,
    searched_words,
};
pub use space::{MOST_SPACES, SPACE_NAME_LENGTH, space_name};
pub use uuid::Uuid;
pub use zoom::{ACTUAL_SIZE, ZOOM_LEVELS, zoom_in, zoom_out};
