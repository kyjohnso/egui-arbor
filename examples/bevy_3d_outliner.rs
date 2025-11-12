//! Bevy 3D Outliner Example
//!
//! This example demonstrates:
//! - Integration of egui-arbor with Bevy 0.16.1
//! - 3D scene with collections and objects (cubes, cylinders, cones)
//! - Tree outliner synchronized with 3D scene visibility
//! - Drag and drop to reorganize scene hierarchy
//! - Orbit camera controls with mouse
//! - Three collections with different colored objects
//!
//! Controls:
//! - Left mouse: Orbit camera
//! - Right mouse: Pan camera
//! - Scroll: Zoom camera
//! - Drag nodes in outliner to reorganize hierarchy
//! - Click visibility icons in outliner to show/hide objects

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use egui_arbor::{
    ActionIcon, DropPosition, IconType, Outliner, OutlinerActions, OutlinerNode,
    tree_ops::TreeOperations,
};
use std::collections::HashSet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy 3D Outliner Example".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin::default())
        .init_resource::<SceneTree>()
        .init_resource::<TreeActions>()
        .add_systems(Startup, setup_scene)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, (orbit_camera_system, sync_visibility_system))
        .run();
}

/// Marker component for objects in the scene
#[derive(Component)]
struct SceneObject {
    id: u64,
}

/// Marker component for the orbit camera
#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 15.0,
        }
    }
}

/// Tree node for the outliner
#[derive(Clone, Debug)]
struct TreeNode {
    id: u64,
    name: String,
    is_collection: bool,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn collection(id: u64, name: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            name: name.into(),
            is_collection: true,
            children,
        }
    }

    fn entity(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            is_collection: false,
            children: Vec::new(),
        }
    }

    /// Recursively find and rename a node by ID
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
        vec![ActionIcon::Visibility]
    }
}

/// Implement TreeOperations to get drag-drop functionality
impl TreeOperations for TreeNode {}

/// Resource holding the scene tree structure
#[derive(Resource)]
struct SceneTree {
    nodes: Vec<TreeNode>,
}

impl Default for SceneTree {
    fn default() -> Self {
        Self {
            nodes: vec![
                TreeNode::collection(
                    0,
                    "Collection Red",
                    vec![
                        TreeNode::entity(1, "Red Cube"),
                        TreeNode::entity(2, "Red Cylinder"),
                        TreeNode::entity(3, "Red Cone"),
                    ],
                ),
                TreeNode::collection(
                    4,
                    "Collection Green",
                    vec![
                        TreeNode::entity(5, "Green Cube"),
                        TreeNode::entity(6, "Green Cylinder"),
                        TreeNode::entity(7, "Green Cone"),
                    ],
                ),
                TreeNode::collection(
                    8,
                    "Collection Blue",
                    vec![
                        TreeNode::entity(9, "Blue Cube"),
                        TreeNode::entity(10, "Blue Cylinder"),
                        TreeNode::entity(11, "Blue Cone"),
                    ],
                ),
            ],
        }
    }
}

/// Actions handler for the outliner
#[derive(Resource)]
struct TreeActions {
    visible: HashSet<u64>,
}

impl Default for TreeActions {
    fn default() -> Self {
        let mut visible = HashSet::new();
        // All nodes visible by default (collections: 0, 4, 8 and objects: 1-3, 5-7, 9-11)
        visible.insert(0); // Collection Red
        for id in 1..=3 {
            visible.insert(id);
        }
        visible.insert(4); // Collection Green
        for id in 5..=7 {
            visible.insert(id);
        }
        visible.insert(8); // Collection Blue
        for id in 9..=11 {
            visible.insert(id);
        }
        Self { visible }
    }
}

