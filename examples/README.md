# Examples

This directory contains examples demonstrating the usage of egui-arbor.

## Basic Example

A comprehensive example showing all egui-arbor features in a standalone egui application.

```bash
cargo run --example basic
```

Features demonstrated:
- Tree structure with collections and entities
- All action icons (visibility, lock, selection)
- Drag & drop with visual feedback
- Inline renaming with double-click
- Event logging and statistics
- Custom styling options

## Bevy 3D Outliner Example

An example integrating egui-arbor with Bevy 0.16.1, featuring a 3D scene with synchronized tree outliner.

```bash
cargo run --example bevy_3d_outliner
```

Features demonstrated:
- Integration with Bevy game engine
- 3D scene with three collections (Red, Green, Blue)
- Each collection contains a cube, cylinder, and cone
- Tree outliner synchronized with 3D scene visibility
- Orbit camera controls with mouse:
  - **Left mouse button**: Orbit camera around the scene
  - **Right mouse button**: Pan camera
  - **Mouse wheel**: Zoom in/out
- Click the eye icon (👁) in the outliner to toggle object visibility in the 3D scene

The example creates 9 objects total (3 shapes × 3 colors) arranged in a grid pattern, with a tree outliner on the left side that controls their visibility in real-time.