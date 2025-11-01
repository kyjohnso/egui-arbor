//! Comprehensive example demonstrating all egui-arbor features.
//!
//! This example showcases:
//! - **Tree Structure**: Creating hierarchical data with collections and entities
//! - **OutlinerNode Trait**: Implementing the core trait for custom node types
//! - **OutlinerActions Trait**: Handling user interactions and state management
//! - **Action Icons**: Visibility, lock, and selection toggles with visual feedback
//! - **Drag & Drop**: Moving nodes with Before/After/Inside positioning
//! - **Node Selection**: Single-selection with visual highlighting
//! - **Rename Functionality**: Double-click to edit node names inline
//! - **Expand/Collapse**: Navigate through the tree hierarchy
//! - **Event Logging**: Track all user interactions in real-time
//!
//! ## Key Features Demonstrated:
//!
//! ### Action Icons
//! - **Visibility (👁)**: Toggle node visibility state
//! - **Lock (🔒)**: Prevent node modifications
//! - **Selection (☑)**: Quick selection toggle
//!
//! ### Drag & Drop Operations
//! - **Before**: Insert dragged node before the target
//! - **After**: Insert dragged node after the target
//! - **Inside**: Add dragged node as a child of the target (collections only)
//!
//! ### Interactive Elements
//! - Click to select nodes
//! - Double-click to rename
//! - Drag nodes to reorder or reparent
//! - Click action icons for state changes
//! - Expand/collapse collections with arrow icons
//!
//! To run this example:
//! ```
//! cargo run --example basic
//! ```

use egui_arbor::{
    ActionIcon, DropPosition, IconType, Outliner, OutlinerActions, OutlinerNode,
};
use std::collections::{HashSet, VecDeque};
use std::time::SystemTime;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("egui-arbor Basic Example"),
        ..Default::default()
    };

    eframe::run_native(
        "egui-arbor Example",
        options,
        Box::new(|cc| {
            // Load fonts with better Unicode support for triangle characters
            let mut fonts = egui::FontDefinitions::default();
            
            // Try to load system fonts with good Unicode coverage
            let font_loaded = load_unicode_font(&mut fonts);
            
            if font_loaded {
                cc.egui_ctx.set_fonts(fonts);
            }
            
            Ok(Box::new(ExampleApp::new()))
        }),
    )
}

/// Attempts to load a system font with good Unicode coverage.
/// Returns true if a font was successfully loaded.
fn load_unicode_font(fonts: &mut egui::FontDefinitions) -> bool {
    // Try different font paths based on the operating system
    let font_paths = [
        // macOS
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        // Windows
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        // Linux (common paths)
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ];
    
    for font_path in &font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "unicode_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "unicode_font".to_owned());
            return true;
        }
    }
    
    false
}

/// A tree node representing files and folders in a project structure.
///
/// This demonstrates how to create a custom node type that works with egui-arbor.
/// The node stores:
/// - A unique identifier for tracking and operations
/// - A display name that can be edited
/// - Whether it's a collection (folder) or entity (file)
/// - Child nodes for hierarchical structure
#[derive(Clone, Debug)]
struct TreeNode {
    id: u64,
    name: String,
    is_collection: bool,
    children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a new collection node (folder)
    fn collection(id: u64, name: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            name: name.into(),
            is_collection: true,
            children,
        }
    }

    /// Create a new entity node (file)
    fn entity(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            is_collection: false,
            children: Vec::new(),
        }
    }

    /// Find a node by ID and update its name
    fn rename_node(&mut self, id: u64, new_name: String) -> bool {
        if self.id == id {
            self.name = new_name;
            return true;
        }

        for child in &mut self.children {
            if child.rename_node(id, new_name.clone()) {
                return true;
            }
        }

        false
    }

    /// Remove a node by ID and return it if found
    fn remove_node(&mut self, id: u64) -> Option<TreeNode> {
        for i in 0..self.children.len() {
            if self.children[i].id == id {
                return Some(self.children.remove(i));
            }
            if let Some(node) = self.children[i].remove_node(id) {
                return Some(node);
            }
        }
        None
    }

    /// Insert a node at a specific position relative to a target node
    fn insert_node(&mut self, target_id: u64, node: TreeNode, position: DropPosition) -> bool {
        // Check if this is the target node
        if self.id == target_id {
            match position {
                DropPosition::Inside => {
                    if self.is_collection {
                        self.children.push(node);
                        return true;
                    }
                }
                _ => {
                    // Can't insert before/after at root level
                    return false;
                }
            }
        }

        // Search in children
        for i in 0..self.children.len() {
            if self.children[i].id == target_id {
                match position {
                    DropPosition::Before => {
                        self.children.insert(i, node);
                        return true;
                    }
                    DropPosition::After => {
                        self.children.insert(i + 1, node);
                        return true;
                    }
                    DropPosition::Inside => {
                        if self.children[i].is_collection {
                            self.children[i].children.push(node);
                            return true;
                        }
                    }
                }
            }
            if self.children[i].insert_node(target_id, node.clone(), position) {
                return true;
            }
        }

        false
    }
}

