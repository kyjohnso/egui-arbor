//! Style configuration for the outliner widget.
//!
//! This module provides types for customizing the visual appearance of the outliner,
//! including colors, spacing, and icon styles.

/// Style configuration for the outliner widget.
///
/// Controls the visual appearance including spacing, colors, and icon sizes.
/// Use the builder pattern methods for convenient construction:
///
/// ```rust
/// use egui_arbor::Style;
/// use egui::Color32;
///
/// let style = Style::default()
///     .with_indent(20.0)
///     .with_selection_color(Color32::from_rgb(100, 150, 200));
/// ```
#[derive(Debug, Clone)]
pub struct Style {
    /// Indentation per hierarchy level in logical pixels.
    ///
    /// Default: 16.0
    pub indent: f32,

    /// Spacing between icon and text in logical pixels.
    ///
    /// Default: 4.0
    pub icon_spacing: f32,

    /// Height of each row in logical pixels.
    ///
    /// Default: 20.0
    pub row_height: f32,

    /// Size of expand/collapse arrow in logical pixels.
    ///
    /// Default: 12.0
    pub expand_icon_size: f32,

    /// Size of action icons in logical pixels.
    ///
    /// Default: 16.0
    pub action_icon_size: f32,

    /// Optional selection highlight color.
    ///
    /// If `None`, uses egui's default selection color.
    pub selection_color: Option<egui::Color32>,

    /// Optional hover highlight color.
    ///
    /// If `None`, uses egui's default hover color.
    pub hover_color: Option<egui::Color32>,

    /// Style of the expand/collapse icon.
    ///
    /// Default: `ExpandIconStyle::Arrow`
    pub expand_icon_style: ExpandIconStyle,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            indent: 16.0,
            icon_spacing: 4.0,
            row_height: 20.0,
            expand_icon_size: 12.0,
            action_icon_size: 16.0,
            selection_color: Some(egui::Color32::from_rgba_unmultiplied(100, 150, 200, 100)),
            hover_color: Some(egui::Color32::from_rgba_unmultiplied(100, 150, 200, 50)),
            expand_icon_style: ExpandIconStyle::Arrow,
        }
    }
}

impl Style {
    /// Set the indentation per hierarchy level.
    ///
    /// # Arguments
    /// * `indent` - Indentation in logical pixels
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    ///
    /// let style = Style::default().with_indent(20.0);
    /// ```
    pub fn with_indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    /// Set the spacing between icon and text.
    ///
    /// # Arguments
    /// * `spacing` - Spacing in logical pixels
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    ///
    /// let style = Style::default().with_icon_spacing(6.0);
    /// ```
    pub fn with_icon_spacing(mut self, spacing: f32) -> Self {
        self.icon_spacing = spacing;
        self
    }

    /// Set the height of each row.
    ///
    /// # Arguments
    /// * `height` - Row height in logical pixels
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    ///
    /// let style = Style::default().with_row_height(24.0);
    /// ```
    pub fn with_row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Set the size of expand/collapse arrows.
    ///
    /// # Arguments
    /// * `size` - Icon size in logical pixels
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    ///
    /// let style = Style::default().with_expand_icon_size(14.0);
    /// ```
    pub fn with_expand_icon_size(mut self, size: f32) -> Self {
        self.expand_icon_size = size;
        self
    }

    /// Set the size of action icons.
    ///
    /// # Arguments
    /// * `size` - Icon size in logical pixels
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    ///
    /// let style = Style::default().with_action_icon_size(18.0);
    /// ```
    pub fn with_action_icon_size(mut self, size: f32) -> Self {
        self.action_icon_size = size;
        self
    }

    /// Set the selection highlight color.
    ///
    /// # Arguments
    /// * `color` - The color to use for selection highlighting
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    /// use egui::Color32;
    ///
    /// let style = Style::default()
    ///     .with_selection_color(Color32::from_rgb(100, 150, 200));
    /// ```
    pub fn with_selection_color(mut self, color: egui::Color32) -> Self {
        self.selection_color = Some(color);
        self
    }

    /// Set the hover highlight color.
    ///
    /// # Arguments
    /// * `color` - The color to use for hover highlighting
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::Style;
    /// use egui::Color32;
    ///
    /// let style = Style::default()
    ///     .with_hover_color(Color32::from_rgb(150, 180, 210));
    /// ```
    pub fn with_hover_color(mut self, color: egui::Color32) -> Self {
        self.hover_color = Some(color);
        self
    }

    /// Set the expand/collapse icon style.
    ///
    /// # Arguments
    /// * `style` - The icon style to use
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::{Style, ExpandIconStyle};
    ///
    /// let style = Style::default()
    ///     .with_expand_icon_style(ExpandIconStyle::PlusMinus);
    /// ```
    pub fn with_expand_icon_style(mut self, style: ExpandIconStyle) -> Self {
        self.expand_icon_style = style;
        self
    }
}

/// Style of the expand/collapse icon.
///
/// Determines the visual appearance of the icon used to expand and collapse
/// tree nodes in the outliner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandIconStyle {
    /// Simple arrow style (▶ when collapsed, ▼ when expanded).
    Arrow,

    /// Plus/minus signs (+ when collapsed, - when expanded).
    PlusMinus,

    /// Chevron style (› when collapsed, ⌄ when expanded).
    ChevronRight,

    /// Custom strings for collapsed and expanded states.
    ///
    /// # Example
    /// ```rust
    /// use egui_arbor::ExpandIconStyle;
    ///
    /// let style = ExpandIconStyle::Custom {
    ///     collapsed: "→".to_string(),
    ///     expanded: "↓".to_string(),
    /// };
    /// ```
    Custom {
        /// String to display when the node is collapsed.
        collapsed: String,
        /// String to display when the node is expanded.
        expanded: String,
    },
}

impl ExpandIconStyle {
    /// Get the string representation for the collapsed state.
    ///
    /// # Returns
    /// The string to display when a node is collapsed.
    pub fn collapsed_str(&self) -> &str {
        match self {
            ExpandIconStyle::Arrow => "▶",
            ExpandIconStyle::PlusMinus => "+",
            ExpandIconStyle::ChevronRight => "›",
            ExpandIconStyle::Custom { collapsed, .. } => collapsed,
        }
    }

    /// Get the string representation for the expanded state.
    ///
    /// # Returns
    /// The string to display when a node is expanded.
    pub fn expanded_str(&self) -> &str {
        match self {
            ExpandIconStyle::Arrow => "▼",
            ExpandIconStyle::PlusMinus => "-",
            ExpandIconStyle::ChevronRight => "⌄",
            ExpandIconStyle::Custom { expanded, .. } => expanded,
        }
    }
}

impl Default for ExpandIconStyle {
    fn default() -> Self {
        Self::Arrow
    }
}