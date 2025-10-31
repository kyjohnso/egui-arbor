//! Core trait system for egui-arbor outliner nodes and actions.
//!
//! This module defines the fundamental traits that users implement to integrate
//! their data structures with the outliner widget.

use std::hash::Hash;

/// Represents a node in the outliner hierarchy.
///
/// Users implement this trait on their own data structures to integrate with
/// the outliner widget. The trait provides methods for accessing node properties,
/// hierarchy information, and visual customization.
///
/// # Type Parameters
///
/// * `Id` - A unique identifier type that must be hashable, comparable, and cloneable.
///   This is used internally for state management and node tracking.
///
/// # Example
///
/// ```rust
/// use egui_arbor::{OutlinerNode, IconType, ActionIcon};
///
/// struct SceneNode {
///     id: u64,
///     name: String,
///     children: Vec<SceneNode>,
/// }
///
/// impl OutlinerNode for SceneNode {
///     type Id = u64;
///
///     fn id(&self) -> Self::Id {
///         self.id
///     }
///
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn is_collection(&self) -> bool {
///         !self.children.is_empty()
///     }
///
///     fn children(&self) -> &[Self] {
///         &self.children
///     }
///
///     fn children_mut(&mut self) -> &mut Vec<Self> {
///         &mut self.children
///     }
/// }
/// ```
pub trait OutlinerNode: Sized {
    /// The type used to uniquely identify nodes.
    ///
    /// Must implement [`Hash`], [`Eq`], [`Clone`], [`Send`], and [`Sync`] for use in internal state management.
    type Id: Hash + Eq + Clone + Send + Sync;

    /// Returns the unique identifier for this node.
    ///
    /// This ID is used for state tracking, selection management, and drag-drop operations.
    /// It must be stable across frames and unique within the hierarchy.
    fn id(&self) -> Self::Id;

    /// Returns the display name of the node.
    ///
    /// This is the text shown in the outliner next to the node's icon.
    fn name(&self) -> &str;

    /// Returns whether this node can contain children.
    ///
    /// Collections display an expand/collapse arrow and can have child nodes.
    /// Non-collection nodes (entities) cannot have children.
    fn is_collection(&self) -> bool;

    /// Returns an immutable slice of this node's children.
    ///
    /// For non-collection nodes, this should return an empty slice.
    /// For collection nodes, this returns all direct children.
    fn children(&self) -> &[Self];

    /// Returns a mutable reference to this node's children vector.
    ///
    /// This is used for drag-drop operations and hierarchy modifications.
    /// For non-collection nodes, this should return an empty vector.
    fn children_mut(&mut self) -> &mut Vec<Self>;

    /// Returns the icon to display next to the node name.
    ///
    /// If `None`, no icon is displayed. The default implementation returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_arbor::{OutlinerNode, IconType};
    /// # struct MyNode { children: Vec<MyNode> }
    /// # impl OutlinerNode for MyNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// fn icon(&self) -> Option<IconType> {
    ///     if self.is_collection() {
    ///         Some(IconType::Collection)
    ///     } else {
    ///         Some(IconType::Entity)
    ///     }
    /// }
    /// # }
    /// ```
    fn icon(&self) -> Option<IconType> {
        None
    }

    /// Returns the action icons to display on the right side of the node.
    ///
    /// These icons are right-aligned and provide quick access to common operations
    /// like visibility toggling, locking, and selection.
    ///
    /// The default implementation returns the standard set of action icons:
    /// visibility, lock, and selection.
    fn action_icons(&self) -> Vec<ActionIcon> {
        vec![
            ActionIcon::Visibility,
            ActionIcon::Lock,
            ActionIcon::Selection,
        ]
    }
}

/// Handles user interactions and state changes for outliner nodes.
///
/// This trait defines callbacks for various outliner operations. Users implement
/// this trait to respond to user actions like renaming, moving, selecting nodes,
/// and toggling node states.
///
/// # Type Parameters
///
/// * `N` - The node type that implements [`OutlinerNode`]
///
/// # Example
///
/// ```rust
/// use egui_arbor::{OutlinerNode, OutlinerActions, DropPosition};
/// use std::collections::HashSet;
///
/// # struct SceneNode { id: u64, name: String, children: Vec<SceneNode> }
/// # impl OutlinerNode for SceneNode {
/// #     type Id = u64;
/// #     fn id(&self) -> Self::Id { self.id }
/// #     fn name(&self) -> &str { &self.name }
/// #     fn is_collection(&self) -> bool { !self.children.is_empty() }
/// #     fn children(&self) -> &[Self] { &self.children }
/// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
/// # }
/// struct SceneActions {
///     selection: HashSet<u64>,
///     visible: HashSet<u64>,
///     locked: HashSet<u64>,
/// }
///
/// impl OutlinerActions<SceneNode> for SceneActions {
///     fn on_rename(&mut self, id: &u64, new_name: String) {
///         // Update node name in your data structure
///     }
///
///     fn on_move(&mut self, id: &u64, target: &u64, position: DropPosition) {
///         // Handle node reparenting
///     }
///
///     fn on_select(&mut self, id: &u64, selected: bool) {
///         if selected {
///             self.selection.insert(*id);
///         } else {
///             self.selection.remove(id);
///         }
///     }
///
///     fn is_selected(&self, id: &u64) -> bool {
///         self.selection.contains(id)
///     }
///
///     fn is_visible(&self, id: &u64) -> bool {
///         self.visible.contains(id)
///     }
///
///     fn is_locked(&self, id: &u64) -> bool {
///         self.locked.contains(id)
///     }
///
///     fn on_visibility_toggle(&mut self, id: &u64) {
///         if self.visible.contains(id) {
///             self.visible.remove(id);
///         } else {
///             self.visible.insert(*id);
///         }
///     }
///
///     fn on_lock_toggle(&mut self, id: &u64) {
///         if self.locked.contains(id) {
///             self.locked.remove(id);
///         } else {
///             self.locked.insert(*id);
///         }
///     }
///
///     fn on_selection_toggle(&mut self, id: &u64) {
///         let is_selected = self.is_selected(id);
///         self.on_select(id, !is_selected);
///     }
///
///     fn on_custom_action(&mut self, _id: &u64, _icon: &str) {
///         // Handle custom action
///     }
/// }
/// ```
pub trait OutlinerActions<N: OutlinerNode> {
    /// Called when a node is renamed by the user.
    ///
    /// This is triggered when the user finishes editing a node's name
    /// (e.g., by pressing Enter or clicking outside the text field).
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node being renamed
    /// * `new_name` - The new name entered by the user
    fn on_rename(&mut self, id: &N::Id, new_name: String);

    /// Called when a node is moved via drag-and-drop.
    ///
    /// This is triggered when the user successfully completes a drag-drop operation.
    /// The implementation should update the hierarchy to reflect the new position.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node being moved
    /// * `target` - The unique identifier of the target node
    /// * `position` - Where to place the node relative to the target
    fn on_move(&mut self, id: &N::Id, target: &N::Id, position: DropPosition);

    /// Called when a node's selection state changes.
    ///
    /// This is triggered when the user clicks on a node or uses keyboard navigation
    /// to change the selection.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node
    /// * `selected` - Whether the node should be selected or deselected
    fn on_select(&mut self, id: &N::Id, selected: bool);

    /// Returns whether a node is currently selected.
    ///
    /// This is used to determine visual highlighting and multi-selection state.
    fn is_selected(&self, id: &N::Id) -> bool;

    /// Returns whether a node is currently visible.
    ///
    /// This affects the state of the visibility action icon. The interpretation
    /// of "visible" is up to the implementation (e.g., visible in a 3D viewport,
    /// visible in a list, etc.).
    fn is_visible(&self, id: &N::Id) -> bool;

    /// Returns whether a node is currently locked.
    ///
    /// This affects the state of the lock action icon. The interpretation of
    /// "locked" is up to the implementation (e.g., locked from editing, locked
    /// from selection, etc.).
    fn is_locked(&self, id: &N::Id) -> bool;

    /// Called when the visibility action icon is clicked.
    ///
    /// This is triggered when the user clicks the visibility icon (eye icon).
    /// The implementation should toggle the visibility state of the node.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node whose visibility is being toggled
    fn on_visibility_toggle(&mut self, id: &N::Id);

    /// Called when the lock action icon is clicked.
    ///
    /// This is triggered when the user clicks the lock icon.
    /// The implementation should toggle the lock state of the node.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node whose lock state is being toggled
    fn on_lock_toggle(&mut self, id: &N::Id);

    /// Called when the selection action icon is clicked.
    ///
    /// This is triggered when the user clicks the selection icon (checkbox).
    /// The implementation should toggle the selection state of the node.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node whose selection is being toggled
    fn on_selection_toggle(&mut self, id: &N::Id);

    /// Called when a custom action icon is clicked.
    ///
    /// This is triggered when the user clicks a custom action icon.
    /// The implementation should handle the custom action based on the icon identifier.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the node
    /// * `icon` - The icon identifier from the custom action icon
    fn on_custom_action(&mut self, id: &N::Id, icon: &str);
}

/// The type of icon to display next to a node.
///
/// This enum defines the built-in icon types that can be used for nodes.
/// Custom icons can be added using the `Custom` variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IconType {
    /// Icon for collection nodes (nodes that can contain children)
    Collection,
    
    /// Icon for entity nodes (leaf nodes)
    Entity,
    
    /// Custom icon with a user-defined identifier
    ///
    /// The string can be used to look up custom icon rendering logic
    /// or to specify an icon from an external icon set.
    Custom(String),
}

