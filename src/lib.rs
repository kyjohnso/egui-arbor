//! egui-arbor: A hierarchical outliner widget library for egui
//!
//! egui-arbor provides a flexible, trait-based outliner widget inspired by Blender's outliner.
//! It allows you to display hierarchical data structures with collections, entities, customizable
//! icons, and drag-and-drop support.
//!
//! # Features
//!
//! - **Trait-based integration**: Implement traits on your own data structures
//! - **Hierarchical display**: Collections and entities with expand/collapse
//! - **Customizable icons**: Built-in and custom icon support
//! - **Action icons**: Right-aligned icons for common operations (visibility, lock, selection)
//! - **Drag-and-drop**: Intuitive hierarchy reorganization
//! - **egui integration**: Follows egui ecosystem conventions and patterns
//!
//! # Quick Start
//!
//! ```rust
//! use egui_arbor::{OutlinerNode, OutlinerActions, IconType, ActionIcon, DropPosition};
//!
//! // Define your data structure
//! struct SceneNode {
//!     id: u64,
//!     name: String,
//!     children: Vec<SceneNode>,
//! }
//!
//! // Implement OutlinerNode trait
//! impl OutlinerNode for SceneNode {
//!     type Id = u64;
//!
//!     fn id(&self) -> Self::Id {
//!         self.id
//!     }
//!
//!     fn name(&self) -> &str {
//!         &self.name
//!     }
//!
//!     fn is_collection(&self) -> bool {
//!         !self.children.is_empty()
//!     }
//!
//!     fn children(&self) -> &[Self] {
//!         &self.children
//!     }
//!
//!     fn children_mut(&mut self) -> &mut Vec<Self> {
//!         &mut self.children
//!     }
//! }
//!
//! // Implement OutlinerActions trait
//! struct SceneActions {
//!     selection: std::collections::HashSet<u64>,
//! }
//!
//! impl OutlinerActions<SceneNode> for SceneActions {
//!     fn on_rename(&mut self, id: &u64, new_name: String) {
//!         // Handle rename
//!     }
//!
//!     fn on_move(&mut self, id: &u64, target: &u64, position: DropPosition) {
//!         // Handle move
//!     }
//!
//!     fn on_select(&mut self, id: &u64, selected: bool) {
//!         if selected {
//!             self.selection.insert(*id);
//!         } else {
//!             self.selection.remove(id);
//!         }
//!     }
//!
//!     fn is_selected(&self, id: &u64) -> bool {
//!         self.selection.contains(id)
//!     }
//!
//!     fn is_visible(&self, id: &u64) -> bool {
//!         true // Implement your logic
//!     }
//!
//!     fn is_locked(&self, id: &u64) -> bool {
//!         false // Implement your logic
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! The library is built around two core traits:
//!
//! - [`OutlinerNode`]: Defines the interface for hierarchical nodes
//! - [`OutlinerActions`]: Handles user interactions and state changes
//!
//! Users implement these traits on their own data structures, giving them full control
//! over data storage and behavior while the library handles rendering and interaction.

mod traits;

// Re-export all public types from traits module
pub use traits::{
    ActionIcon,
    DropPosition,
    IconType,
    OutlinerActions,
    OutlinerNode,
};