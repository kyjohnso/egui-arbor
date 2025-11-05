//! Main outliner widget implementation.
//!
//! This module provides the [`Outliner`] widget, which renders an interactive
//! hierarchical tree view with support for expansion, selection, editing, and
//! custom actions.

use crate::{
    drag_drop::{calculate_drop_position, validate_drop, DragDropVisuals},
    response::{DropEvent, OutlinerResponse},
    state::OutlinerState,
    style::Style,
    traits::{ActionIcon, DropPosition, OutlinerActions, OutlinerNode},
};

/// The main outliner widget for rendering hierarchical tree structures.
///
/// This widget provides a complete tree view with support for:
/// - Hierarchical display with indentation
/// - Expand/collapse functionality for collection nodes
/// - Node selection and editing (rename)
/// - Action icons (visibility, lock, selection, custom)
/// - Keyboard navigation and shortcuts
///
/// # Examples
///
/// ```ignore
/// use egui_arbor::Outliner;
///
/// let response = Outliner::new("my_outliner")
///     .show(ui, &nodes, &mut actions);
///
/// if let Some(id) = response.selected() {
///     println!("Selected: {:?}", id);
/// }
/// ```
pub struct Outliner {
    /// Unique identifier for this outliner instance.
    id: egui::Id,
    
    /// Visual styling configuration.
    style: Style,

    /// Visual configuration for drag-drop operations.
    drag_drop_visuals: DragDropVisuals,
}

impl Outliner {
    /// Creates a new outliner widget with default styling.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for this outliner instance. This is used for
    ///   state persistence across frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::Outliner;
    ///
    /// let outliner = Outliner::new("my_outliner");
    /// ```
    pub fn new(id: impl Into<egui::Id>) -> Self {
        Self {
            id: id.into(),
            style: Style::default(),
            drag_drop_visuals: DragDropVisuals::default(),
        }
    }

    /// Sets a custom style for this outliner.
    ///
    /// # Arguments
    ///
    /// * `style` - The style configuration to use
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::{Outliner, Style};
    ///
    /// let outliner = Outliner::new("my_outliner")
    ///     .with_style(Style::default().with_indent(20.0));
    /// ```
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets custom drag-drop visuals for this outliner.
    ///
    /// # Arguments
    ///
    /// * `visuals` - The drag-drop visual configuration to use
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::{Outliner, DragDropVisuals};
    ///
    /// let outliner = Outliner::new("my_outliner")
    ///     .with_drag_drop_visuals(DragDropVisuals::default());
    /// ```
    pub fn with_drag_drop_visuals(mut self, visuals: DragDropVisuals) -> Self {
        self.drag_drop_visuals = visuals;
        self
    }

    /// Renders the outliner widget and returns the response.
    ///
    /// This is the main entry point for using the outliner. It renders all nodes
    /// in the hierarchy and handles user interactions.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The node type implementing [`OutlinerNode`]
    /// * `A` - The actions handler implementing [`OutlinerActions<N>`]
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI context to render into
    /// * `nodes` - The root-level nodes to display
    /// * `actions` - The actions handler for responding to user interactions
    ///
    /// # Returns
    ///
    /// An [`OutlinerResponse`] containing information about events that occurred
    /// during rendering.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = outliner.show(ui, &nodes, &mut actions);
    ///
    /// if let Some(id) = response.selected() {
    ///     println!("Node selected: {:?}", id);
    /// }
    /// ```
    pub fn show<N, A>(
        self,
        ui: &mut egui::Ui,
        nodes: &[N],
        actions: &mut A,
    ) -> OutlinerResponse<N::Id>
    where
        N: OutlinerNode,
        N::Id: 'static,
        A: OutlinerActions<N>,
    {
        // Load state from previous frame
        let mut state = OutlinerState::load(ui.ctx(), self.id);

        // Collect all visible node IDs in order for range selection
        let mut visible_nodes = Vec::new();
        Self::collect_visible_node_ids(nodes, &state, &mut visible_nodes);

        // Render within a scroll area and capture the inner response
        let scroll_output = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Track node rectangles for box selection
                let mut node_rects: Vec<(N::Id, egui::Rect)> = Vec::new();

                // Collect all currently selected nodes for potential multi-drag
                let selected_nodes: Vec<N::Id> = visible_nodes.iter()
                    .filter(|id| actions.is_selected(id))
                    .cloned()
                    .collect();

                // Create the outliner response wrapper
                let mut outliner_response = OutlinerResponse::new(
                    ui.allocate_response(egui::vec2(ui.available_width(), 0.0), egui::Sense::hover())
                );

                // Render all root nodes
                for node in nodes {
                    self.render_node(ui, node, 0, nodes, &mut state, actions, &mut outliner_response, &visible_nodes, &mut node_rects, &selected_nodes);
                }

                // Handle box selection in the background
                let available_rect = ui.available_rect_before_wrap();
                let bg_response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());