/// Action icons displayed on the right side of each node.
///
/// These icons provide quick access to common operations and display
/// the current state of various node properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionIcon {
    /// Toggle visibility of the node
    ///
    /// Typically displayed as an eye icon. The visual state reflects
    /// the result of [`OutlinerActions::is_visible`].
    Visibility,
    
    /// Toggle lock state of the node
    ///
    /// Typically displayed as a lock/unlock icon. The visual state reflects
    /// the result of [`OutlinerActions::is_locked`].
    Lock,
    
    /// Toggle selection state of the node
    ///
    /// Typically displayed as a checkbox or selection indicator. The visual
    /// state reflects the result of [`OutlinerActions::is_selected`].
    Selection,
    
    /// Custom action icon with user-defined behavior
    ///
    /// # Fields
    ///
    /// * `icon` - Identifier for the icon (e.g., emoji, icon name, or custom identifier)
    /// * `tooltip` - Optional tooltip text to display on hover
    Custom {
        /// The icon identifier or character to display
        icon: String,
        /// Optional tooltip text
        tooltip: Option<String>,
    },
}

/// Specifies where a node should be placed relative to a target during drag-drop.
///
/// This enum is used in [`OutlinerActions::on_move`] to indicate the desired
/// position of the dragged node relative to the drop target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropPosition {
    /// Place the node before the target (as a sibling)
    Before,
    
    /// Place the node after the target (as a sibling)
    After,
    
    /// Place the node inside the target (as a child)
    ///
    /// This is only valid if the target is a collection node.
    Inside,
}