/// Implementation of OutlinerNode trait for TreeNode.
///
/// This trait defines how nodes are displayed and interacted with in the outliner.
/// Each method serves a specific purpose:
///
/// - `id()`: Returns unique identifier for tracking and operations
/// - `name()`: Provides the display text for the node
/// - `is_collection()`: Determines if node can contain children
/// - `children()`: Provides read access to child nodes
/// - `children_mut()`: Provides write access for modifications
/// - `icon()`: Specifies the icon to display (folder vs file)
/// - `action_icons()`: Defines which action buttons appear for this node
impl OutlinerNode for TreeNode {
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

    /// Returns the appropriate icon based on node type.
    /// Collections (folders) get a folder icon, entities (files) get a file icon.
    fn icon(&self) -> Option<IconType> {
        if self.is_collection {
            Some(IconType::Collection)
        } else {
            Some(IconType::Entity)
        }
    }

    /// Defines which action icons are available for this node.
    /// All nodes in this example support visibility, lock, and selection toggles.
    fn action_icons(&self) -> Vec<ActionIcon> {
        vec![
            ActionIcon::Visibility,  // Toggle visibility state
            ActionIcon::Lock,        // Toggle lock state
            ActionIcon::Selection,   // Quick selection toggle
        ]
    }
}

/// Event log entry for tracking user interactions.
#[derive(Clone, Debug)]
struct LogEntry {
    timestamp: SystemTime,
    message: String,
    event_type: EventType,
}

#[derive(Clone, Debug, PartialEq)]
enum EventType {
    Selection,
    Visibility,
    Lock,
    DragDrop,
    Rename,
}

impl LogEntry {
    fn new(message: String, event_type: EventType) -> Self {
        Self {
            timestamp: SystemTime::now(),
            message,
            event_type,
        }
    }
}

/// Actions handler that manages node state and tracks user interactions.
///
/// This struct demonstrates how to implement the OutlinerActions trait to:
/// - Track which nodes are currently selected (supports multi-selection)
/// - Maintain visibility state for each node
/// - Maintain lock state for each node
/// - Log all user interactions for debugging and demonstration
///
/// The actions handler is the bridge between user interactions and your application state.
struct TreeActions {
    selected: HashSet<u64>,
    visible: HashSet<u64>,
    locked: HashSet<u64>,
    event_log: VecDeque<LogEntry>,
    max_log_entries: usize,
}

impl TreeActions {
    fn new() -> Self {
        // Initialize with all nodes visible by default
        let mut visible = HashSet::new();
        for id in 0..46 {
            visible.insert(id);
        }

        Self {
            selected: HashSet::new(),
            visible,
            locked: HashSet::new(),
            event_log: VecDeque::new(),
            max_log_entries: 10,
        }
    }

    /// Add an entry to the event log, maintaining the maximum size.
    fn log_event(&mut self, message: String, event_type: EventType) {
        self.event_log.push_front(LogEntry::new(message, event_type));
        if self.event_log.len() > self.max_log_entries {
            self.event_log.pop_back();
        }
    }

    /// Get statistics about current node states.
    fn get_stats(&self) -> NodeStats {
        NodeStats {
            total_nodes: 46,
            visible_count: self.visible.len(),
            hidden_count: 46 - self.visible.len(),
            locked_count: self.locked.len(),
            selected_count: self.selected.len(),
        }
    }
}

#[derive(Debug)]
struct NodeStats {
    total_nodes: usize,
    visible_count: usize,
    hidden_count: usize,
    locked_count: usize,
    selected_count: usize,
}

/// Implementation of OutlinerActions trait for TreeActions.
///
/// This trait handles all user interactions with the outliner:
/// - Selection changes
/// - Visibility toggles
/// - Lock toggles
/// - Rename operations
/// - Drag & drop moves
/// - Custom actions
///
/// Each callback is invoked when the corresponding user action occurs,
/// allowing you to update your application state accordingly.
impl OutlinerActions<TreeNode> for TreeActions {
    /// Called when a node is renamed (after double-click edit).
    /// The actual tree modification happens in the app's update method.
    fn on_rename(&mut self, id: &u64, new_name: String) {
        self.log_event(
            format!("Renamed node {} to '{}'", id, new_name),
            EventType::Rename,
        );
    }

