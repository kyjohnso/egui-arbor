//! egui-arbor: A flexible tree/outliner widget for egui
//!
//! This crate provides a customizable tree/outliner widget for egui applications,
//! with support for hierarchical data, expand/collapse functionality, and custom styling.

pub mod drag_drop;
pub mod outliner;
pub mod response;
pub mod state;
pub mod style;
pub mod traits;

// Re-export main types for convenience
pub use drag_drop::{DragDropState, DragDropVisuals};
pub use outliner::Outliner;
pub use response::{DropEvent, OutlinerResponse};
pub use state::OutlinerState;
pub use style::{ExpandIconStyle, Style};
pub use traits::{ActionIcon, DropPosition, IconType, OutlinerActions, OutlinerNode};