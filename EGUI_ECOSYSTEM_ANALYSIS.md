# egui Ecosystem Analysis for egui-arbor

## Overview

This document analyzes major egui ecosystem libraries to ensure egui-arbor follows established patterns and conventions.

## Major egui Libraries Analyzed

1. **egui_extras** - Official extras (tables, file dialogs, etc.)
2. **egui_plot** - Plotting library
3. **egui_dock** - Docking system
4. **egui_tiles** - Tiling layout manager
5. **egui-notify** - Toast notifications
6. **egui_graphs** - Graph visualization
7. **egui-modal** - Modal dialogs

## Common Patterns in egui Ecosystem

### 1. Widget API Pattern

**Standard Pattern:**
```rust
// Builder pattern with method chaining
Widget::new(id)
    .option1(value)
    .option2(value)
    .show(ui)
```

**Examples:**

**egui_extras::TableBuilder:**
```rust
TableBuilder::new(ui)
    .striped(true)
    .resizable(true)
    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
    .column(Column::auto())
    .body(|mut body| { /* ... */ });
```

**egui_plot::Plot:**
```rust
Plot::new("my_plot")
    .view_aspect(2.0)
    .show(ui, |plot_ui| {
        plot_ui.line(line);
    });
```

**egui_dock::DockArea:**
```rust
DockArea::new(&mut tree)
    .style(style)
    .show(ctx, &mut tab_viewer);
```

### 2. State Management Pattern

**Standard Pattern:**
- Use `egui::Id` for widget identification
- Store state in `egui::Memory` via `ui.data()` or `ui.data_mut()`
- Provide state structs that implement `Clone` and optionally `serde::Serialize`

**Examples:**

**egui_dock:**
```rust
pub struct DockState<Tab> {
    tree: Tree<Tab>,
}

impl<Tab> DockState<Tab> {
    pub fn new(tabs: Vec<Tab>) -> Self { /* ... */ }
}

// Used with egui's memory
DockArea::new(&mut state)
    .show(ctx, &mut tab_viewer);
```

**egui-notify:**
```rust
pub struct Toasts {
    toasts: VecDeque<Toast>,
    // ...
}

// Stored in egui memory
let mut toasts = ctx.data_mut(|d| d.get_temp::<Toasts>(Id::null()));
```

### 3. Response Pattern

**Standard Pattern:**
- Return a response struct with interaction information
- Include `inner` field for nested responses
- Provide helper methods for common queries

**Examples:**

**egui::Response:**
```rust
pub struct Response {
    pub clicked: bool,
    pub hovered: bool,
    pub dragged: bool,
    // ...
}
```

**egui_plot::PlotResponse:**
```rust
pub struct PlotResponse<R> {
    pub response: Response,
    pub inner: R,
    pub transform: PlotTransform,
}
```

### 4. Trait-Based Customization

**Standard Pattern:**
- Define traits for user customization
- Provide default implementations
- Use generic parameters with trait bounds

**Examples:**

**egui_dock::TabViewer:**
```rust
pub trait TabViewer {
    type Tab;
    
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);
    fn context_menu(&mut self, ui: &mut Ui, tab: &mut Self::Tab) { /* default */ }
}
```

**egui_extras::TableDelegate:**
```rust
pub trait TableDelegate {
    fn prepare(&mut self, row: usize);
    fn num_rows(&self) -> usize;
    fn row_ui(&mut self, ui: &mut Ui, row: usize);
}
```

### 5. Styling Pattern

**Standard Pattern:**
- Provide a `Style` struct with visual customization
- Use `Visuals` from egui for colors
- Allow per-widget style overrides

**Examples:**

**egui_dock::Style:**
```rust
pub struct Style {
    pub border: Stroke,
    pub tab_bar: TabBarStyle,
    pub separator: SeparatorStyle,
    // ...
}

impl Default for Style {
    fn default() -> Self { /* ... */ }
}
```

### 6. ID Generation Pattern

