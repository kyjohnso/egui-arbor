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

        // Render within a scroll area and capture the inner response
        let scroll_output = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Create the outliner response wrapper
                let mut outliner_response = OutlinerResponse::new(
                    ui.allocate_response(egui::vec2(ui.available_width(), 0.0), egui::Sense::hover())
                );

                // Render all root nodes
                for node in nodes {
                    self.render_node(ui, node, 0, nodes, &mut state, actions, &mut outliner_response);
                }

                outliner_response
            });

        // Store state for next frame
        state.store(ui.ctx(), self.id);

        scroll_output.inner
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
                    let new_selection = !is_selected;
                    actions.on_select(&node_id, new_selection);
                    response.selected = Some(node_id.clone());
                    response.changed = true;
                }

                if label_response.double_clicked() {
                    state.start_editing(node_id.clone());
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

        // Use the label response for drag detection
        let drag_response = label_response;

        // Handle drag-drop interactions
        if !is_editing {
            // Detect drag start
            if drag_response.drag_started() {
                state.drag_drop_mut().start_drag(node_id.clone());
                response.drag_started = Some(node_id.clone());
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
                    
                    // Record the drop event in the response
                    response.drop_event = Some(DropEvent::new(source_id, target_id, position));
                    response.changed = true;
                } else {
                    state.drag_drop_mut().cancel_drag();
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
                self.render_node(ui, child, depth + 1, all_nodes, state, actions, response);
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
            let mut text = node.name().to_string();
            let text_edit_response = ui.text_edit_singleline(&mut text);

            // Check for Enter key to confirm
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
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
                        actions.on_selection_toggle(&node_id);
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