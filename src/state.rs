//! State management for the outliner widget.
//!
//! This module provides the [`OutlinerState`] struct which tracks the expansion
//! and editing state of nodes in the outliner. The state integrates with egui's
//! memory system to persist across frames.

use crate::drag_drop::DragDropState;
use std::collections::HashSet;
use std::hash::Hash;

/// State for an outliner widget instance.
///
/// This struct tracks which collection nodes are expanded and which node (if any)
/// is currently being edited. The state is generic over the node ID type and
/// integrates with egui's memory system for automatic persistence.
///
/// # Type Parameters
///
/// * `Id` - The type used to identify nodes. Must implement `Hash`, `Eq`, and `Clone`.
///
/// # Examples
///
/// ```
/// use egui_arbor::OutlinerState;
/// use std::collections::HashSet;
///
/// let mut state = OutlinerState::<String>::default();
/// 
/// // Toggle expansion state
/// state.toggle_expanded(&"node1".to_string());
/// assert!(state.is_expanded(&"node1".to_string()));
///
/// // Start editing a node
/// state.start_editing("node2".to_string());
/// assert!(state.is_editing(&"node2".to_string()));
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutlinerState<Id>
where
    Id: Hash + Eq + Clone + Send + Sync,
{
    /// Set of expanded collection node IDs.
    ///
    /// A node ID in this set indicates that the collection node is expanded
    /// and its children should be visible.
    expanded: HashSet<Id>,

    /// The ID of the node currently being edited, if any.
    ///
    /// When `Some(id)`, the node with the given ID is in edit mode (e.g., for renaming).
    /// Only one node can be edited at a time.
    editing: Option<Id>,

    /// Drag-and-drop state for this outliner.
    ///
    /// Tracks the current drag operation, hover targets, and drop positions.
    /// This field is not persisted across frames (it's transient state).
    #[cfg_attr(feature = "serde", serde(skip))]
    drag_drop: DragDropState<Id>,
}

impl<Id> Default for OutlinerState<Id>
where
    Id: Hash + Eq + Clone + Send + Sync,
{
    /// Creates a new outliner state with no expanded nodes and no editing node.
    fn default() -> Self {
        Self {
            expanded: HashSet::new(),
            editing: None,
            drag_drop: DragDropState::new(),
        }
    }
}

