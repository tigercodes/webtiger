//! What every other crate of the core stands on: ids, addresses, the
//! navigation policy and clocks. No I/O, no async, no platform code.

pub mod address;
pub mod clock;
pub mod id;
pub mod policy;
pub mod uuid;

pub use address::{Address, AddressError, Input, Security, classify, search_address};
pub use clock::{Clock, ManualClock, SystemClock, Timestamp};
pub use id::{IdMint, LoginSetId, SpaceId, SubscriptionId, TabId, WebViewId, WindowId};
pub use policy::{NavigationAction, check_navigation, opens_outside};
pub use uuid::Uuid;
