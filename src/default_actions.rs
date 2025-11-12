//! Default implementation of the OutlinerActions trait.
//!
//! This module provides [`DefaultActions`], a ready-to-use implementation of the
//! [`OutlinerActions`] trait that handles common outliner state management including
//! selection, visibility, lock states, and optional event logging.
//!
//! # Examples
//!
//! ```
//! use egui_arbor::{Outliner, OutlinerNode, default_actions::DefaultActions};
//!
//! // Create actions handler with logging
//! let mut actions = DefaultActions::<u64>::with_logging(10);
//!
//! // Use with outliner
//! // let response = Outliner::new("tree").show(ui, &nodes, &mut actions);
//!
//! // Access state
//! assert_eq!(actions.selected_count(), 0);
//! assert_eq!(actions.visible_count(), 0);
//! ```

use crate::event_log::{EventLog, EventType};
use crate::traits::{DropPosition, OutlinerActions, OutlinerNode};
use std::collections::HashSet;
use std::hash::Hash;

/// Default implementation of outliner actions with state tracking.
///
/// This struct provides a complete, ready-to-use implementation of the
/// [`OutlinerActions`] trait. It tracks:
/// - **Selection state**: Which nodes are currently selected
/// - **Visibility state**: Which nodes are visible/hidden
/// - **Lock state**: Which nodes are locked
/// - **Event log**: Optional logging of all interactions
///
/// # Type Parameters
///
/// * `Id` - The type used to identify nodes. Must implement `Hash`, `Eq`, and `Clone`.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use egui_arbor::default_actions::DefaultActions;
/// use egui_arbor::OutlinerActions;
///
/// # struct TestNode { children: Vec<TestNode> }
/// # impl egui_arbor::OutlinerNode for TestNode {
/// #     type Id = u64;
/// #     fn id(&self) -> Self::Id { 0 }
/// #     fn name(&self) -> &str { "" }
/// #     fn is_collection(&self) -> bool { false }
/// #     fn children(&self) -> &[Self] { &self.children }
/// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
/// # }
/// let mut actions = DefaultActions::<u64>::new();
///
/// // All nodes start unselected, visible, and unlocked
/// assert!(!OutlinerActions::<TestNode>::is_selected(&actions, &1));
/// assert!(!OutlinerActions::<TestNode>::is_visible(&actions, &1));
/// assert!(!OutlinerActions::<TestNode>::is_locked(&actions, &1));
/// ```
///
/// ## With Event Logging
///
/// ```
/// use egui_arbor::default_actions::DefaultActions;
/// use egui_arbor::OutlinerActions;
///
/// # struct TestNode { children: Vec<TestNode> }
/// # impl egui_arbor::OutlinerNode for TestNode {
/// #     type Id = u64;
/// #     fn id(&self) -> Self::Id { 0 }
/// #     fn name(&self) -> &str { "" }
/// #     fn is_collection(&self) -> bool { false }
/// #     fn children(&self) -> &[Self] { &self.children }
/// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
/// # }
/// let mut actions = DefaultActions::<u64>::with_logging(50);
///
/// // Events are automatically logged
/// OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
/// assert_eq!(actions.event_log().unwrap().len(), 1);
/// ```
///
/// ## Pre-populate State
///
/// ```
/// use egui_arbor::default_actions::DefaultActions;
/// use egui_arbor::OutlinerActions;
/// use std::collections::HashSet;
///
/// # struct TestNode { children: Vec<TestNode> }
/// # impl egui_arbor::OutlinerNode for TestNode {
/// #     type Id = u64;
/// #     fn id(&self) -> Self::Id { 0 }
/// #     fn name(&self) -> &str { "" }
/// #     fn is_collection(&self) -> bool { false }
/// #     fn children(&self) -> &[Self] { &self.children }
/// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
/// # }
/// let mut actions = DefaultActions::<u64>::new();
///
/// // Make all nodes visible by default
/// let visible_ids: HashSet<_> = (0..10).collect();
/// actions.set_all_visible(visible_ids);
///
/// assert!(OutlinerActions::<TestNode>::is_visible(&actions, &5));
/// ```
#[derive(Clone, Debug)]
pub struct DefaultActions<Id>
where
    Id: Hash + Eq + Clone + std::fmt::Debug,
{
    /// Set of selected node IDs.
    selected: HashSet<Id>,

    /// Set of visible node IDs.
    visible: HashSet<Id>,

    /// Set of locked node IDs.
    locked: HashSet<Id>,

    /// Optional event log for tracking interactions.
    event_log: Option<EventLog<Id>>,
}