impl TreeActions {
    /// Recursively set visibility for all children of a node
    /// Blender-style: sets all children to match the parent's new state
    fn set_children_visibility(&mut self, parent_id: u64, visible: bool) {
        // Map parent IDs to their children based on the tree structure
        let child_ids = match parent_id {
            0 => vec![1, 2, 3],   // Collection Red
            4 => vec![5, 6, 7],   // Collection Green
            8 => vec![9, 10, 11], // Collection Blue
            _ => vec![],
        };

        for child_id in child_ids {
            if visible {
                self.visible.insert(child_id);
            } else {
                self.visible.remove(&child_id);
            }
        }
    }
}

impl OutlinerActions<TreeNode> for TreeActions {
    fn on_rename(&mut self, id: &u64, new_name: String) {
        // Note: The actual renaming happens in the ui_system where we have mutable access to SceneTree
        // This callback is just for tracking that a rename occurred
        println!("Rename requested for node {}: {}", id, new_name);
    }

    fn on_move(&mut self, _id: &u64, _target: &u64, _position: DropPosition) {}

    fn on_select(&mut self, _id: &u64, _selected: bool) {}

    fn is_selected(&self, _id: &u64) -> bool {
        false
    }

    fn is_visible(&self, id: &u64) -> bool {
        self.visible.contains(id)
    }

    fn is_locked(&self, _id: &u64) -> bool {
        false
    }

    fn on_visibility_toggle(&mut self, id: &u64) {
        let was_visible = self.visible.contains(id);
        let new_state = !was_visible;

        // Set the parent's new state
        if new_state {
            self.visible.insert(*id);
        } else {
            self.visible.remove(id);
        }

        // Set all children to match the parent's new state (Blender-style)
        self.set_children_visibility(*id, new_state);
    }

    fn on_lock_toggle(&mut self, _id: &u64) {}

    fn on_selection_toggle(&mut self, _id: &u64) {}

    fn on_custom_action(&mut self, _id: &u64, _icon: &str) {}
}