**Standard Pattern:**
- Accept `impl Into<egui::Id>` for flexibility
- Use `ui.make_persistent_id()` for automatic ID generation
- Support both string and numeric IDs

**Examples:**

**egui_plot:**
```rust
Plot::new("my_plot")  // String ID
Plot::new(egui::Id::new(42))  // Numeric ID
```

### 7. Memory Persistence Pattern

**Standard Pattern:**
- Use `ui.data()` for read-only access
- Use `ui.data_mut()` for mutable access
- Implement `Clone` for state structs
- Optionally implement `serde::{Serialize, Deserialize}` for persistence

**Examples:**

**egui_extras (internal):**
```rust
let state = ui.data_mut(|d| {
    d.get_temp_mut_or_default::<State>(id)
});
```

### 8. Callback Pattern

**Standard Pattern:**
- Use closures for user callbacks
- Provide context objects with necessary data
- Return values from callbacks when needed

**Examples:**

**egui_extras::TableBuilder:**
```rust
.body(|mut body| {
    body.row(30.0, |mut row| {
        row.col(|ui| { ui.label("Cell"); });
    });
})
```

**egui_plot::Plot:**
```rust
.show(ui, |plot_ui| {
    plot_ui.line(line);
})
```

## egui-arbor Alignment Analysis

### ✅ Aligned Patterns

1. **Builder Pattern**: egui-arbor uses builder pattern with method chaining
   ```rust
   Outliner::new(id, nodes, actions)
       .with_icons(&icon_registry)
       .with_actions(&mut action_registry)
       .show(ui);
   ```

2. **ID-based State**: Uses `egui::Id` for widget identification
   ```rust
   pub struct Outliner<'a, N: OutlinerNode> {
       id: egui::Id,
       // ...
   }
   ```

3. **Trait-Based Customization**: Uses traits for user data integration
   ```rust
   pub trait OutlinerNode { /* ... */ }
   pub trait OutlinerActions<N: OutlinerNode> { /* ... */ }
   ```

4. **Response Pattern**: Returns response with interaction info
   ```rust
   pub struct OutlinerResponse {
       pub changed: bool,
       pub selected: Vec<NodeId>,
   }
   ```

### ⚠️ Areas Needing Adjustment

1. **State Management**: Current design stores state in `WidgetState` struct, but should use egui's memory system more directly

2. **Registry Pattern**: Icon and action registries are passed as parameters, but could be stored in egui memory

3. **Lifetime Management**: Current design uses lifetimes extensively, but egui ecosystem prefers owned data or `Arc`

4. **Styling**: No explicit `Style` struct defined yet

## Recommended Architecture Updates

### 1. State Management Refinement

**Current:**
```rust
pub struct WidgetState {
    pub expanded: HashSet<NodeId>,
    pub selection: HashSet<NodeId>,
    pub editing: Option<EditState>,
    pub drag_state: DragState<NodeId>,
}
```

**Recommended:**
```rust
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutlinerState {
    expanded: HashSet<egui::Id>,
    editing: Option<egui::Id>,
}

impl OutlinerState {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Self {
        ctx.data(|d| d.get_temp(id).unwrap_or_default())
    }
    
    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}
```

### 2. Simplified Widget API

**Current:**
```rust
pub struct Outliner<'a, N: OutlinerNode> {
    id: egui::Id,
    nodes: &'a mut [N],
    actions: &'a mut dyn OutlinerActions<N>,
    icon_registry: &'a IconRegistry,
    action_registry: &'a mut ActionRegistry<N>,
}
```

**Recommended:**
```rust
pub struct Outliner<'a, N> {
    id: egui::Id,
    nodes: &'a mut [N],
    style: Option<&'a Style>,
}

impl<'a, N: OutlinerNode> Outliner<'a, N> {
    pub fn new(id: impl Into<egui::Id>, nodes: &'a mut [N]) -> Self {
        Self {
            id: id.into(),
            nodes,
            style: None,
        }
    }
    
    pub fn style(mut self, style: &'a Style) -> Self {
        self.style = Some(style);
        self
    }
    
    pub fn show<A: OutlinerActions<N>>(
        self,
        ui: &mut egui::Ui,
        actions: &mut A,
    ) -> OutlinerResponse<N::Id> {
        // Implementation
    }
}
```