impl<Id> DefaultActions<Id>
where
    Id: Hash + Eq + Clone + std::fmt::Debug,
{
    /// Creates a new actions handler with no event logging.
    ///
    /// All nodes start unselected, invisible, and unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    ///
    /// let actions = DefaultActions::<u64>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            visible: HashSet::new(),
            locked: HashSet::new(),
            event_log: None,
        }
    }

    /// Creates a new actions handler with event logging enabled.
    ///
    /// # Arguments
    ///
    /// * `max_log_entries` - Maximum number of log entries to keep
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    ///
    /// let actions = DefaultActions::<u64>::with_logging(100);
    /// ```
    pub fn with_logging(max_log_entries: usize) -> Self {
        Self {
            selected: HashSet::new(),
            visible: HashSet::new(),
            locked: HashSet::new(),
            event_log: Some(EventLog::new(max_log_entries)),
        }
    }

    /// Returns a reference to the event log, if logging is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    ///
    /// let actions = DefaultActions::<u64>::with_logging(10);
    /// assert!(actions.event_log().is_some());
    ///
    /// let actions = DefaultActions::<u64>::new();
    /// assert!(actions.event_log().is_none());
    /// ```
    pub fn event_log(&self) -> Option<&EventLog<Id>> {
        self.event_log.as_ref()
    }

    /// Returns a mutable reference to the event log, if logging is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    ///
    /// let mut actions = DefaultActions::<u64>::with_logging(10);
    /// if let Some(log) = actions.event_log_mut() {
    ///     log.clear();
    /// }
    /// ```
    pub fn event_log_mut(&mut self) -> Option<&mut EventLog<Id>> {
        self.event_log.as_mut()
    }

    /// Returns the number of selected nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);
    /// assert_eq!(actions.selected_count(), 2);
    /// ```
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Returns the number of visible nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// OutlinerActions::<TestNode>::on_visibility_toggle(&mut actions, &1);
    /// assert_eq!(actions.visible_count(), 1);
    /// ```
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Returns the number of locked nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// OutlinerActions::<TestNode>::on_lock_toggle(&mut actions, &1);
    /// assert_eq!(actions.locked_count(), 1);
    /// ```
    pub fn locked_count(&self) -> usize {
        self.locked.len()
    }

    /// Returns a reference to the set of selected node IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);
    ///
    /// assert!(actions.selected().contains(&1));
    /// assert!(actions.selected().contains(&2));
    /// ```
    pub fn selected(&self) -> &HashSet<Id> {
        &self.selected
    }

    /// Returns a reference to the set of visible node IDs.
    pub fn visible(&self) -> &HashSet<Id> {
        &self.visible
    }

    /// Returns a reference to the set of locked node IDs.
    pub fn locked(&self) -> &HashSet<Id> {
        &self.locked
    }

    /// Sets all nodes as visible.
    ///
    /// # Arguments
    ///
    /// * `ids` - Set of node IDs to mark as visible
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    /// use std::collections::HashSet;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// let visible: HashSet<_> = (0..10).collect();
    /// actions.set_all_visible(visible);
    ///
    /// assert!(OutlinerActions::<TestNode>::is_visible(&actions, &5));
    /// ```
    pub fn set_all_visible(&mut self, ids: HashSet<Id>) {
        self.visible = ids;
    }

    /// Clears all selections.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::default_actions::DefaultActions;
    /// use egui_arbor::OutlinerActions;
    ///
    /// # struct TestNode { children: Vec<TestNode> }
    /// # impl egui_arbor::OutlinerNode for TestNode {
    /// #     type Id = u64;
    /// #     fn id(&self) -> Self::Id { 0 }
    /// #     fn name(&self) -> &str { "" }
    /// #     fn is_collection(&self) -> bool { false }
    /// #     fn children(&self) -> &[Self] { &self.children }
    /// #     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
    /// # }
    /// let mut actions = DefaultActions::<u64>::new();
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
    /// OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);
    /// assert_eq!(actions.selected_count(), 2);
    ///
    /// actions.clear_selection();
    /// assert_eq!(actions.selected_count(), 0);
    /// ```
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Logs an event if logging is enabled.
    fn log_event(&mut self, message: String, event_type: EventType, node_id: Option<Id>) {
        if let Some(log) = &mut self.event_log {
            log.log(message, event_type, node_id);
        }
    }
}

impl<Id> Default for DefaultActions<Id>
where
    Id: Hash + Eq + Clone + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, N> OutlinerActions<N> for DefaultActions<Id>
