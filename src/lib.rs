//! A flexible tree/outliner widget for [egui](https://github.com/emilk/egui).
//!
//! `egui-arbor` provides a customizable hierarchical tree view widget inspired by
//! Blender's outliner, designed to integrate seamlessly with egui applications.
//!
//! # Features
//!
//! - **Hierarchical Tree View**: Display nested data structures with collections and entities
//! - **Expand/Collapse**: Navigate through tree hierarchy with visual expand/collapse arrows
//! - **Drag & Drop**: Reorder and reparent nodes with Before/After/Inside positioning
//! - **Action Icons**: Built-in visibility, lock, and selection toggles with custom icon support
//! - **Inline Editing**: Double-click to rename nodes with keyboard shortcuts
//! - **Multi-Selection**: Select multiple nodes with Shift-click, Ctrl/Cmd-click, or box selection
//! - **Customizable Styling**: Configure indentation, colors, icons, and spacing
//! - **Trait-Based Integration**: Works with any data structure implementing [`OutlinerNode`]
//! - **State Persistence**: Automatic state management via egui's memory system
//!
//! # Multi-Selection
//!
//! The outliner supports multiple selection modes:
//! - **Click**: Select a single node (clears other selections)
//! - **Ctrl/Cmd-Click**: Toggle selection of a node without clearing others
//! - **Shift-Click**: Select a range of nodes from the last selected to the clicked node
//! - **Box Selection**: Click and drag in empty space to select multiple nodes with a selection box
//!   - Hold Ctrl/Cmd while box selecting to add to existing selection
//!
//! # Quick Start
//!
//! To use the outliner, you need to:
//! 1. Implement [`OutlinerNode`] on your data structure
//! 2. Implement [`OutlinerActions`] to handle user interactions
//! 3. Create an [`Outliner`] and call its [`show`](Outliner::show) method
//!
//! # Example
//!
//! ```rust
//! use egui_arbor::{Outliner, OutlinerNode, OutlinerActions, ActionIcon, DropPosition};
//!
//! // 1. Define your data structure
//! #[derive(Clone)]
//! struct TreeNode {
//!     id: u64,
//!     name: String,
//!     children: Vec<TreeNode>,
//! }
//!
//! // 2. Implement OutlinerNode trait
//! impl OutlinerNode for TreeNode {
//!     type Id = u64;
//!
//!     fn id(&self) -> Self::Id { self.id }
//!     fn name(&self) -> &str { &self.name }
//!     fn is_collection(&self) -> bool { !self.children.is_empty() }
//!     fn children(&self) -> &[Self] { &self.children }
//!     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
//! }
//!
//! // 3. Implement OutlinerActions trait
//! struct MyActions {
//!     selected: Option<u64>,
//! }
//!
//! impl OutlinerActions<TreeNode> for MyActions {
//!     fn on_select(&mut self, id: &u64, selected: bool) {
//!         self.selected = if selected { Some(*id) } else { None };
//!     }
//!
//!     fn is_selected(&self, id: &u64) -> bool {
//!         self.selected == Some(*id)
//!     }
//!
//!     // Implement other required methods...
//!     fn on_rename(&mut self, _id: &u64, _new_name: String) {}
//!     fn on_move(&mut self, _id: &u64, _target: &u64, _position: DropPosition) {}
//!     fn is_visible(&self, _id: &u64) -> bool { true }
//!     fn is_locked(&self, _id: &u64) -> bool { false }
//!     fn on_visibility_toggle(&mut self, _id: &u64) {}
//!     fn on_lock_toggle(&mut self, _id: &u64) {}
//!     fn on_selection_toggle(&mut self, _id: &u64) {}
//!     fn on_custom_action(&mut self, _id: &u64, _icon: &str) {}
//! }
//!
//! // 4. Use in your egui code
//! fn show_tree(ui: &mut egui::Ui, nodes: &[TreeNode], actions: &mut MyActions) {
//!     let response = Outliner::new("my_tree")
//!         .show(ui, nodes, actions);
//!
//!     // Handle events
//!     if let Some(id) = response.selected() {
//!         println!("Selected node: {:?}", id);
//!     }
//! }
//! ```
//!
//! # Core Types
//!
//! - [`Outliner`] - The main widget for rendering hierarchical trees
//! - [`OutlinerNode`] - Trait to implement on your data structures
//! - [`OutlinerActions`] - Trait for handling user interactions
//! - [`OutlinerResponse`] - Response type containing event information
//! - [`OutlinerState`] - Persistent state for expansion and editing
//! - [`Style`] - Visual styling configuration
//! - [`DragDropState`] - State tracking for drag-drop operations
//!
//! # Optional Features
//!
//! - `serde` - Enable serialization support for state persistence

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
pub use state::{BoxSelectionState, OutlinerState};
pub use style::{ExpandIconStyle, Style};
pub use traits::{ActionIcon, DropPosition, IconType, OutlinerActions, OutlinerNode};