/// Setup the 3D scene with collections and objects
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with orbit controls
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::default(),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });

    // Create meshes
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cylinder_mesh = meshes.add(Cylinder::new(0.5, 1.0));
    let cone_mesh = meshes.add(Cone {
        radius: 0.5,
        height: 1.0,
    });

    // Collection Red (left side)
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        ..default()
    });

    // Red Cube (ID: 1)
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(red_material.clone()),
        Transform::from_xyz(-6.0, 0.5, 0.0),
        SceneObject { id: 1 },
    ));

    // Red Cylinder (ID: 2)
    commands.spawn((
        Mesh3d(cylinder_mesh.clone()),
        MeshMaterial3d(red_material.clone()),
        Transform::from_xyz(-6.0, 0.5, 2.5),
        SceneObject { id: 2 },
    ));

    // Red Cone (ID: 3)
    commands.spawn((
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(red_material),
        Transform::from_xyz(-6.0, 0.5, -2.5),
        SceneObject { id: 3 },
    ));

    // Collection Green (center)
    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        ..default()
    });

    // Green Cube (ID: 5)
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(green_material.clone()),
        Transform::from_xyz(0.0, 0.5, 0.0),
        SceneObject { id: 5 },
    ));

    // Green Cylinder (ID: 6)
    commands.spawn((
        Mesh3d(cylinder_mesh.clone()),
        MeshMaterial3d(green_material.clone()),
        Transform::from_xyz(0.0, 0.5, 2.5),
        SceneObject { id: 6 },
    ));

    // Green Cone (ID: 7)
    commands.spawn((
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(green_material),
        Transform::from_xyz(0.0, 0.5, -2.5),
        SceneObject { id: 7 },
    ));

    // Collection Blue (right side)
    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.8),
        ..default()
    });

    // Blue Cube (ID: 9)
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(blue_material.clone()),
        Transform::from_xyz(6.0, 0.5, 0.0),
        SceneObject { id: 9 },
    ));

    // Blue Cylinder (ID: 10)
    commands.spawn((
        Mesh3d(cylinder_mesh.clone()),
        MeshMaterial3d(blue_material.clone()),
        Transform::from_xyz(6.0, 0.5, 2.5),
        SceneObject { id: 10 },
    ));

    // Blue Cone (ID: 11)
    commands.spawn((
        Mesh3d(cone_mesh),
        MeshMaterial3d(blue_material),
        Transform::from_xyz(6.0, 0.5, -2.5),
        SceneObject { id: 11 },
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(20.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// UI system showing the outliner
fn ui_system(
    mut contexts: bevy_egui::EguiContexts,
    mut scene_tree: ResMut<SceneTree>,
    mut actions: ResMut<TreeActions>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    bevy_egui::egui::SidePanel::left("outliner_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("🌳 Scene Outliner");
            ui.separator();

            ui.label("Drag and drop to reorganize");
            ui.label("Click the eye icon to toggle visibility");
            ui.label("Double-click to rename");
            ui.add_space(8.0);

            let response =
                Outliner::new("scene_outliner").show(ui, &scene_tree.nodes, &mut *actions);

            // Handle rename events
            if let Some((node_id, new_name)) = response.renamed() {
                // Update the node name in the tree
                for root in &mut scene_tree.nodes {
                    if root.rename_node(*node_id, new_name.to_string()) {
                        break;
                    }
                }
            }

            // Handle drag-drop events
            if let Some(drop_event) = response.drop_event() {
                let target_id = &drop_event.target;
                let position = drop_event.position;

                // Get all nodes being dragged (primary + selected)
                let dragging_ids = response.dragging_nodes();

                if !dragging_ids.is_empty() {
                    // Step 1: Remove all dragging nodes from their current locations
                    let mut removed_nodes = Vec::new();
                    for drag_id in dragging_ids {
                        for root in &mut scene_tree.nodes {
                            if let Some(node) = root.remove_node(drag_id) {
                                removed_nodes.push(node);
                                break;
                            }
                        }
                    }

                    // Step 2: Insert all nodes at the target position
                    for node in removed_nodes {
                        let mut inserted = false;
                        for root in &mut scene_tree.nodes {
                            if root.insert_node(target_id, node.clone(), position) {
                                inserted = true;
                                break;
                            }
                        }
                        if !inserted {
                            // If insertion failed, log it (in a real app you might want to restore the node)
                            eprintln!("Failed to insert node {} at target {}", node.id, target_id);
                        }
                    }
                }
            }
        });
}

/// Orbit camera system with mouse controls
fn orbit_camera_system(
    mut query: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
) {
    let Ok((mut transform, mut orbit)) = query.single_mut() else {
        return;
    };

    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut pan_move = Vec2::ZERO;

    // Handle mouse input
    if mouse_button_input.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            rotation_move += ev.delta;
        }
    } else if mouse_button_input.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            pan_move += ev.delta;
        }
    } else {
        mouse_motion.clear();
    }

    for ev in mouse_wheel.read() {
        scroll += ev.y;
    }

    // Apply rotation
    if rotation_move.length_squared() > 0.0 {
        let delta_x = rotation_move.x * 0.003;
        let delta_y = rotation_move.y * 0.003;

        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        transform.rotation = yaw * transform.rotation * pitch;
    }

    // Apply panning
    if pan_move.length_squared() > 0.0 {
        let right = transform.rotation * Vec3::X * -pan_move.x * 0.01;
        let up = transform.rotation * Vec3::Y * pan_move.y * 0.01;
        orbit.focus += right + up;
    }

    // Apply zoom
    if scroll.abs() > 0.0 {
        orbit.radius -= scroll * orbit.radius * 0.1;
        orbit.radius = orbit.radius.clamp(2.0, 50.0);
    }

    // Update camera position
    let rot_matrix = Mat3::from_quat(transform.rotation);
    transform.translation = orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, orbit.radius));
}

/// Sync visibility between outliner and 3D scene
fn sync_visibility_system(
    mut query: Query<(&SceneObject, &mut Visibility)>,
    actions: Res<TreeActions>,
) {
    for (scene_object, mut visibility) in query.iter_mut() {
        *visibility = if actions.is_visible(&scene_object.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