### 3. Add Style Struct

```rust
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Style {
    pub indent: f32,
    pub icon_spacing: f32,
    pub row_height: f32,
    pub expand_icon: ExpandIconStyle,
    pub selection_color: Option<egui::Color32>,
    pub hover_color: Option<egui::Color32>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            indent: 16.0,
            icon_spacing: 4.0,
            row_height: 20.0,
            expand_icon: ExpandIconStyle::default(),
            selection_color: None,
            hover_color: None,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpandIconStyle {
    pub collapsed: char,
    pub expanded: char,
    pub size: f32,
}

impl Default for ExpandIconStyle {
    fn default() -> Self {
        Self {
            collapsed: '▶',
            expanded: '▼',
            size: 12.0,
        }
    }
}
```

### 4. Icon System Simplification

**Current:** Registry-based with separate icon and action registries

**Recommended:** Integrate with trait system

```rust
pub trait OutlinerNode {
    type Id: Hash + Eq + Clone;
    
    fn id(&self) -> Self::Id;
    fn name(&self) -> &str;
    fn is_collection(&self) -> bool;
    fn children(&self) -> Option<&[Self]> where Self: Sized;
    fn children_mut(&mut self) -> Option<&mut Vec<Self>> where Self: Sized;
    
    // Icon methods
    fn icon(&self) -> Option<&str> {
        None
    }
    
    fn action_icons(&self) -> Vec<ActionIcon> {
        vec![
            ActionIcon::Visibility,
            ActionIcon::Lock,
            ActionIcon::Selection,
        ]
    }
}

pub enum ActionIcon {
    Visibility,
    Lock,
    Selection,
    Custom { icon: String, tooltip: Option<String> },
}
```

### 5. Response Enhancement

```rust
pub struct OutlinerResponse<Id> {
    /// The response from the outliner widget itself
    pub response: egui::Response,
    
    /// Whether any changes were made
    pub changed: bool,
    
    /// Currently selected node IDs
    pub selected: Vec<Id>,
    
    /// Node that was clicked (if any)
    pub clicked: Option<Id>,
    
    /// Node that was double-clicked (if any)
    pub double_clicked: Option<Id>,
    
    /// Node that was right-clicked (if any)
    pub context_menu: Option<Id>,
    
    /// Drag-drop operation that occurred (if any)
    pub drag_drop: Option<DragDropEvent<Id>>,
}

pub struct DragDropEvent<Id> {
    pub dragged: Id,
    pub target: Id,
    pub position: DropPosition,
}
```

### 6. Drag-Drop Integration

Use egui's built-in drag-drop more directly:

```rust
impl<'a, N: OutlinerNode> Outliner<'a, N> {
    fn render_node(
        &self,
        ui: &mut egui::Ui,
        node: &N,
        state: &mut OutlinerState,
    ) -> egui::Response {
        let id = ui.make_persistent_id(node.id());
        
        // Drag source
        let response = ui.horizontal(|ui| {
            // Node content
        }).response;
        
        let response = response.dnd_set_drag_payload(node.id());
        
        // Drop target
        if let Some(payload) = response.dnd_hover_payload::<N::Id>() {
            // Handle drop visualization
        }
        
        if let Some(payload) = response.dnd_release_payload::<N::Id>() {
            // Handle drop
        }
        
        response
    }
}
```

## Updated Architecture Summary

### Core Changes

1. **State Management**: Use egui's memory system directly with `OutlinerState`
2. **Simplified API**: Remove registry parameters, integrate into trait methods
3. **Style System**: Add `Style` struct following egui ecosystem patterns
4. **Response Enhancement**: Richer response with more interaction details
5. **Drag-Drop**: Use egui's built-in drag-drop API directly
6. **Lifetime Simplification**: Reduce lifetime complexity where possible

### Module Structure (Updated)