where
    Id: Hash + Eq + Clone + std::fmt::Debug,
    N: OutlinerNode<Id = Id>,
{
    fn on_rename(&mut self, id: &Id, new_name: String) {
        self.log_event(
            format!("Renamed node {:?} to '{}'", id, new_name),
            EventType::Rename,
            Some(id.clone()),
        );
    }

    fn on_move(&mut self, id: &Id, target: &Id, position: DropPosition) {
        self.log_event(
            format!("Move: node {:?} → target {:?} ({:?})", id, target, position),
            EventType::DragDrop,
            Some(id.clone()),
        );
    }

    fn on_select(&mut self, id: &Id, selected: bool) {
        if selected {
            self.selected.insert(id.clone());
            self.log_event(
                format!("Selected node {:?}", id),
                EventType::Selection,
                Some(id.clone()),
            );
        } else {
            self.selected.remove(id);
            self.log_event(
                format!("Deselected node {:?}", id),
                EventType::Selection,
                Some(id.clone()),
            );
        }
    }

    fn is_selected(&self, id: &Id) -> bool {
        self.selected.contains(id)
    }

    fn is_visible(&self, id: &Id) -> bool {
        self.visible.contains(id)
    }

    fn is_locked(&self, id: &Id) -> bool {
        self.locked.contains(id)
    }

    fn on_visibility_toggle(&mut self, id: &Id) {
        let was_visible = self.visible.contains(id);
        if was_visible {
            self.visible.remove(id);
            self.log_event(
                format!("Hidden node {:?}", id),
                EventType::Visibility,
                Some(id.clone()),
            );
        } else {
            self.visible.insert(id.clone());
            self.log_event(
                format!("Shown node {:?}", id),
                EventType::Visibility,
                Some(id.clone()),
            );
        }
    }

    fn on_lock_toggle(&mut self, id: &Id) {
        let was_locked = self.locked.contains(id);
        if was_locked {
            self.locked.remove(id);
            self.log_event(
                format!("Unlocked node {:?}", id),
                EventType::Lock,
                Some(id.clone()),
            );
        } else {
            self.locked.insert(id.clone());
            self.log_event(
                format!("Locked node {:?}", id),
                EventType::Lock,
                Some(id.clone()),
            );
        }
    }

    fn on_selection_toggle(&mut self, id: &Id) {
        let is_selected = OutlinerActions::<N>::is_selected(self, id);
        OutlinerActions::<N>::on_select(self, id, !is_selected);
    }

    fn on_custom_action(&mut self, id: &Id, icon: &str) {
        self.log_event(
            format!("Custom action '{}' on node {:?}", icon, id),
            EventType::Custom(icon.to_string()),
            Some(id.clone()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ActionIcon, IconType};

    #[derive(Clone, Debug)]
    struct TestNode {
        id: u64,
        name: String,
        children: Vec<TestNode>,
    }

    impl OutlinerNode for TestNode {
        type Id = u64;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_collection(&self) -> bool {
            !self.children.is_empty()
        }

        fn children(&self) -> &[Self] {
            &self.children
        }

        fn children_mut(&mut self) -> &mut Vec<Self> {
            &mut self.children
        }

        fn icon(&self) -> Option<IconType> {
            None
        }

        fn action_icons(&self) -> Vec<ActionIcon> {
            vec![]
        }
    }

    #[test]
    fn test_new() {
        let actions = DefaultActions::<u64>::new();
        assert_eq!(actions.selected_count(), 0);
        assert_eq!(actions.visible_count(), 0);
        assert_eq!(actions.locked_count(), 0);
        assert!(actions.event_log().is_none());
    }

    #[test]
    fn test_with_logging() {
        let actions = DefaultActions::<u64>::with_logging(10);
        assert!(actions.event_log().is_some());
        assert_eq!(actions.event_log().unwrap().max_entries(), 10);
    }

    #[test]
    fn test_selection() {
        let mut actions = DefaultActions::<u64>::new();

        assert!(!OutlinerActions::<TestNode>::is_selected(&actions, &1));

        OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
        assert!(OutlinerActions::<TestNode>::is_selected(&actions, &1));
        assert_eq!(actions.selected_count(), 1);

        OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);
        assert_eq!(actions.selected_count(), 2);

        OutlinerActions::<TestNode>::on_select(&mut actions, &1, false);
        assert!(!OutlinerActions::<TestNode>::is_selected(&actions, &1));
        assert_eq!(actions.selected_count(), 1);
    }

    #[test]
    fn test_visibility() {
        let mut actions = DefaultActions::<u64>::new();

        assert!(!OutlinerActions::<TestNode>::is_visible(&actions, &1));

        OutlinerActions::<TestNode>::on_visibility_toggle(&mut actions, &1);
        assert!(OutlinerActions::<TestNode>::is_visible(&actions, &1));
        assert_eq!(actions.visible_count(), 1);

        OutlinerActions::<TestNode>::on_visibility_toggle(&mut actions, &1);
        assert!(!OutlinerActions::<TestNode>::is_visible(&actions, &1));
        assert_eq!(actions.visible_count(), 0);
    }

    #[test]
    fn test_lock() {
        let mut actions = DefaultActions::<u64>::new();

        assert!(!OutlinerActions::<TestNode>::is_locked(&actions, &1));

        OutlinerActions::<TestNode>::on_lock_toggle(&mut actions, &1);
        assert!(OutlinerActions::<TestNode>::is_locked(&actions, &1));
        assert_eq!(actions.locked_count(), 1);

        OutlinerActions::<TestNode>::on_lock_toggle(&mut actions, &1);
        assert!(!OutlinerActions::<TestNode>::is_locked(&actions, &1));
        assert_eq!(actions.locked_count(), 0);
    }

    #[test]
    fn test_selection_toggle() {
        let mut actions = DefaultActions::<u64>::new();

        OutlinerActions::<TestNode>::on_selection_toggle(&mut actions, &1);
        assert!(OutlinerActions::<TestNode>::is_selected(&actions, &1));

        OutlinerActions::<TestNode>::on_selection_toggle(&mut actions, &1);
        assert!(!OutlinerActions::<TestNode>::is_selected(&actions, &1));
    }

    #[test]
    fn test_clear_selection() {
        let mut actions = DefaultActions::<u64>::new();

        OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
        OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);
        OutlinerActions::<TestNode>::on_select(&mut actions, &3, true);
        assert_eq!(actions.selected_count(), 3);

        actions.clear_selection();
        assert_eq!(actions.selected_count(), 0);
    }

    #[test]
    fn test_set_all_visible() {
        let mut actions = DefaultActions::<u64>::new();

        let visible: HashSet<_> = (0..5).collect();
        actions.set_all_visible(visible);

        assert_eq!(actions.visible_count(), 5);
        assert!(OutlinerActions::<TestNode>::is_visible(&actions, &0));
        assert!(OutlinerActions::<TestNode>::is_visible(&actions, &4));
        assert!(!OutlinerActions::<TestNode>::is_visible(&actions, &5));
    }

    #[test]
    fn test_event_logging() {
        let mut actions = DefaultActions::<u64>::with_logging(10);

        OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
        OutlinerActions::<TestNode>::on_visibility_toggle(&mut actions, &2);
        OutlinerActions::<TestNode>::on_lock_toggle(&mut actions, &3);

        let log = actions.event_log().unwrap();
        assert_eq!(log.len(), 3);

        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].event_type, EventType::Lock);
        assert_eq!(entries[1].event_type, EventType::Visibility);
        assert_eq!(entries[2].event_type, EventType::Selection);
    }

    #[test]
    fn test_on_rename_logging() {
        let mut actions = DefaultActions::<u64>::with_logging(10);

        OutlinerActions::<TestNode>::on_rename(&mut actions, &1, "New Name".to_string());

        let log = actions.event_log().unwrap();
        assert_eq!(log.len(), 1);
        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].event_type, EventType::Rename);
    }

    #[test]
    fn test_on_move_logging() {
        let mut actions = DefaultActions::<u64>::with_logging(10);

        OutlinerActions::<TestNode>::on_move(&mut actions, &1, &2, DropPosition::Inside);

        let log = actions.event_log().unwrap();
        assert_eq!(log.len(), 1);
        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].event_type, EventType::DragDrop);
    }

    #[test]
    fn test_on_custom_action() {
        let mut actions = DefaultActions::<u64>::with_logging(10);

        OutlinerActions::<TestNode>::on_custom_action(&mut actions, &1, "custom_icon");

        let log = actions.event_log().unwrap();
        assert_eq!(log.len(), 1);
        let entries: Vec<_> = log.entries().collect();
        assert!(matches!(entries[0].event_type, EventType::Custom(_)));
    }

    #[test]
    fn test_default() {
        let actions = DefaultActions::<u64>::default();
        assert_eq!(actions.selected_count(), 0);
        assert!(actions.event_log().is_none());
    }

    #[test]
    fn test_selected_reference() {
        let mut actions = DefaultActions::<u64>::new();
        OutlinerActions::<TestNode>::on_select(&mut actions, &1, true);
        OutlinerActions::<TestNode>::on_select(&mut actions, &2, true);

        let selected = actions.selected();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&1));
        assert!(selected.contains(&2));
    }
}
