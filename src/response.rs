//! Response types for the outliner widget.
//!
//! This module provides types that represent the result of rendering an outliner widget,
//! including information about user interactions and state changes.

use crate::traits::DropPosition;
use std::hash::Hash;
use std::ops::Deref;

/// The response from rendering an outliner widget.
///
/// This type wraps an [`egui::Response`] and provides additional information about
/// outliner-specific events that occurred during the frame, such as node selection,
/// double-clicks, context menu requests, renaming, and drag-drop operations.
///
/// # Generic Parameters
///
/// * `Id` - The type used to identify nodes in the outliner. Must implement
///   [`Hash`], [`Eq`], and [`Clone`].
///
/// # Examples
///
/// ```ignore
/// let response = outliner.show(ui, &mut state);
///
/// if let Some(id) = response.selected() {
///     println!("Node selected: {:?}", id);
/// }
///
/// if let Some((id, new_name)) = response.renamed() {
///     println!("Node {} renamed to: {}", id, new_name);
/// }
///
/// if let Some(drop_event) = response.drop_event() {
///     println!("Dropped {:?} onto {:?}", drop_event.source, drop_event.target);
/// }
/// ```
#[derive(Debug)]
pub struct OutlinerResponse<Id>
where
    Id: Hash + Eq + Clone,
{
    /// The underlying egui widget response.
    ///
    /// This can be accessed directly via [`Deref`] to check standard widget properties
    /// like hover state, clicks, etc.
    pub inner: egui::Response,

    /// Whether any outliner state changed this frame.
    ///
    /// This includes selection changes, expansion/collapse, renaming, etc.
    /// Useful for determining if you need to save state or trigger updates.
    pub changed: bool,

    /// ID of the node that was newly selected this frame, if any.
    ///
    /// This is `Some` only when the selection changes, not on every frame
    /// where a node is selected.
    pub selected: Option<Id>,

    /// ID of the node that was double-clicked this frame, if any.
    ///
    /// Double-clicking typically triggers an action like opening or editing a node.
    pub double_clicked: Option<Id>,

    /// ID of the node for which a context menu was requested this frame, if any.
    ///
    /// This is typically triggered by right-clicking on a node.
    pub context_menu: Option<Id>,

    /// ID and new name of a node that was renamed this frame, if any.
    ///
    /// The tuple contains `(node_id, new_name)`.
    pub renamed: Option<(Id, String)>,

    /// ID of the node where a drag operation started this frame, if any.
    ///
    /// This indicates the user began dragging a node.
    pub drag_started: Option<Id>,

    /// IDs of all nodes being dragged (includes the primary drag node and any selected nodes).
    ///
    /// When dragging with multiple selections, this contains all selected node IDs.
    pub dragging_nodes: Vec<Id>,

    /// Details of a drop event that occurred this frame, if any.
    ///
    /// This contains information about the source node, target node, and drop position.
    pub drop_event: Option<DropEvent<Id>>,
}