                // Check if we're starting a box selection (clicking in empty space)
                if bg_response.drag_started() {
                    if let Some(start_pos) = ui.ctx().pointer_interact_pos() {
                        // Only start box selection if not clicking on any node
                        let clicking_on_node = node_rects.iter().any(|(_, rect)| rect.contains(start_pos));
                        if !clicking_on_node {
                            state.start_box_selection(start_pos);
                        }
                    }
                }

                // Draw and update box selection
                if let Some(box_sel) = state.box_selection() {
                    if let Some(current_pos) = ui.ctx().pointer_interact_pos() {
                        // Draw selection box
                        let min_x = box_sel.start_pos.x.min(current_pos.x);
                        let max_x = box_sel.start_pos.x.max(current_pos.x);
                        let min_y = box_sel.start_pos.y.min(current_pos.y);
                        let max_y = box_sel.start_pos.y.max(current_pos.y);
                        let selection_rect = egui::Rect::from_min_max(
                            egui::pos2(min_x, min_y),
                            egui::pos2(max_x, max_y),
                        );

                        // Draw the selection box
                        ui.painter().rect_stroke(
                            selection_rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
                            egui::epaint::StrokeKind::Outside,
                        );
                        ui.painter().rect_filled(
                            selection_rect,
                            0.0,
                            egui::Color32::from_rgba_premultiplied(100, 150, 255, 30),
                        );

                        // Update selection based on box
                        if bg_response.dragged() {
                            let ctrl_or_cmd_pressed = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
                            
                            // If not holding ctrl/cmd, deselect all first
                            if !ctrl_or_cmd_pressed {
                                for id in &visible_nodes {
                                    if actions.is_selected(id) {
                                        actions.on_select(id, false);
                                    }
                                }
                            }

                            // Select nodes that intersect with the box
                            for (node_id, node_rect) in &node_rects {
                                if selection_rect.intersects(*node_rect) {
                                    actions.on_select(node_id, true);
                                }
                            }
                            outliner_response.changed = true;
                        }
                    }
                }

                if bg_response.drag_stopped() {
                    state.end_box_selection();
                }

