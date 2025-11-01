//! Drag-and-drop functionality for the outliner widget.
//!
//! This module provides types and utilities for implementing drag-and-drop
//! operations in the outliner, including state tracking, drop validation,
//! and visual feedback.

use crate::traits::{DropPosition, OutlinerNode};
use std::hash::Hash;

/// Tracks the current drag-and-drop state for the outliner.
///
/// This structure maintains information about ongoing drag operations,
/// including which node is being dragged and potential drop targets.
#[derive(Debug, Clone)]
pub struct DragDropState<Id>
where
    Id: Hash + Eq + Clone,
{
    /// The ID of the node currently being dragged, if any.
    pub dragging: Option<Id>,

    /// The ID of the node currently being hovered over as a potential drop target.
    pub hover_target: Option<Id>,

    /// The position where the dragged node would be dropped relative to the hover target.
    pub drop_position: Option<DropPosition>,
}

impl<Id> Default for DragDropState<Id>
where
    Id: Hash + Eq + Clone,
{
    fn default() -> Self {
        Self {
            dragging: None,
            hover_target: None,
            drop_position: None,
        }
    }
}

impl<Id> DragDropState<Id>
where
    Id: Hash + Eq + Clone,
{
    /// Creates a new drag-drop state with no active operations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts dragging a node.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the node being dragged
    pub fn start_drag(&mut self, id: Id) {
        self.dragging = Some(id);
        self.hover_target = None;
        self.drop_position = None;
    }

    /// Updates the hover target and drop position.
    ///
    /// # Arguments
    ///
    /// * `target` - The ID of the node being hovered over
    /// * `position` - The position where the drop would occur
    pub fn update_hover(&mut self, target: Id, position: DropPosition) {
        self.hover_target = Some(target);
        self.drop_position = Some(position);
    }

    /// Clears the hover state.
    pub fn clear_hover(&mut self) {
        self.hover_target = None;
        self.drop_position = None;
    }

    /// Ends the drag operation and returns the drop information if valid.
    ///
    /// # Returns
    ///
    /// A tuple of `(source_id, target_id, position)` if a valid drop occurred,
    /// or `None` if the drag was cancelled or invalid.
    pub fn end_drag(&mut self) -> Option<(Id, Id, DropPosition)> {
        let result = if let (Some(source), Some(target), Some(position)) =
            (&self.dragging, &self.hover_target, &self.drop_position)
        {
            Some((source.clone(), target.clone(), *position))
        } else {
            None
        };

        self.dragging = None;
        self.hover_target = None;
        self.drop_position = None;

        result
    }

    /// Cancels the current drag operation.
    pub fn cancel_drag(&mut self) {
        self.dragging = None;
        self.hover_target = None;
        self.drop_position = None;
    }

    /// Returns whether a drag operation is currently active.
    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    /// Returns the ID of the node being dragged, if any.
    pub fn dragging_id(&self) -> Option<&Id> {
        self.dragging.as_ref()
    }

    /// Returns whether the given node is currently being dragged.
    pub fn is_dragging_node(&self, id: &Id) -> bool {
        self.dragging.as_ref() == Some(id)
    }

    /// Returns whether the given node is the current hover target.
    pub fn is_hover_target(&self, id: &Id) -> bool {
        self.hover_target.as_ref() == Some(id)
    }

    /// Returns the current drop position, if any.
    pub fn current_drop_position(&self) -> Option<DropPosition> {
        self.drop_position
    }
}

/// Validates whether a drop operation is allowed.
///
/// This function checks if dropping `source` onto `target` at the given `position`
/// would create a valid hierarchy without circular dependencies.
///
/// # Arguments
///
/// * `source_id` - The ID of the node being dragged
/// * `target_id` - The ID of the potential drop target
/// * `position` - Where the source would be placed relative to the target
/// * `target_node` - The target node (used to check if it's a collection for Inside drops)
/// * `is_descendant` - A function that checks if the first ID is a descendant of the second
///
/// # Returns
///
/// `true` if the drop is valid, `false` otherwise.
pub fn validate_drop<N, F>(
    source_id: &N::Id,
    target_id: &N::Id,
    position: DropPosition,
    target_node: &N,
    is_descendant: F,
) -> bool
where
    N: OutlinerNode,
    F: Fn(&N::Id, &N::Id) -> bool,
{
    // Can't drop a node onto itself
    if source_id == target_id {
        return false;
    }

    // Can't drop a parent into its own descendant (would create a cycle)
    if is_descendant(target_id, source_id) {
        return false;
    }

    // For Inside drops, target must be a collection
    if position == DropPosition::Inside && !target_node.is_collection() {
        return false;
    }

    true
}