```
egui-arbor/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── outliner.rs         # Main Outliner widget
│   ├── state.rs            # OutlinerState (egui memory integration)
│   ├── style.rs            # Style configuration
│   ├── response.rs         # OutlinerResponse and related types
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── node.rs         # OutlinerNode trait
│   │   └── actions.rs      # OutlinerActions trait
│   ├── icons/
│   │   ├── mod.rs
│   │   └── builtin.rs      # Built-in icon definitions
│   ├── drag_drop/
│   │   ├── mod.rs
│   │   └── types.rs        # Drag-drop types
│   └── utils/
│       ├── mod.rs
│       └── id.rs           # ID utilities
```

### Simplified Usage Example

```rust
use egui_arbor::*;

#[derive(Clone)]
struct SceneNode {
    id: u64,
    name: String,
    children: Vec<SceneNode>,
    visible: bool,
    locked: bool,
}

impl OutlinerNode for SceneNode {
    type Id = u64;
    
    fn id(&self) -> Self::Id { self.id }
    fn name(&self) -> &str { &self.name }
    fn is_collection(&self) -> bool { !self.children.is_empty() }
    fn children(&self) -> Option<&[Self]> {
        if self.children.is_empty() { None } else { Some(&self.children) }
    }
    fn children_mut(&mut self) -> Option<&mut Vec<Self>> {
        if self.children.is_empty() { None } else { Some(&mut self.children) }
    }
}

struct MyActions {
    selection: HashSet<u64>,
}

impl OutlinerActions<SceneNode> for MyActions {
    fn on_rename(&mut self, node_id: &u64, new_name: String) { /* ... */ }
    fn on_move(&mut self, node_id: &u64, new_parent: Option<&u64>, index: usize) { /* ... */ }
    fn on_select(&mut self, node_id: &u64, multi_select: bool) { /* ... */ }
    fn is_selected(&self, node_id: &u64) -> bool { self.selection.contains(node_id) }
    fn is_visible(&self, node_id: &u64) -> bool { /* ... */ }
    fn is_locked(&self, node_id: &u64) -> bool { /* ... */ }
    fn toggle_visibility(&mut self, node_id: &u64) { /* ... */ }
    fn toggle_lock(&mut self, node_id: &u64) { /* ... */ }
}

// In your egui code
fn show_outliner(ui: &mut egui::Ui, nodes: &mut [SceneNode], actions: &mut MyActions) {
    let response = Outliner::new("scene_outliner", nodes)
        .show(ui, actions);
    
    if response.changed {
        // Handle changes
    }
    
    if let Some(clicked) = response.clicked {
        // Handle click
    }
}
```

## Comparison with Similar Libraries

### egui_dock (Most Similar)

**Similarities:**
- Hierarchical structure (tabs in dock areas)
- Drag-drop support
- Trait-based customization (`TabViewer`)
- State management via egui memory

**Differences:**
- egui_dock manages layout, egui-arbor manages hierarchy
- egui_dock uses `Tree<Tab>`, egui-arbor uses user's data structure

**Lessons:**
- Use `Tree` pattern for internal state if needed
- Provide clear separation between data and presentation
- Use trait for rendering customization

### egui_extras::TableBuilder

**Similarities:**
- Hierarchical display (rows/columns)
- Builder pattern
- Callback-based content

**Differences:**
- Table is more structured, outliner is more flexible
- Table uses delegates, outliner uses traits

**Lessons:**
- Provide both high-level and low-level APIs
- Use closures for simple cases, traits for complex ones

## Conclusion

The updated architecture aligns egui-arbor with egui ecosystem conventions:

1. ✅ Uses egui's memory system for state
2. ✅ Follows builder pattern with method chaining
3. ✅ Provides rich response types
4. ✅ Uses trait-based customization
5. ✅ Includes style configuration
6. ✅ Integrates with egui's drag-drop
7. ✅ Supports optional serde serialization
8. ✅ Simplifies API surface

These changes make egui-arbor feel like a natural part of the egui ecosystem while maintaining its unique functionality.