                outliner_response
            });

        // Store state for next frame
        state.store(ui.ctx(), self.id);

        scroll_output.inner
    }

    /// Collects all visible node IDs in order (depth-first traversal).
    ///
    /// This is used for shift-click range selection.
    fn collect_visible_node_ids<N>(
        nodes: &[N],
        state: &OutlinerState<N::Id>,
        result: &mut Vec<N::Id>,
    ) where
        N: OutlinerNode,
    {
        for node in nodes {
            result.push(node.id());
            if node.is_collection() && state.is_expanded(&node.id()) {
                Self::collect_visible_node_ids(node.children(), state, result);
            }
        }
    }

    /// Renders a single node and its children recursively.
    ///
    /// This method handles the complete rendering of a node including:
    /// - Indentation based on depth
    /// - Expand/collapse arrow (for collections)
    /// - Node icon (if provided)
    /// - Node label (clickable, editable)
    /// - Action icons
    /// - Recursive rendering of children (if expanded)
    #[allow(clippy::too_many_arguments)]
    fn render_node<N, A>(
        &self,
        ui: &mut egui::Ui,
        node: &N,
        depth: usize,
        all_nodes: &[N],
        state: &mut OutlinerState<N::Id>,
        actions: &mut A,
        response: &mut OutlinerResponse<N::Id>,
        visible_nodes: &[N::Id],
        node_rects: &mut Vec<(N::Id, egui::Rect)>,
        selected_nodes: &[N::Id],
    ) where
        N: OutlinerNode,
        A: OutlinerActions<N>,
    {
        let node_id = node.id();
        let is_collection = node.is_collection();
        let is_expanded = state.is_expanded(&node_id);
        let is_editing = state.is_editing(&node_id);
        let is_selected = actions.is_selected(&node_id);

        // Check drag-drop state
        let is_dragging = state.drag_drop().is_dragging_node(&node_id);
        let is_hover_target = state.drag_drop().is_hover_target(&node_id);
        let drop_position = state.drag_drop().current_drop_position();

        // Start horizontal layout for this row
        let row_output = ui.horizontal(|ui| {
            // Calculate space needed for action icons upfront
            let num_action_icons = node.action_icons().len();
            let icons_width = num_action_icons as f32 * (self.style.action_icon_size + self.style.icon_spacing);
            
            // Add indentation
            ui.add_space(depth as f32 * self.style.indent);

            // Render expand/collapse arrow for collections
            if is_collection {
                let expand_response = self.render_expand_icon(ui, is_expanded);
                if expand_response.clicked() {
                    state.toggle_expanded(&node_id);
                    response.changed = true;
                }
            } else {
                // Add spacing to align with non-collection nodes
                ui.add_space(self.style.expand_icon_size + self.style.icon_spacing);
            }

            // Render node icon (placeholder for now)
            if node.icon().is_some() {
                ui.label("📄");
                ui.add_space(self.style.icon_spacing);
            }

            // Render node label (or text edit if editing)
            let label_response = self.render_node_label(
                ui,
                node,
                is_editing,
                is_selected,
                icons_width,
                state,
                actions,
                response,
            );

            // Handle label interactions
            if !is_editing {
                if label_response.clicked() {
                    // Check for modifier keys
                    let shift_pressed = ui.input(|i| i.modifiers.shift);
                    let ctrl_or_cmd_pressed = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);

                    if shift_pressed && state.last_selected().is_some() {
                        // Shift-click: select range
                        let last_id = state.last_selected().unwrap();
                        if let (Some(start_idx), Some(end_idx)) = (
                            visible_nodes.iter().position(|id| id == last_id),
                            visible_nodes.iter().position(|id| id == &node_id),
                        ) {
                            let (min_idx, max_idx) = if start_idx < end_idx {
                                (start_idx, end_idx)
                            } else {
                                (end_idx, start_idx)
                            };
                            
                            // Select all nodes in range
                            for id in &visible_nodes[min_idx..=max_idx] {
                                actions.on_select(id, true);
                            }
                        }
                        response.changed = true;
                    } else if ctrl_or_cmd_pressed {
                        // Ctrl/Cmd-click: toggle selection without clearing others
                        let new_selection = !is_selected;
                        actions.on_select(&node_id, new_selection);
                        if new_selection {
                            state.set_last_selected(Some(node_id.clone()));
                        }
                        response.selected = Some(node_id.clone());
                        response.changed = true;
                    } else {
                        // Normal click: clear other selections and select this one
                        // First, deselect all nodes
                        for id in visible_nodes {
                            if actions.is_selected(id) {
                                actions.on_select(id, false);
                            }
                        }
                        // Then select this node
                        actions.on_select(&node_id, true);
                        state.set_last_selected(Some(node_id.clone()));
                        response.selected = Some(node_id.clone());
                        response.changed = true;
                    }
                }

                if label_response.double_clicked() {
                    state.start_editing(node_id.clone(), node.name().to_string());
                    response.double_clicked = Some(node_id.clone());
                    response.changed = true;
                }

                if label_response.secondary_clicked() {
                    response.context_menu = Some(node_id.clone());
                }
            }

            // Render action icons (right-aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_action_icons(ui, node, actions);
            });

            // Return the label response so we can use it for drag detection
            label_response
        });

        let row_rect = row_output.response.rect;
        let label_response = row_output.inner;

        // Store the node rectangle for box selection
        node_rects.push((node_id.clone(), row_rect));

        // Use the label response for drag detection
        let drag_response = label_response;

        // Handle drag-drop interactions
        if !is_editing {
            // Detect drag start
            if drag_response.drag_started() {
                state.drag_drop_mut().start_drag(node_id.clone());
                response.drag_started = Some(node_id.clone());
                
                // Collect all selected nodes for multi-drag
                // If the dragged node is selected, include all selected nodes
                // Otherwise, just drag this single node
                let dragging_nodes = if actions.is_selected(&node_id) {
                    selected_nodes.to_vec()
                } else {
                    vec![node_id.clone()]
                };
                
                state.set_dragging_nodes(dragging_nodes.clone());
                response.dragging_nodes = dragging_nodes;
                response.changed = true;
            }

            // Handle hover for drop target detection
            if state.drag_drop().is_dragging() && !is_dragging {
                // Check if cursor is hovering over this row
                if let Some(cursor_pos) = ui.ctx().pointer_hover_pos()
                    && row_rect.contains(cursor_pos) {
                    let position = calculate_drop_position(
                        cursor_pos.y,
                        row_rect,
                        is_collection,
                    );

                    // Validate the drop
                    if let Some(source_id) = state.drag_drop().dragging_id() {
                        let is_valid = validate_drop(
                            source_id,
                            &node_id,
                            position,
                            node,
                            |target, source| Self::is_descendant_of_impl(all_nodes, target, source),
                        );

                        if is_valid {
                            state.drag_drop_mut().update_hover(node_id.clone(), position);
                        } else {
                            state.drag_drop_mut().clear_hover();
                        }
                    }
                }
            }

            // Handle drop
            if state.drag_drop().is_dragging() && drag_response.drag_stopped() {
                if let Some((source_id, target_id, position)) = state.drag_drop_mut().end_drag() {
                    // Invoke the on_move callback
                    actions.on_move(&source_id, &target_id, position);
                    
                    // Get the dragging nodes and add them to the response
                    response.dragging_nodes = state.dragging_nodes().to_vec();
                    
                    // Record the drop event in the response
                    response.drop_event = Some(DropEvent::new(source_id, target_id, position));
                    response.changed = true;
                    
                    // Clear dragging nodes after drop
                    state.clear_dragging_nodes();
                } else {
                    state.drag_drop_mut().cancel_drag();
                    state.clear_dragging_nodes();
                }
            }
        }

        // Draw visual feedback for drag-drop
        if is_dragging {
            self.drag_drop_visuals.draw_drag_source(ui.painter(), row_rect);
        }

        if is_hover_target
            && let Some(position) = drop_position {
                match position {
                    DropPosition::Before | DropPosition::After => {
                        self.drag_drop_visuals.draw_drop_line(ui.painter(), row_rect, position);
                    }
                    DropPosition::Inside => {
                        self.drag_drop_visuals.draw_drop_highlight(ui.painter(), row_rect);
                    }
                }
            }

        // Render children if this is an expanded collection
        if is_collection && is_expanded {
            for child in node.children() {
                self.render_node(ui, child, depth + 1, all_nodes, state, actions, response, visible_nodes, node_rects, selected_nodes);
            }
        }
    }

    /// Helper function to check if target_id is a descendant of source_id.
    ///
    /// This is used to prevent circular dependencies in drag-drop operations.
    fn is_descendant_of_impl<N>(all_nodes: &[N], target_id: &N::Id, source_id: &N::Id) -> bool
    where
        N: OutlinerNode,
    {
        // Find the source node
        if let Some(source_node) = Self::find_node_by_id_impl(all_nodes, source_id) {
            return Self::contains_descendant_impl(source_node, target_id);
        }
        false
    }

    /// Helper function to find a node by its ID.
    fn find_node_by_id_impl<'a, N>(nodes: &'a [N], id: &N::Id) -> Option<&'a N>
    where
        N: OutlinerNode,
    {
        for node in nodes {
            if node.id() == *id {
                return Some(node);
            }
            if let Some(found) = Self::find_node_by_id_impl(node.children(), id) {
                return Some(found);
            }
        }
        None
    }

    /// Helper function to check if a node contains a descendant with the given ID.
    fn contains_descendant_impl<N>(node: &N, target_id: &N::Id) -> bool
    where
        N: OutlinerNode,
    {
        for child in node.children() {
            if child.id() == *target_id {
                return true;
            }
            if Self::contains_descendant_impl(child, target_id) {
                return true;
            }
        }
        false
    }

    /// Renders the expand/collapse arrow icon.
    ///
    /// Returns the response from the arrow button/label.
    fn render_expand_icon(&self, ui: &mut egui::Ui, is_expanded: bool) -> egui::Response {
        let icon_text = if is_expanded {
            self.style.expand_icon_style.expanded_str()
        } else {
            self.style.expand_icon_style.collapsed_str()
        };

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(self.style.expand_icon_size, self.style.row_height),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let text_color = visuals.text_color();

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon_text,
                egui::FontId::proportional(self.style.expand_icon_size),
                text_color,
            );
        }

        response
    }

    /// Renders the node label, either as a selectable label or text edit.
    ///
    /// Returns the response from the label or text edit.
    #[allow(clippy::too_many_arguments)]
    fn render_node_label<N, A>(
        &self,
        ui: &mut egui::Ui,
        node: &N,
        is_editing: bool,
        is_selected: bool,
        icons_width: f32,
        state: &mut OutlinerState<N::Id>,
        actions: &mut A,
        response: &mut OutlinerResponse<N::Id>,
    ) -> egui::Response
    where
        N: OutlinerNode,
        A: OutlinerActions<N>,
    {
        if is_editing {
            // Render text edit for renaming
            let text_edit_response = ui.text_edit_singleline(state.editing_text_mut());

            // Check for Enter key to confirm
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let text = state.editing_text().to_string();
                actions.on_rename(&node.id(), text.clone());
                state.stop_editing();
                response.renamed = Some((node.id(), text));
                response.changed = true;
            }

            // Check for Escape key to cancel
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.stop_editing();
                response.changed = true;
            }

            // Auto-focus the text edit
            text_edit_response.request_focus();

            text_edit_response
        } else {
            // Render selectable label
            let label_text = node.name();
            
            // Create a custom selectable label with our styling
            // Include drag sensing so we can detect drag operations on the label
            // Reserve space for action icons to prevent layout shifts
            let available_width = ui.available_width();
            let label_width = (available_width - icons_width - 10.0).max(50.0);
            
            let (rect, label_response) = ui.allocate_exact_size(
                egui::vec2(label_width, self.style.row_height),
                egui::Sense::click_and_drag(),
            );

            if ui.is_rect_visible(rect) {
                let visuals = ui.style().interact(&label_response);
                
                // Draw background if selected or hovered
                if is_selected {
                    let bg_color = self.style.selection_color
                        .unwrap_or_else(|| ui.visuals().selection.bg_fill);
                    ui.painter().rect_filled(rect, 2.0, bg_color);
                } else if label_response.hovered() {
                    let bg_color = self.style.hover_color
                        .unwrap_or_else(|| ui.visuals().widgets.hovered.bg_fill);
                    ui.painter().rect_filled(rect, 2.0, bg_color);
                }

                // Draw text
                let text_color = if is_selected {
                    ui.visuals().selection.stroke.color
                } else {
                    visuals.text_color()
                };

                ui.painter().text(
                    rect.left_center() + egui::vec2(4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    label_text,
                    egui::FontId::proportional(self.style.row_height * 0.8),
                    text_color,
                );
            }

            label_response
        }
    }

    /// Collects all descendant node IDs recursively.
    ///
    /// This helper method traverses the tree starting from the given node
    /// and collects all descendant IDs into a vector.
    fn collect_descendant_ids<N>(node: &N) -> Vec<N::Id>
    where
        N: OutlinerNode,
    {
        let mut ids = Vec::new();
        for child in node.children() {
            ids.push(child.id());
            ids.extend(Self::collect_descendant_ids(child));
        }
        ids
    }

    /// Renders the action icons for a node.
    ///
    /// Icons are rendered right-to-left in the order they appear in the
    /// node's action_icons() list.
    fn render_action_icons<N, A>(&self, ui: &mut egui::Ui, node: &N, actions: &mut A)
    where
        N: OutlinerNode,
        A: OutlinerActions<N>,
    {
        let node_id = node.id();
        let is_collection = node.is_collection();
        
        for action_icon in node.action_icons().iter().rev() {
            match action_icon {
                ActionIcon::Visibility => {
                    let is_visible = actions.is_visible(&node_id);
                    let icon_text = if is_visible { "👁" } else { "🚫" };
                    
                    let (rect, icon_response) = ui.allocate_exact_size(
                        egui::vec2(self.style.action_icon_size, self.style.row_height),
                        egui::Sense::click(),
                    );

                    if ui.is_rect_visible(rect) {
                        let visuals = ui.style().interact(&icon_response);
                        let text_color = if is_visible {
                            visuals.text_color()
                        } else {
                            visuals.text_color().gamma_multiply(0.5)
                        };

                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon_text,
                            egui::FontId::proportional(self.style.action_icon_size * 0.8),
                            text_color,
                        );
                    }

                    // Handle click to toggle visibility
                    if icon_response.clicked() {
                        actions.on_visibility_toggle(&node_id);
                        // If this is a collection, apply to all children
                        if is_collection {
                            for child_id in Self::collect_descendant_ids(node) {
                                actions.on_visibility_toggle(&child_id);
                            }
                        }
                    }
                }
                ActionIcon::Lock => {
                    let is_locked = actions.is_locked(&node_id);
                    let icon_text = if is_locked { "🔒" } else { "🔓" };
                    
                    let (rect, icon_response) = ui.allocate_exact_size(
                        egui::vec2(self.style.action_icon_size, self.style.row_height),
                        egui::Sense::click(),
                    );

                    if ui.is_rect_visible(rect) {
                        let visuals = ui.style().interact(&icon_response);
                        let text_color = if is_locked {
                            visuals.text_color()
                        } else {
                            visuals.text_color().gamma_multiply(0.5)
                        };

                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon_text,
                            egui::FontId::proportional(self.style.action_icon_size * 0.8),
                            text_color,
                        );
                    }

                    // Handle click to toggle lock state
                    if icon_response.clicked() {
                        actions.on_lock_toggle(&node_id);
                        // If this is a collection, apply to all children
                        if is_collection {
                            for child_id in Self::collect_descendant_ids(node) {
                                actions.on_lock_toggle(&child_id);
                            }
                        }
                    }
                }
                ActionIcon::Selection => {
                    let is_selected = actions.is_selected(&node_id);
                    let icon_text = if is_selected { "☑" } else { "☐" };
                    
                    let (rect, icon_response) = ui.allocate_exact_size(
                        egui::vec2(self.style.action_icon_size, self.style.row_height),
                        egui::Sense::click(),
                    );

                    if ui.is_rect_visible(rect) {
                        let visuals = ui.style().interact(&icon_response);
                        let text_color = if is_selected {
                            visuals.text_color()
                        } else {
                            visuals.text_color().gamma_multiply(0.5)
                        };

                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon_text,
                            egui::FontId::proportional(self.style.action_icon_size * 0.8),
                            text_color,
                        );
                    }

                    // Handle click to toggle selection
                    if icon_response.clicked() {
                        // Determine the new selection state based on current state
                        let current_state = actions.is_selected(&node_id);
                        let new_state = !current_state;
                        
                        // Apply the new state to the parent
                        actions.on_select(&node_id, new_state);
                        
                        // If this is a collection, apply the same state to all children
                        if is_collection {
                            for child_id in Self::collect_descendant_ids(node) {
                                actions.on_select(&child_id, new_state);
                            }
                        }
                    }
                }
                ActionIcon::Custom { icon, tooltip } => {
                    let (rect, icon_response) = ui.allocate_exact_size(
                        egui::vec2(self.style.action_icon_size, self.style.row_height),
                        egui::Sense::click(),
                    );

                    if ui.is_rect_visible(rect) {
                        let visuals = ui.style().interact(&icon_response);
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon.as_str(),
                            egui::FontId::proportional(self.style.action_icon_size * 0.8),
                            visuals.text_color(),
                        );
                    }

                    // Handle click for custom action
                    let clicked = icon_response.clicked();
                    
                    // Add tooltip if provided (consumes the response)
                    let _icon_response = if let Some(tip) = tooltip {
                        icon_response.on_hover_text(tip)
                    } else {
                        icon_response
                    };

                    if clicked {
                        actions.on_custom_action(&node_id, icon.as_str());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{OutlinerNode, OutlinerActions, IconType, ActionIcon};
    use std::collections::{HashSet, HashMap};

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
            vec![ActionIcon::Visibility, ActionIcon::Lock, ActionIcon::Selection]
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

    // Mock actions handler for testing
    struct TestActions {
        selected: HashSet<u64>,
        visible: HashSet<u64>,
        locked: HashSet<u64>,
        renamed: HashMap<u64, String>,
        moved: Vec<(u64, u64, DropPosition)>,
        custom_actions: Vec<(u64, String)>,
    }

    impl TestActions {
        fn new() -> Self {
            Self {
                selected: HashSet::new(),
                visible: HashSet::new(),
                locked: HashSet::new(),
                renamed: HashMap::new(),
                moved: Vec::new(),
                custom_actions: Vec::new(),
            }
        }
    }

    impl OutlinerActions<TestNode> for TestActions {
        fn on_rename(&mut self, id: &u64, new_name: String) {
            self.renamed.insert(*id, new_name);
        }

        fn on_move(&mut self, id: &u64, target: &u64, position: DropPosition) {
            self.moved.push((*id, *target, position));
        }

        fn on_select(&mut self, id: &u64, selected: bool) {
            if selected {
                self.selected.insert(*id);
            } else {
                self.selected.remove(id);
            }
        }

        fn is_selected(&self, id: &u64) -> bool {
            self.selected.contains(id)
        }

        fn is_visible(&self, id: &u64) -> bool {
            self.visible.contains(id)
        }

        fn is_locked(&self, id: &u64) -> bool {
            self.locked.contains(id)
        }

        fn on_visibility_toggle(&mut self, id: &u64) {
            if self.visible.contains(id) {
                self.visible.remove(id);
            } else {
                self.visible.insert(*id);
            }
        }

        fn on_lock_toggle(&mut self, id: &u64) {
            if self.locked.contains(id) {
                self.locked.remove(id);
            } else {
                self.locked.insert(*id);
            }
        }

        fn on_selection_toggle(&mut self, id: &u64) {
            let is_selected = self.is_selected(id);
            self.on_select(id, !is_selected);
        }

        fn on_custom_action(&mut self, id: &u64, icon: &str) {
            self.custom_actions.push((*id, icon.to_string()));
        }
    }

    #[test]
    fn test_collect_visible_node_ids_flat() {
        let nodes = vec![
            TestNode::new(1, "Node1", false),
            TestNode::new(2, "Node2", false),
            TestNode::new(3, "Node3", false),
        ];
        
        let state = OutlinerState::<u64>::default();
        let mut result = Vec::new();
        
        Outliner::collect_visible_node_ids(&nodes, &state, &mut result);
        
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_collect_visible_node_ids_with_collapsed_children() {
        let nodes = vec![
            TestNode::new(1, "Node1", true).with_children(vec![
                TestNode::new(2, "Child1", false),
                TestNode::new(3, "Child2", false),
            ]),
        ];
        
        let state = OutlinerState::<u64>::default();
        let mut result = Vec::new();
        
        Outliner::collect_visible_node_ids(&nodes, &state, &mut result);
        
        // Only parent should be visible when collapsed
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_collect_visible_node_ids_with_expanded_children() {
        let nodes = vec![
            TestNode::new(1, "Node1", true).with_children(vec![
                TestNode::new(2, "Child1", false),
                TestNode::new(3, "Child2", false),
            ]),
        ];
        
        let mut state = OutlinerState::<u64>::default();
        state.set_expanded(&1, true);
        let mut result = Vec::new();
        
        Outliner::collect_visible_node_ids(&nodes, &state, &mut result);
        
        // Parent and children should be visible when expanded
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_collect_visible_node_ids_nested() {
        let nodes = vec![
            TestNode::new(1, "Node1", true).with_children(vec![
                TestNode::new(2, "Child1", true).with_children(vec![
                    TestNode::new(3, "GrandChild1", false),
                ]),
            ]),
        ];
        
        let mut state = OutlinerState::<u64>::default();
        state.set_expanded(&1, true);
        state.set_expanded(&2, true);
        let mut result = Vec::new();
        
        Outliner::collect_visible_node_ids(&nodes, &state, &mut result);
        
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_find_node_by_id_root_level() {
        let nodes = vec![
            TestNode::new(1, "Node1", false),
            TestNode::new(2, "Node2", false),
        ];
        
        let found = Outliner::find_node_by_id_impl(&nodes, &1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), 1);
        
        let found = Outliner::find_node_by_id_impl(&nodes, &2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), 2);
    }

    #[test]
    fn test_find_node_by_id_nested() {
        let nodes = vec![
            TestNode::new(1, "Node1", true).with_children(vec![
                TestNode::new(2, "Child1", false),
                TestNode::new(3, "Child2", true).with_children(vec![
                    TestNode::new(4, "GrandChild1", false),
                ]),
            ]),
        ];
        
        let found = Outliner::find_node_by_id_impl(&nodes, &4);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), 4);
    }

    #[test]
    fn test_find_node_by_id_not_found() {
        let nodes = vec![
            TestNode::new(1, "Node1", false),
        ];
        
        let found = Outliner::find_node_by_id_impl(&nodes, &999);
        assert!(found.is_none());
    }

    #[test]
    fn test_contains_descendant_direct_child() {
        let node = TestNode::new(1, "Parent", true).with_children(vec![
            TestNode::new(2, "Child", false),
        ]);
        
        assert!(Outliner::contains_descendant_impl(&node, &2));
        assert!(!Outliner::contains_descendant_impl(&node, &999));
    }

    #[test]
    fn test_contains_descendant_nested() {
        let node = TestNode::new(1, "Parent", true).with_children(vec![
            TestNode::new(2, "Child", true).with_children(vec![
                TestNode::new(3, "GrandChild", false),
            ]),
        ]);
        
        assert!(Outliner::contains_descendant_impl(&node, &2));
        assert!(Outliner::contains_descendant_impl(&node, &3));
        assert!(!Outliner::contains_descendant_impl(&node, &1));
    }

    #[test]
    fn test_is_descendant_of_impl() {
        let nodes = vec![
            TestNode::new(1, "Parent", true).with_children(vec![
                TestNode::new(2, "Child", true).with_children(vec![
                    TestNode::new(3, "GrandChild", false),
                ]),
            ]),
        ];
        
        // Node 2 is a descendant of node 1
        assert!(Outliner::is_descendant_of_impl(&nodes, &2, &1));
        
        // Node 3 is a descendant of node 1
        assert!(Outliner::is_descendant_of_impl(&nodes, &3, &1));
        
        // Node 3 is a descendant of node 2
        assert!(Outliner::is_descendant_of_impl(&nodes, &3, &2));
        
        // Node 1 is not a descendant of node 2
        assert!(!Outliner::is_descendant_of_impl(&nodes, &1, &2));
    }

    #[test]
    fn test_collect_descendant_ids() {
        let node = TestNode::new(1, "Parent", true).with_children(vec![
            TestNode::new(2, "Child1", false),
            TestNode::new(3, "Child2", true).with_children(vec![
                TestNode::new(4, "GrandChild", false),
            ]),
        ]);
        
        let ids = Outliner::collect_descendant_ids(&node);
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
        assert!(ids.contains(&4));
    }

    #[test]
    fn test_collect_descendant_ids_empty() {
        let node = TestNode::new(1, "Leaf", false);
        let ids = Outliner::collect_descendant_ids(&node);
        assert!(ids.is_empty());
    }

    #[test]
    fn test_outliner_new() {
        let outliner = Outliner::new("test_outliner");
        // Just verify it can be created
        assert_eq!(outliner.id, egui::Id::new("test_outliner"));
    }

    #[test]
    fn test_outliner_with_style() {
        let style = Style::default().with_indent(30.0);
        let outliner = Outliner::new("test").with_style(style.clone());
        assert_eq!(outliner.style.indent, 30.0);
    }

    #[test]
    fn test_outliner_with_drag_drop_visuals() {
        let visuals = DragDropVisuals::default();
        let outliner = Outliner::new("test").with_drag_drop_visuals(visuals);
        // Just verify it can be created with custom visuals
        assert_eq!(outliner.drag_drop_visuals.drop_line_thickness, 2.0);
    }
}