# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-11-12

This is the first tagged release of egui-arbor, a hierarchical tree view widget for egui with drag-and-drop support, multi-selection, and customizable styling.

### Added
- **Bevy Integration**: Complete 3D outliner example demonstrating integration with Bevy game engine
  - Drag-and-drop support for scene hierarchy manipulation
  - Visibility toggling for 3D entities
  - Inline rename functionality for scene objects
- **Multi-Selection**: Full multi-select support with keyboard modifiers (Ctrl/Cmd for toggle, Shift for range)
  - Multi-select drag-and-drop operations
  - Batch operations on selected items
- **Drag and Drop System**: Comprehensive drag-and-drop implementation
  - Hierarchical drag-and-drop with parent-child relationships
  - Top-level drag-and-drop improvements
  - Visual feedback during drag operations
- **Action Icons**: Customizable action buttons for tree items
  - Built-in actions for common operations
  - Extensible action system via traits
- **Inline Editing**: Direct text editing of tree item labels
- **Examples**: Multiple example implementations
  - Basic usage example
  - Bevy 3D outliner integration example
  - Comprehensive examples documentation
- **Testing**: Library test suite for core functionality
- **CI/CD**: GitHub Actions workflow for automated testing and validation
- **Documentation**: 
  - README with usage examples and feature overview
  - Architecture documentation
  - Example images and visual guides
  - Ecosystem analysis documentation

### Changed
- **Library Structure**: Refactored codebase with improved organization
  - Moved core functions to library modules
  - Enhanced modularity and reusability
  - Trait-based integration system for custom implementations
- **egui Compatibility**: Updated to egui 0.32
- **Project Metadata**: Prepared for crates.io publication
  - Added dual licensing (Apache-2.0 OR MIT)
  - Updated Cargo.toml with complete package metadata

### Fixed
- Various stability improvements and bug fixes throughout development

[Unreleased]: https://github.com/kyjohnso/egui-arbor/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kyjohnso/egui-arbor/releases/tag/v0.2.0