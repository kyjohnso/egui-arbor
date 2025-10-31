//! Basic usage example for egui-arbor outliner widget.
//!
//! This example demonstrates:
//! - Creating a simple tree data structure
//! - Implementing OutlinerNode and OutlinerActions traits
//! - Using the Outliner widget in an eframe application
//! - Expand/collapse functionality
//! - Node selection
//! - Rename functionality (double-click to edit)
//! - Action icons (visibility, lock, selection)
//!
//! To run this example:
//! ```
//! cargo run --example basic
//! ```

use egui_arbor::{
    ActionIcon, DropPosition, IconType, Outliner, OutlinerActions, OutlinerNode,
};
use std::collections::HashSet;

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
        Box::new(|_cc| Ok(Box::new(ExampleApp::new()))),
    )
}

/// A simple tree node that can represent files and folders
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

    fn icon(&self) -> Option<IconType> {
        if self.is_collection {
            Some(IconType::Collection)
        } else {
            Some(IconType::Entity)
        }
    }

    fn action_icons(&self) -> Vec<ActionIcon> {
        vec![
            ActionIcon::Visibility,
            ActionIcon::Lock,
            ActionIcon::Selection,
        ]
    }
}

/// Actions handler for the tree
struct TreeActions {
    selected: Option<u64>,
    visible: HashSet<u64>,
    locked: HashSet<u64>,
}

impl TreeActions {
    fn new() -> Self {
        // Initialize with all nodes visible by default
        let mut visible = HashSet::new();
        for id in 0..46 {
            visible.insert(id);
        }

        Self {
            selected: None,
            visible,
            locked: HashSet::new(),
        }
    }
}

impl OutlinerActions<TreeNode> for TreeActions {
    fn on_rename(&mut self, _id: &u64, _new_name: String) {
        // Renaming is handled in the app's update method
        // This callback is just for notification
    }

    fn on_move(&mut self, id: &u64, target: &u64, position: DropPosition) {
        // Store the move operation for the app to handle
        // We can't modify the tree directly here since we don't have access to it
        println!("Move requested: node {} to target {} at position {:?}", id, target, position);
    }

    fn on_select(&mut self, id: &u64, selected: bool) {
        if selected {
            self.selected = Some(*id);
        } else if self.selected == Some(*id) {
            self.selected = None;
        }
    }

    fn is_selected(&self, id: &u64) -> bool {
        self.selected == Some(*id)
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

    fn on_custom_action(&mut self, _id: &u64, _icon: &str) {
        // Custom actions not used in this example
    }
}

/// The main application
struct ExampleApp {
    tree: Vec<TreeNode>,
    actions: TreeActions,
    last_drop_event: Option<String>,
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
            last_drop_event: None,
        }
    }
}

impl eframe::App for ExampleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌳 egui-arbor Basic Example");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Instructions:");
                ui.label("• Click to select");
                ui.label("• Double-click to rename");
                ui.label("• Click arrows to expand/collapse");
            });

            ui.separator();

            // Show the outliner
            let response = Outliner::new("example_outliner").show(ui, &self.tree, &mut self.actions);

            // Handle rename events
            if let Some((id, new_name)) = response.renamed() {
                // Update the node name in the tree
                for root in &mut self.tree {
                    if root.rename_node(*id, new_name.to_string()) {
                        break;
                    }
                }
            }

            // Handle drag-drop events
            if let Some(drop_event) = response.drop_event() {
                let source_id = &drop_event.source;
                let target_id = &drop_event.target;
                let position = drop_event.position;

                // Remove the source node from the tree
                let mut removed_node = None;
                for root in &mut self.tree {
                    if let Some(node) = root.remove_node(*source_id) {
                        removed_node = Some(node);
                        break;
                    }
                }

                // Insert the node at the target position
                if let Some(node) = removed_node {
                    let mut inserted = false;
                    for root in &mut self.tree {
                        if root.insert_node(*target_id, node.clone(), position) {
                            inserted = true;
                            break;
                        }
                    }
                    
                    if inserted {
                        self.last_drop_event = Some(format!(
                            "Moved node {} to target {} ({:?})",
                            source_id, target_id, position
                        ));
                    }
                }
            }

            ui.separator();

            // Display information about the current state
            ui.horizontal(|ui| {
                ui.label("Status:");
                
                if let Some(selected_id) = self.actions.selected {
                    ui.label(format!("Selected node ID: {}", selected_id));
                } else {
                    ui.label("No node selected");
                }

                ui.separator();

                if response.changed() {
                    ui.label("✓ State changed this frame");
                }
            });

            // Show event information
            if let Some(id) = response.double_clicked() {
                ui.label(format!("Double-clicked node: {}", id));
            }

            if let Some(id) = response.context_menu() {
                ui.label(format!("Context menu requested for node: {}", id));
            }

            if let Some(ref event) = self.last_drop_event {
                ui.label(format!("Last drop: {}", event));
            }
        });
    }
}