/// Determines the drop position based on the cursor position within a node's rect.
///
/// This function divides the node's vertical space into three zones:
/// - Top 25%: Before
/// - Middle 50%: Inside (if the node is a collection)
/// - Bottom 25%: After
///
/// # Arguments
///
/// * `cursor_y` - The Y coordinate of the cursor
/// * `rect` - The rectangle of the node being hovered over
/// * `is_collection` - Whether the target node can accept children
///
/// # Returns
///
/// The appropriate [`DropPosition`] based on cursor location.
pub fn calculate_drop_position(
    cursor_y: f32,
    rect: egui::Rect,
    is_collection: bool,
) -> DropPosition {
    let relative_y = (cursor_y - rect.top()) / rect.height();

    if relative_y < 0.25 {
        DropPosition::Before
    } else if relative_y > 0.75 {
        DropPosition::After
    } else if is_collection {
        DropPosition::Inside
    } else {
        // For non-collections in the middle zone, prefer After
        DropPosition::After
    }
}

/// Visual feedback configuration for drag-drop operations.
#[derive(Debug, Clone)]
pub struct DragDropVisuals {
    /// Color for the drop indicator line (Before/After positions).
    pub drop_line_color: egui::Color32,

    /// Thickness of the drop indicator line.
    pub drop_line_thickness: f32,

    /// Color for highlighting the drop target (Inside position).
    pub drop_target_color: egui::Color32,

    /// Color for the dragged node while dragging.
    pub drag_source_color: egui::Color32,

    /// Opacity multiplier for invalid drop targets.
    pub invalid_target_opacity: f32,
}

impl Default for DragDropVisuals {
    fn default() -> Self {
        Self {
            drop_line_color: egui::Color32::from_rgb(100, 150, 255),
            drop_line_thickness: 2.0,
            drop_target_color: egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
            drag_source_color: egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100),
            invalid_target_opacity: 0.3,
        }
    }
}

impl DragDropVisuals {
    /// Draws a drop indicator line at the specified position.
    ///
    /// # Arguments
    ///
    /// * `painter` - The egui painter to draw with
    /// * `rect` - The rectangle of the target node
    /// * `position` - Whether to draw the line before or after the node
    pub fn draw_drop_line(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        position: DropPosition,
    ) {
        let y = match position {
            DropPosition::Before => rect.top(),
            DropPosition::After => rect.bottom(),
            DropPosition::Inside => return, // Inside uses highlight instead
        };

        let start = egui::pos2(rect.left(), y);
        let end = egui::pos2(rect.right(), y);

        painter.line_segment(
            [start, end],
            egui::Stroke::new(self.drop_line_thickness, self.drop_line_color),
        );
    }

    /// Draws a highlight for an Inside drop target.
    ///
    /// # Arguments
    ///
    /// * `painter` - The egui painter to draw with
    /// * `rect` - The rectangle of the target node
    pub fn draw_drop_highlight(&self, painter: &egui::Painter, rect: egui::Rect) {
        painter.rect_filled(rect, 2.0, self.drop_target_color);
    }

    /// Draws visual feedback for the node being dragged.
    ///
    /// # Arguments
    ///
    /// * `painter` - The egui painter to draw with
    /// * `rect` - The rectangle of the dragged node
    pub fn draw_drag_source(&self, painter: &egui::Painter, rect: egui::Rect) {
        painter.rect_filled(rect, 2.0, self.drag_source_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{OutlinerNode, IconType, ActionIcon};

    // Mock node for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestNode {
        id: u64,
        name: String,
        is_collection: bool,
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
            self.is_collection
        }

        fn children(&self) -> &[Self] {
            &self.children
        }

        fn children_mut(&mut self) -> &mut Vec<Self> {
            &mut self.children
        }

        fn icon(&self) -> Option<IconType> {
            if self.is_collection {
                Some(IconType::Collection)
            } else {
                Some(IconType::Entity)
            }
        }

        fn action_icons(&self) -> Vec<ActionIcon> {
            vec![ActionIcon::Visibility, ActionIcon::Lock]
        }
    }

    impl TestNode {
        fn new(id: u64, name: &str, is_collection: bool) -> Self {
            Self {
                id,
                name: name.to_string(),
                is_collection,
                children: Vec::new(),
            }
        }

        fn with_children(mut self, children: Vec<TestNode>) -> Self {
            self.children = children;
            self
        }
    }

    #[test]
    fn test_drag_drop_state_default() {
        let state = DragDropState::<u64>::default();
        assert!(!state.is_dragging());
        assert_eq!(state.dragging_id(), None);
        assert_eq!(state.current_drop_position(), None);
    }

    #[test]
    fn test_drag_drop_state_new() {
        let state = DragDropState::<u64>::new();
        assert!(!state.is_dragging());
        assert_eq!(state.dragging_id(), None);
    }

    #[test]
    fn test_start_drag() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(42);
        