impl<Id> OutlinerState<Id>
where
    Id: Hash + Eq + Clone + Send + Sync,
{
    /// Loads the outliner state from egui's memory system.
    ///
    /// If no state exists for the given ID, returns a default empty state.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The egui context to load state from
    /// * `id` - The unique identifier for this outliner instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use egui_arbor::OutlinerState;
    /// # fn example(ctx: &egui::Context) {
    /// let state = OutlinerState::<String>::load(ctx, egui::Id::new("my_outliner"));
    /// # }
    /// ```
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Self
    where
        Id: 'static,
    {
        ctx.data_mut(|d| d.get_persisted(id).unwrap_or_default())
    }

    /// Stores the outliner state to egui's memory system.
    ///
    /// The state will be persisted across frames and can be retrieved using
    /// [`load`](Self::load) with the same ID.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The egui context to store state in
    /// * `id` - The unique identifier for this outliner instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use egui_arbor::OutlinerState;
    /// # fn example(ctx: &egui::Context) {
    /// let mut state = OutlinerState::<String>::default();
    /// state.toggle_expanded(&"node1".to_string());
    /// state.store(ctx, egui::Id::new("my_outliner"));
    /// # }
    /// ```
    pub fn store(&self, ctx: &egui::Context, id: egui::Id)
    where
        Id: 'static,
    {
        ctx.data_mut(|d| d.insert_persisted(id, self.clone()));
    }

    /// Checks if a node is currently expanded.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the node to check
    ///
    /// # Returns
    ///
    /// `true` if the node is expanded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.set_expanded(&"node1".to_string(), true);
    /// assert!(state.is_expanded(&"node1".to_string()));
    /// ```
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded.contains(id)
    }

    /// Toggles the expansion state of a node.
    ///
    /// If the node is currently expanded, it will be collapsed.
    /// If the node is currently collapsed, it will be expanded.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the node to toggle
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.toggle_expanded(&"node1".to_string());
    /// assert!(state.is_expanded(&"node1".to_string()));
    /// state.toggle_expanded(&"node1".to_string());
    /// assert!(!state.is_expanded(&"node1".to_string()));
    /// ```
    pub fn toggle_expanded(&mut self, id: &Id) {
        if self.expanded.contains(id) {
            self.expanded.remove(id);
        } else {
            self.expanded.insert(id.clone());
        }
    }

    /// Sets the expansion state of a node.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the node to modify
    /// * `expanded` - `true` to expand the node, `false` to collapse it
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.set_expanded(&"node1".to_string(), true);
    /// assert!(state.is_expanded(&"node1".to_string()));
    /// state.set_expanded(&"node1".to_string(), false);
    /// assert!(!state.is_expanded(&"node1".to_string()));
    /// ```
    pub fn set_expanded(&mut self, id: &Id, expanded: bool) {
        if expanded {
            self.expanded.insert(id.clone());
        } else {
            self.expanded.remove(id);
        }
    }

    /// Checks if a node is currently being edited.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the node to check
    ///
    /// # Returns
    ///
    /// `true` if the node is being edited, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.start_editing("node1".to_string());
    /// assert!(state.is_editing(&"node1".to_string()));
    /// ```
    pub fn is_editing(&self, id: &Id) -> bool {
        self.editing.as_ref() == Some(id)
    }

    /// Starts editing a node.
    ///
    /// This will stop editing any previously edited node, as only one node
    /// can be edited at a time.
    ///
    /// # Parameters
    ///
    /// * `id` - The ID of the node to start editing
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.start_editing("node1".to_string());
    /// assert!(state.is_editing(&"node1".to_string()));
    /// ```
    pub fn start_editing(&mut self, id: Id) {
        self.editing = Some(id);
    }

    /// Stops editing the currently edited node, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.start_editing("node1".to_string());
    /// state.stop_editing();
    /// assert!(!state.is_editing(&"node1".to_string()));
    /// ```
    pub fn stop_editing(&mut self) {
        self.editing = None;
    }

    /// Returns a reference to the drag-drop state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let state = OutlinerState::<String>::default();
    /// assert!(!state.drag_drop().is_dragging());
    /// ```
    pub fn drag_drop(&self) -> &DragDropState<Id> {
        &self.drag_drop
    }

    /// Returns a mutable reference to the drag-drop state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use egui_arbor::OutlinerState;
    /// let mut state = OutlinerState::<String>::default();
    /// state.drag_drop_mut().start_drag("node1".to_string());
    /// assert!(state.drag_drop().is_dragging());
    /// ```
    pub fn drag_drop_mut(&mut self) -> &mut DragDropState<Id> {
        &mut self.drag_drop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = OutlinerState::<String>::default();
        assert!(!state.is_expanded(&"test".to_string()));
        assert!(!state.is_editing(&"test".to_string()));
    }

    #[test]
    fn test_expansion() {
        let mut state = OutlinerState::<String>::default();
        let id = "node1".to_string();

        assert!(!state.is_expanded(&id));

        state.set_expanded(&id, true);
        assert!(state.is_expanded(&id));

        state.set_expanded(&id, false);
        assert!(!state.is_expanded(&id));
    }

    #[test]
    fn test_toggle_expansion() {
        let mut state = OutlinerState::<String>::default();
        let id = "node1".to_string();

        state.toggle_expanded(&id);
        assert!(state.is_expanded(&id));

        state.toggle_expanded(&id);
        assert!(!state.is_expanded(&id));
    }

    #[test]
    fn test_editing() {
        let mut state = OutlinerState::<String>::default();
        let id1 = "node1".to_string();
        let id2 = "node2".to_string();

        assert!(!state.is_editing(&id1));

        state.start_editing(id1.clone());
        assert!(state.is_editing(&id1));
        assert!(!state.is_editing(&id2));

        state.start_editing(id2.clone());
        assert!(!state.is_editing(&id1));
        assert!(state.is_editing(&id2));

        state.stop_editing();
        assert!(!state.is_editing(&id1));
        assert!(!state.is_editing(&id2));
    }
}