    /// Called when a drag & drop operation is initiated.
    /// The actual tree modification happens in the app's update method.
    fn on_move(&mut self, id: &u64, target: &u64, position: DropPosition) {
        self.log_event(
            format!("Move: node {} → target {} ({:?})", id, target, position),
            EventType::DragDrop,
        );
    }

    /// Called when a node's selection state changes.
    /// This example implements multi-selection behavior.
    fn on_select(&mut self, id: &u64, selected: bool) {
        if selected {
            self.selected.insert(*id);
            self.log_event(
                format!("Selected node {}", id),
                EventType::Selection,
            );
        } else {
            self.selected.remove(id);
            self.log_event(
                format!("Deselected node {}", id),
                EventType::Selection,
            );
        }
    }

    /// Query whether a node is currently selected.
    fn is_selected(&self, id: &u64) -> bool {
        self.selected.contains(id)
    }

    /// Query whether a node is currently visible.
    /// Hidden nodes are visually indicated in the UI.
    fn is_visible(&self, id: &u64) -> bool {
        self.visible.contains(id)
    }

    /// Query whether a node is currently locked.
    /// Locked nodes cannot be modified or moved.
    fn is_locked(&self, id: &u64) -> bool {
        self.locked.contains(id)
    }

    /// Called when the visibility action icon is clicked.
    /// Toggles the node between visible and hidden states.
    fn on_visibility_toggle(&mut self, id: &u64) {
        let was_visible = self.visible.contains(id);
        if was_visible {
            self.visible.remove(id);
            self.log_event(
                format!("Hidden node {}", id),
                EventType::Visibility,
            );
        } else {
            self.visible.insert(*id);
            self.log_event(
                format!("Shown node {}", id),
                EventType::Visibility,
            );
        }
    }

    /// Called when the lock action icon is clicked.
    /// Toggles the node between locked and unlocked states.
    fn on_lock_toggle(&mut self, id: &u64) {
        let was_locked = self.locked.contains(id);
        if was_locked {
            self.locked.remove(id);
            self.log_event(
                format!("Unlocked node {}", id),
                EventType::Lock,
            );
        } else {
            self.locked.insert(*id);
            self.log_event(
                format!("Locked node {}", id),
                EventType::Lock,
            );
        }
    }

    /// Called when the selection action icon is clicked.
    /// Provides a quick way to toggle selection without clicking the node itself.
    fn on_selection_toggle(&mut self, id: &u64) {
        let is_selected = self.is_selected(id);
        self.on_select(id, !is_selected);
    }

    /// Called for custom action icons (not used in this example).
    /// You can extend this to add your own custom actions.
    fn on_custom_action(&mut self, _id: &u64, _icon: &str) {
        // Custom actions not used in this example
    }
}

/// The main application demonstrating egui-arbor features.
///
/// This app maintains:
/// - The tree structure (nodes and hierarchy)
/// - The actions handler (state and event tracking)
/// - UI state for panels and display options
struct ExampleApp {
    tree: Vec<TreeNode>,
    actions: TreeActions,
    show_help: bool,
    show_stats: bool,
    show_log: bool,
}