        assert!(state.is_dragging());
        assert_eq!(state.dragging_id(), Some(&42));
        assert!(state.is_dragging_node(&42));
        assert!(!state.is_dragging_node(&99));
    }

    #[test]
    fn test_update_hover() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        state.update_hover(2, DropPosition::Before);
        
        assert!(state.is_hover_target(&2));
        assert!(!state.is_hover_target(&1));
        assert_eq!(state.current_drop_position(), Some(DropPosition::Before));
    }

    #[test]
    fn test_clear_hover() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        state.update_hover(2, DropPosition::After);
        state.clear_hover();
        
        assert!(!state.is_hover_target(&2));
        assert_eq!(state.current_drop_position(), None);
        assert!(state.is_dragging()); // Drag should still be active
    }

    #[test]
    fn test_end_drag_with_valid_drop() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        state.update_hover(2, DropPosition::Inside);
        
        let result = state.end_drag();
        assert_eq!(result, Some((1, 2, DropPosition::Inside)));
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_end_drag_without_hover() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        
        let result = state.end_drag();
        assert_eq!(result, None);
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_cancel_drag() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        state.update_hover(2, DropPosition::Before);
        state.cancel_drag();
        
        assert!(!state.is_dragging());
        assert_eq!(state.dragging_id(), None);
        assert_eq!(state.current_drop_position(), None);
    }

    #[test]
    fn test_validate_drop_same_node() {
        let node = TestNode::new(1, "Node1", false);
        let is_descendant = |_: &u64, _: &u64| false;
        
        // Cannot drop a node onto itself
        assert!(!validate_drop::<TestNode, _>(
            &1,
            &1,
            DropPosition::Before,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_validate_drop_into_descendant() {
        let node = TestNode::new(2, "Node2", true);
        let is_descendant = |target: &u64, source: &u64| {
            // Simulate node 2 being a descendant of node 1
            *target == 2 && *source == 1
        };
        
        // Cannot drop a parent into its own descendant
        assert!(!validate_drop::<TestNode, _>(
            &1,
            &2,
            DropPosition::Inside,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_validate_drop_inside_non_collection() {
        let node = TestNode::new(2, "Node2", false);
        let is_descendant = |_: &u64, _: &u64| false;
        
        // Cannot drop inside a non-collection node
        assert!(!validate_drop::<TestNode, _>(
            &1,
            &2,
            DropPosition::Inside,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_validate_drop_valid_before() {
        let node = TestNode::new(2, "Node2", false);
        let is_descendant = |_: &u64, _: &u64| false;
        
        // Valid drop before a node
        assert!(validate_drop::<TestNode, _>(
            &1,
            &2,
            DropPosition::Before,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_validate_drop_valid_after() {
        let node = TestNode::new(2, "Node2", true);
        let is_descendant = |_: &u64, _: &u64| false;
        
        // Valid drop after a node
        assert!(validate_drop::<TestNode, _>(
            &1,
            &2,
            DropPosition::After,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_validate_drop_valid_inside_collection() {
        let node = TestNode::new(2, "Node2", true);
        let is_descendant = |_: &u64, _: &u64| false;
        
        // Valid drop inside a collection
        assert!(validate_drop::<TestNode, _>(
            &1,
            &2,
            DropPosition::Inside,
            &node,
            is_descendant
        ));
    }

    #[test]
    fn test_calculate_drop_position_before() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 40.0));
        
        // Top 25% should be Before
        let position = calculate_drop_position(5.0, rect, true);
        assert_eq!(position, DropPosition::Before);
    }

    #[test]
    fn test_calculate_drop_position_after() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 40.0));
        
        // Bottom 25% should be After
        let position = calculate_drop_position(35.0, rect, true);
        assert_eq!(position, DropPosition::After);
    }

    #[test]
    fn test_calculate_drop_position_inside_collection() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 40.0));
        
        // Middle 50% should be Inside for collections
        let position = calculate_drop_position(20.0, rect, true);
        assert_eq!(position, DropPosition::Inside);
    }

    #[test]
    fn test_calculate_drop_position_middle_non_collection() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 40.0));
        
        // Middle 50% should be After for non-collections
        let position = calculate_drop_position(20.0, rect, false);
        assert_eq!(position, DropPosition::After);
    }

    #[test]
    fn test_drag_drop_visuals_default() {
        let visuals = DragDropVisuals::default();
        assert_eq!(visuals.drop_line_thickness, 2.0);
        assert!(visuals.invalid_target_opacity > 0.0 && visuals.invalid_target_opacity < 1.0);
    }

    #[test]
    fn test_multiple_drag_operations() {
        let mut state = DragDropState::<u64>::new();
        
        // First drag
        state.start_drag(1);
        assert!(state.is_dragging_node(&1));
        state.end_drag();
        assert!(!state.is_dragging());
        
        // Second drag
        state.start_drag(2);
        assert!(state.is_dragging_node(&2));
        assert!(!state.is_dragging_node(&1));
        state.cancel_drag();
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_hover_updates_during_drag() {
        let mut state = DragDropState::<u64>::new();
        state.start_drag(1);
        
        // Update hover multiple times
        state.update_hover(2, DropPosition::Before);
        assert!(state.is_hover_target(&2));
        assert_eq!(state.current_drop_position(), Some(DropPosition::Before));
        
        state.update_hover(3, DropPosition::After);
        assert!(!state.is_hover_target(&2));
        assert!(state.is_hover_target(&3));
        assert_eq!(state.current_drop_position(), Some(DropPosition::After));
        
        state.update_hover(4, DropPosition::Inside);
        assert!(state.is_hover_target(&4));
        assert_eq!(state.current_drop_position(), Some(DropPosition::Inside));
    }
}