pub mod events;
pub mod render;
pub mod views;

pub use events::handle_key;
pub use render::render;

pub const TAB_TITLES: [&str; 5] = ["Cast", "Monitors", "Audio", "Profiles", "Troubleshoot"];