impl<Id> OutlinerResponse<Id>
where
    Id: Hash + Eq + Clone,
{
    /// Creates a new outliner response with no events.
    ///
    /// All event fields are initialized to `None` and `changed` is set to `false`.
    /// The widget implementation will populate these fields as events occur.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying egui response from the widget
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = OutlinerResponse::new(ui.allocate_response(size, Sense::click()));
    /// ```
    pub fn new(inner: egui::Response) -> Self {
        Self {
            inner,
            changed: false,
            selected: None,
            double_clicked: None,
            context_menu: None,
            renamed: None,
            drag_started: None,
            dragging_nodes: Vec::new(),
            drop_event: None,
        }
    }

    /// Returns whether any outliner state changed this frame.
    ///
    /// This includes selection changes, expansion/collapse, renaming, etc.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if response.changed() {
    ///     save_state(&state);
    /// }
    /// ```
    #[inline]
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Returns the ID of the node that was newly selected this frame, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(id) = response.selected() {
    ///     println!("Selected node: {:?}", id);
    /// }
    /// ```
    #[inline]
    pub fn selected(&self) -> Option<&Id> {
        self.selected.as_ref()
    }

    /// Returns the ID of the node that was double-clicked this frame, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(id) = response.double_clicked() {
    ///     open_node(id);
    /// }
    /// ```
    #[inline]
    pub fn double_clicked(&self) -> Option<&Id> {
        self.double_clicked.as_ref()
    }

    /// Returns the ID of the node for which a context menu was requested, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(id) = response.context_menu() {
    ///     show_context_menu(ui, id);
    /// }
    /// ```
    #[inline]
    pub fn context_menu(&self) -> Option<&Id> {
        self.context_menu.as_ref()
    }

    /// Returns the ID and new name of a node that was renamed this frame, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some((id, new_name)) = response.renamed() {
    ///     update_node_name(id, new_name);
    /// }
    /// ```
    #[inline]
    pub fn renamed(&self) -> Option<(&Id, &str)> {
        self.renamed.as_ref().map(|(id, name)| (id, name.as_str()))
    }

    /// Returns the ID of the node where a drag operation started, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(id) = response.drag_started() {
    ///     begin_drag_operation(id);
    /// }
    /// ```
    #[inline]
    pub fn drag_started(&self) -> Option<&Id> {
        self.drag_started.as_ref()
    }

    /// Returns the IDs of all nodes being dragged (primary + selected nodes).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if !response.dragging_nodes().is_empty() {
    ///     for id in response.dragging_nodes() {
    ///         highlight_dragging_node(id);
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn dragging_nodes(&self) -> &[Id] {
        &self.dragging_nodes
    }

    /// Returns details of a drop event that occurred this frame, if any.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(drop_event) = response.drop_event() {
    ///     move_node(drop_event.source, drop_event.target, drop_event.position);
    /// }
    /// ```
    #[inline]
    pub fn drop_event(&self) -> Option<&DropEvent<Id>> {
        self.drop_event.as_ref()
    }
}

impl<Id> Deref for OutlinerResponse<Id>
where
    Id: Hash + Eq + Clone,
{
    type Target = egui::Response;

    /// Dereferences to the underlying [`egui::Response`].
    ///
    /// This allows convenient access to standard response methods like
    /// `hovered()`, `clicked()`, `rect`, etc.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = outliner.show(ui, &mut state);
    ///
    /// // Access egui::Response methods directly
    /// if response.hovered() {
    ///     ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    /// }
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Details of a drag-and-drop event in the outliner.
///
/// This struct contains information about a completed drop operation,
/// including the source node that was dragged, the target node it was
/// dropped onto, and the position relative to the target.
///
/// # Generic Parameters
///
/// * `Id` - The type used to identify nodes in the outliner. Must implement
///   [`Hash`], [`Eq`], and [`Clone`].
///
/// # Examples
///
/// ```ignore
/// if let Some(drop_event) = response.drop_event() {
///     match drop_event.position {
///         DropPosition::Before => {
///             insert_before(drop_event.source, drop_event.target);
///         }
///         DropPosition::After => {
///             insert_after(drop_event.source, drop_event.target);
///         }
///         DropPosition::Inside => {
///             make_child_of(drop_event.source, drop_event.target);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropEvent<Id>
where
    Id: Hash + Eq + Clone,
{
    /// The ID of the node that was dragged.
    pub source: Id,

    /// The ID of the node that the source was dropped onto.
    pub target: Id,

    /// The position where the source should be placed relative to the target.
    pub position: DropPosition,
}

impl<Id> DropEvent<Id>
where
    Id: Hash + Eq + Clone,
{
    /// Creates a new drop event.
    ///
    /// # Arguments
    ///
    /// * `source` - The ID of the node that was dragged
    /// * `target` - The ID of the node that was dropped onto
    /// * `position` - Where to place the source relative to the target
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let drop_event = DropEvent::new(
    ///     dragged_id,
    ///     target_id,
    ///     DropPosition::Inside,
    /// );
    /// ```
    pub fn new(source: Id, target: Id, position: DropPosition) -> Self {
        Self {
            source,
            target,
            position,
        }
    }
}