impl ExampleApp {
    fn new() -> Self {
        // Create a sample tree structure representing a project
        let tree = vec![
            TreeNode::collection(
                0,
                "Project",
                vec![
                    TreeNode::collection(
                        1,
                        "src",
                        vec![
                            TreeNode::entity(2, "main.rs"),
                            TreeNode::entity(3, "lib.rs"),
                            TreeNode::collection(
                                4,
                                "components",
                                vec![
                                    TreeNode::entity(5, "button.rs"),
                                    TreeNode::entity(6, "input.rs"),
                                    TreeNode::entity(7, "layout.rs"),
                                    TreeNode::entity(8, "modal.rs"),
                                    TreeNode::entity(9, "dropdown.rs"),
                                ],
                            ),
                            TreeNode::collection(
                                10,
                                "utils",
                                vec![
                                    TreeNode::entity(11, "helpers.rs"),
                                    TreeNode::entity(12, "validators.rs"),
                                    TreeNode::entity(13, "formatters.rs"),
                                ],
                            ),
                        ],
                    ),
                    TreeNode::collection(
                        14,
                        "examples",
                        vec![
                            TreeNode::entity(15, "basic.rs"),
                            TreeNode::entity(16, "advanced.rs"),
                            TreeNode::entity(17, "custom_styling.rs"),
                        ],
                    ),
                    TreeNode::collection(
                        18,
                        "tests",
                        vec![
                            TreeNode::entity(19, "integration_test.rs"),
                            TreeNode::entity(20, "unit_test.rs"),
                            TreeNode::entity(21, "ui_test.rs"),
                        ],
                    ),
                    TreeNode::collection(
                        22,
                        "assets",
                        vec![
                            TreeNode::collection(
                                23,
                                "images",
                                vec![
                                    TreeNode::entity(24, "logo.png"),
                                    TreeNode::entity(25, "icon.svg"),
                                ],
                            ),
                            TreeNode::collection(
                                26,
                                "fonts",
                                vec![
                                    TreeNode::entity(27, "roboto.ttf"),
                                    TreeNode::entity(28, "monospace.ttf"),
                                ],
                            ),
                        ],
                    ),
                    TreeNode::entity(29, "Cargo.toml"),
                    TreeNode::entity(30, "README.md"),
                    TreeNode::entity(31, ".gitignore"),
                    TreeNode::entity(32, "LICENSE"),
                ],
            ),
            TreeNode::collection(
                33,
                "Documentation",
                vec![
                    TreeNode::entity(34, "getting_started.md"),
                    TreeNode::entity(35, "api_reference.md"),
                    TreeNode::entity(36, "examples.md"),
                    TreeNode::entity(37, "contributing.md"),
                    TreeNode::collection(
                        38,
                        "guides",
                        vec![
                            TreeNode::entity(39, "installation.md"),
                            TreeNode::entity(40, "configuration.md"),
                            TreeNode::entity(41, "troubleshooting.md"),
                        ],
                    ),
                ],
            ),
            TreeNode::collection(
                42,
                "Scripts",
                vec![
                    TreeNode::entity(43, "build.sh"),
                    TreeNode::entity(44, "test.sh"),
                    TreeNode::entity(45, "deploy.sh"),
                ],
            ),
        ];

        Self {
            tree,
            actions: TreeActions::new(),
            show_help: true,
            show_stats: true,
            show_log: true,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with title and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌳 egui-arbor Comprehensive Example");
                ui.separator();
                
                // Toggle buttons for panels
                ui.checkbox(&mut self.show_help, "📖 Help");
                ui.checkbox(&mut self.show_stats, "📊 Stats");
                ui.checkbox(&mut self.show_log, "📋 Event Log");
            });
        });

        // Left panel with help and instructions
        if self.show_help {
            egui::SidePanel::left("help_panel")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("📖 Instructions");
                    ui.separator();
                    
                    ui.label(egui::RichText::new("Basic Interactions:").strong());
                    ui.label("• Click to select nodes");
                    ui.label("• Double-click to rename");
                    ui.label("• Click ▶/▼ to expand/collapse");
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Action Icons:").strong());
                    ui.label("• 👁 Toggle visibility");
                    ui.label("• 🔒 Toggle lock state");
                    ui.label("• ☑ Toggle selection");
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Drag & Drop:").strong());
                    ui.label("• Drag nodes to reorder");
                    ui.label("• Drop Before target");
                    ui.label("• Drop After target");
                    ui.label("• Drop Inside collections");
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Visual Indicators:").strong());
                    ui.label("• 🔵 Selected node");
                    ui.label("• 👁‍🗨 Hidden node (dimmed)");
                    ui.label("• 🔒 Locked node");
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Tips:").strong());
                    ui.label("• Try hiding a folder");
                    ui.label("• Lock a node, then try to drag it");
                    ui.label("• Drag files into folders");
                    ui.label("• Reorder items with Before/After");
                });
        }

        // Right panel with stats and event log
        if self.show_stats || self.show_log {
            egui::SidePanel::right("info_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    if self.show_stats {
                        ui.heading("📊 Node Statistics");
                        ui.separator();
                        
                        let stats = self.actions.get_stats();
                        
                        egui::Grid::new("stats_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Total Nodes:");
                                ui.label(stats.total_nodes.to_string());
                                ui.end_row();
                                
                                ui.label("Visible:");
                                ui.label(format!("{} 👁", stats.visible_count));
                                ui.end_row();
                                
                                ui.label("Hidden:");
                                ui.label(format!("{} 👁‍🗨", stats.hidden_count));
                                ui.end_row();
                                
                                ui.label("Locked:");
                                ui.label(format!("{} 🔒", stats.locked_count));
                                ui.end_row();
                                
                                ui.label("Selected:");
                                ui.label(format!("{} 🔵", stats.selected_count));
                                ui.end_row();
                            });
                        
                        ui.add_space(8.0);
                        
                        if !self.actions.selected.is_empty() {
                            ui.label(egui::RichText::new("Selected Node IDs:")
                                .color(egui::Color32::from_rgb(100, 150, 255)));
                            let mut selected_ids: Vec<_> = self.actions.selected.iter().collect();
                            selected_ids.sort();
                            for id in selected_ids.iter().take(5) {
                                ui.label(format!("  • {}", id));
                            }
                            if selected_ids.len() > 5 {
                                ui.label(format!("  ... and {} more", selected_ids.len() - 5));
                            }
                        } else {
                            ui.label(egui::RichText::new("No nodes selected")
                                .color(egui::Color32::GRAY));
                        }
                        
                        ui.separator();
                    }
                    
                    if self.show_log {
                        ui.heading("📋 Event Log");
                        ui.separator();
                        
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                if self.actions.event_log.is_empty() {
                                    ui.label(egui::RichText::new("No events yet...")
                                        .italics()
                                        .color(egui::Color32::GRAY));
                                } else {
                                    for entry in &self.actions.event_log {
                                        let color = match entry.event_type {
                                            EventType::Selection => egui::Color32::from_rgb(100, 150, 255),
                                            EventType::Visibility => egui::Color32::from_rgb(255, 200, 100),
                                            EventType::Lock => egui::Color32::from_rgb(255, 150, 150),
                                            EventType::DragDrop => egui::Color32::from_rgb(150, 255, 150),
                                            EventType::Rename => egui::Color32::from_rgb(200, 150, 255),
                                        };
                                        
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("•").color(color));
                                            
                                            // Format timestamp for display
                                            if let Ok(duration) = entry.timestamp.elapsed() {
                                                let secs = duration.as_secs();
                                                let time_str = if secs < 60 {
                                                    format!("{}s ago", secs)
                                                } else {
                                                    format!("{}m ago", secs / 60)
                                                };
                                                ui.label(egui::RichText::new(time_str)
                                                    .small()
                                                    .color(egui::Color32::GRAY));
                                            }
                                            
                                            ui.label(&entry.message);
                                        });
                                    }
                                }
                            });
                    }
                });
        }

        // Central panel with the outliner
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tree Structure");
            ui.separator();
            
            // Show the outliner widget
            let response = Outliner::new("example_outliner")
                .show(ui, &self.tree, &mut self.actions);

            // Handle rename events
            // When a user double-clicks and edits a node name, this callback fires
            if let Some((id, new_name)) = response.renamed() {
                // Update the node name in the tree structure
                for root in &mut self.tree {
                    if root.rename_node(*id, new_name.to_string()) {
                        break;
                    }
                }
            }

            // Handle drag-drop events
            // When a user drags a node and drops it on a target, this callback fires
            if let Some(drop_event) = response.drop_event() {
                let target_id = &drop_event.target;
                let position = drop_event.position;

                // Get all nodes being dragged (primary + selected)
                let dragging_ids = response.dragging_nodes();
                
                if !dragging_ids.is_empty() {
                    // Step 1: Remove all dragging nodes from their current locations
                    let mut removed_nodes = Vec::new();
                    for drag_id in dragging_ids {
                        for root in &mut self.tree {
                            if let Some(node) = root.remove_node(*drag_id) {
                                removed_nodes.push(node);
                                break;
                            }
                        }
                    }

                    // Step 2: Insert all nodes at the target position
                    let mut all_inserted = true;
                    for node in removed_nodes {
                        let mut inserted = false;
                        for root in &mut self.tree {
                            if root.insert_node(*target_id, node.clone(), position) {
                                inserted = true;
                                break;
                            }
                        }
                        if !inserted {
                            all_inserted = false;
                        }
                    }
                    
                    if all_inserted {
                        self.actions.log_event(
                            format!("✓ Successfully moved {} node(s) to target {} ({:?})",
                                dragging_ids.len(), target_id, position),
                            EventType::DragDrop,
                        );
                    } else {
                        self.actions.log_event(
                            format!("✗ Failed to move some nodes to target {}", target_id),
                            EventType::DragDrop,
                        );
                    }
                }
            }

            ui.separator();
            
            // Status bar showing current frame state
            ui.horizontal(|ui| {
                if response.changed() {
                    ui.label(egui::RichText::new("✓ State changed this frame")
                        .color(egui::Color32::from_rgb(100, 255, 100)));
                }
                
                // Show additional event information
                if let Some(id) = response.double_clicked() {
                    ui.separator();
                    ui.label(format!("Double-clicked: {}", id));
                }

                if let Some(id) = response.context_menu() {
                    ui.separator();
                    ui.label(format!("Context menu: {}", id));
                }
            });
        });
    }
}