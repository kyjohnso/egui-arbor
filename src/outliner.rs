//! Main outliner widget implementation.
//!
//! This module provides the [`Outliner`] widget, which renders an interactive
//! hierarchical tree view with support for expansion, selection, editing, and
//! custom actions.

use crate::{
    response::OutlinerResponse,
    state::OutlinerState,
    style::Style,
    traits::{ActionIcon, OutlinerActions, OutlinerNode},
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
                    self.render_node(ui, node, 0, &mut state, actions, &mut outliner_response);
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
    fn render_node<N, A>(
        &self,
        ui: &mut egui::Ui,
        node: &N,
        depth: usize,
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

        // Start horizontal layout for this row
        ui.horizontal(|ui| {
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
        });

        // Render children if this is an expanded collection
        if is_collection && is_expanded {
            for child in node.children() {
                self.render_node(ui, child, depth + 1, state, actions, response);
            }
        }
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
    fn render_node_label<N, A>(
        &self,
        ui: &mut egui::Ui,
        node: &N,
        is_editing: bool,
        is_selected: bool,
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
            let (rect, label_response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width() - 100.0, self.style.row_height),
                egui::Sense::click(),
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
    fn render_action_icons<N, A>(&self, ui: &mut egui::Ui, node: &N, actions: &A)
    where
        N: OutlinerNode,
        A: OutlinerActions<N>,
    {
        let node_id = node.id();
        
        for action_icon in node.action_icons().iter().rev() {
            let (icon_text, is_active) = match action_icon {
                ActionIcon::Visibility => {
                    let is_visible = actions.is_visible(&node_id);
                    (if is_visible { "👁" } else { "🚫" }, is_visible)
                }
                ActionIcon::Lock => {
                    let is_locked = actions.is_locked(&node_id);
                    (if is_locked { "🔒" } else { "🔓" }, is_locked)
                }
                ActionIcon::Selection => {
                    let is_selected = actions.is_selected(&node_id);
                    (if is_selected { "☑" } else { "☐" }, is_selected)
                }
                ActionIcon::Custom { icon, tooltip } => {
                    let response = ui.label(icon.as_str());
                    if let Some(tip) = tooltip {
                        response.on_hover_text(tip);
                    }
                    continue;
                }
            };

            let (rect, icon_response) = ui.allocate_exact_size(
                egui::vec2(self.style.action_icon_size, self.style.row_height),
                egui::Sense::click(),
            );

            if ui.is_rect_visible(rect) {
                let visuals = ui.style().interact(&icon_response);
                let text_color = if is_active {
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

            // Note: Click handling for action icons would be implemented here
            // For now, we're just rendering them as visual indicators
        }
    }
}