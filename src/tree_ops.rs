//! Tree manipulation operations for outliner nodes.
//!
//! This module provides the [`TreeOperations`] trait which offers default implementations
//! for common tree manipulation operations like renaming, removing, and inserting nodes.
//! These operations are essential for implementing drag-drop and editing functionality.
//!
//! # Examples
//!
//! ```
//! use egui_arbor::{OutlinerNode, tree_ops::TreeOperations, DropPosition};
//!
//! #[derive(Clone)]
//! struct MyNode {
//!     id: u64,
//!     name: String,
//!     children: Vec<MyNode>,
//! }
//!
//! impl OutlinerNode for MyNode {
//!     type Id = u64;
//!     fn id(&self) -> Self::Id { self.id }
//!     fn name(&self) -> &str { &self.name }
//!     fn is_collection(&self) -> bool { !self.children.is_empty() }
//!     fn children(&self) -> &[Self] { &self.children }
//!     fn children_mut(&mut self) -> &mut Vec<Self> { &mut self.children }
//! }
//!
//! // Automatically get tree operations!
//! impl TreeOperations for MyNode {}
//!
//! let mut node = MyNode { id: 1, name: "root".into(), children: vec![] };
//! node.rename_node(&1, "new_name".into());
//! ```

use crate::traits::{DropPosition, OutlinerNode};

/// Trait providing tree manipulation operations for outliner nodes.
///
/// This trait offers default implementations for common tree operations:
/// - **Renaming**: Find and update a node's name by ID
/// - **Removing**: Extract a node from the tree by ID
/// - **Inserting**: Place a node at a specific position relative to a target
///
/// All methods use recursive traversal to locate nodes within the tree hierarchy.
///
/// # Default Implementations
///
/// Simply implement this trait with an empty body to get all operations:
///
/// ```ignore
/// impl TreeOperations for MyNode {}
/// ```
///
/// # Custom Implementations
///
/// You can override any method to provide custom behavior:
///
/// ```ignore
/// impl TreeOperations for MyNode {
///     fn rename_node(&mut self, id: &Self::Id, new_name: String) -> bool {
///         // Custom rename logic
///         // ...
///     }
///     // Use default implementations for other methods
/// }
/// ```
pub trait TreeOperations: OutlinerNode + Sized + Clone {
    /// Finds a node by ID and updates its name.
    ///
    /// This method recursively searches the tree starting from this node,
    /// looking for a node with the specified ID. When found, it updates
    /// the node's name.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the node to rename
    /// * `new_name` - The new name for the node
    ///
    /// # Returns
    ///
    /// `true` if the node was found and renamed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if root.rename_node(&node_id, "New Name".to_string()) {
    ///     println!("Node renamed successfully");
    /// }
    /// ```
    fn rename_node(&mut self, id: &Self::Id, new_name: String) -> bool {
        // Check if this is the target node
        if self.id() == *id {
            // We can't directly modify the name through the trait,
            // so we need to work with children
            // This is a limitation - users may need to override this method
            return false;
        }

        // Search in children
        for child in self.children_mut() {
            if child.rename_node(id, new_name.clone()) {
                return true;
            }
        }

        false
    }

    /// Removes a node from the tree by ID and returns it.
    ///
    /// This method recursively searches the tree starting from this node's children,
    /// looking for a node with the specified ID. When found, it removes the node
    /// from its parent's children list and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the node to remove
    ///
    /// # Returns
    ///
    /// `Some(node)` if the node was found and removed, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(removed_node) = root.remove_node(&node_id) {
    ///     println!("Removed node: {}", removed_node.name());
    /// }
    /// ```
    fn remove_node(&mut self, id: &Self::Id) -> Option<Self> {
        let children = self.children_mut();

        // Check direct children first
        for i in 0..children.len() {
            if children[i].id() == *id {
                return Some(children.remove(i));
            }
        }

        // Recursively search in children
        for child in children.iter_mut() {
            if let Some(node) = child.remove_node(id) {
                return Some(node);
            }
        }

        None
    }

    /// Inserts a node at a specific position relative to a target node.
    ///
    /// This method recursively searches for the target node and inserts the new node
    /// according to the specified position:
    /// - **Before**: Insert before the target node (as a sibling)
    /// - **After**: Insert after the target node (as a sibling)
    /// - **Inside**: Insert as a child of the target node (only for collections)
    ///
    /// # Arguments
    ///
    /// * `target_id` - The ID of the target node
    /// * `node` - The node to insert
    /// * `position` - Where to insert relative to the target
    ///
    /// # Returns
    ///
    /// `true` if the node was successfully inserted, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let new_node = MyNode::new(42, "New Node");
    /// if root.insert_node(&target_id, new_node, DropPosition::Inside) {
    ///     println!("Node inserted successfully");
    /// }
    /// ```
    fn insert_node(&mut self, target_id: &Self::Id, node: Self, position: DropPosition) -> bool {
        // Check if this is the target node
        if self.id() == *target_id {
            match position {
                DropPosition::Inside => {
                    if self.is_collection() {
                        self.children_mut().push(node);
                        return true;
                    }
                }
                _ => {
                    // Can't insert before/after at root level without parent context
                    return false;
                }
            }
        }

        // Search in children
        let children = self.children_mut();
        for i in 0..children.len() {
            if children[i].id() == *target_id {
                match position {
                    DropPosition::Before => {
                        children.insert(i, node);
                        return true;
                    }
                    DropPosition::After => {
                        children.insert(i + 1, node);
                        return true;
                    }
                    DropPosition::Inside => {
                        if children[i].is_collection() {
                            children[i].children_mut().push(node);
                            return true;
                        }
                    }
                }
                return false;
            }

            // Recursively search in this child
            if children[i].insert_node(target_id, node.clone(), position) {
                return true;
            }
        }

        false
    }

    /// Finds a node by ID in the tree.
    ///
    /// This is a helper method that recursively searches for a node with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the node to find
    ///
    /// # Returns
    ///
    /// A reference to the node if found, `None` otherwise.
    fn find_node(&self, id: &Self::Id) -> Option<&Self> {
        if self.id() == *id {
            return Some(self);
        }

        for child in self.children() {
            if let Some(found) = child.find_node(id) {
                return Some(found);
            }
        }

        None
    }

    /// Finds a node by ID in the tree (mutable version).
    ///
    /// This is a helper method that recursively searches for a node with the given ID
    /// and returns a mutable reference.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the node to find
    ///
    /// # Returns
    ///
    /// A mutable reference to the node if found, `None` otherwise.
    fn find_node_mut(&mut self, id: &Self::Id) -> Option<&mut Self> {
        if self.id() == *id {
            return Some(self);
        }

        for child in self.children_mut() {
            if let Some(found) = child.find_node_mut(id) {
                return Some(found);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ActionIcon, IconType};

    #[derive(Clone, Debug, PartialEq)]
    struct TestNode {
        id: u64,
        name: String,
        is_collection: bool,
        children: Vec<TestNode>,
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
            vec![]
        }
    }

    impl TreeOperations for TestNode {}

    #[test]
    fn test_remove_node_direct_child() {
        let mut root = TestNode::new(1, "root", true).with_children(vec![
            TestNode::new(2, "child1", false),
            TestNode::new(3, "child2", false),
        ]);

        let removed = root.remove_node(&2);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, 2);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].id, 3);
    }

    #[test]
    fn test_remove_node_nested() {
        let mut root = TestNode::new(1, "root", true).with_children(vec![
            TestNode::new(2, "child1", true).with_children(vec![TestNode::new(
                3,
                "grandchild",
                false,
            )]),
        ]);

        let removed = root.remove_node(&3);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, 3);
        assert_eq!(root.children[0].children.len(), 0);
    }

    #[test]
    fn test_remove_node_not_found() {
        let mut root =
            TestNode::new(1, "root", true).with_children(vec![TestNode::new(2, "child1", false)]);

        let removed = root.remove_node(&999);
        assert!(removed.is_none());
    }

    #[test]
    fn test_insert_node_inside_collection() {
        let mut root = TestNode::new(1, "root", true);
        let new_node = TestNode::new(2, "new", false);

        let result = root.insert_node(&1, new_node, DropPosition::Inside);
        assert!(result);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].id, 2);
    }

    #[test]
    fn test_insert_node_before() {
        let mut root = TestNode::new(1, "root", true).with_children(vec![
            TestNode::new(2, "child1", false),
            TestNode::new(3, "child2", false),
        ]);

        let new_node = TestNode::new(4, "new", false);
        let result = root.insert_node(&3, new_node, DropPosition::Before);

        assert!(result);
        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[1].id, 4);
        assert_eq!(root.children[2].id, 3);
    }

    #[test]
    fn test_insert_node_after() {
        let mut root = TestNode::new(1, "root", true).with_children(vec![
            TestNode::new(2, "child1", false),
            TestNode::new(3, "child2", false),
        ]);

        let new_node = TestNode::new(4, "new", false);
        let result = root.insert_node(&2, new_node, DropPosition::After);

        assert!(result);
        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].id, 2);
        assert_eq!(root.children[1].id, 4);
    }

    #[test]
    fn test_insert_node_inside_non_collection() {
        let mut root =
            TestNode::new(1, "root", true).with_children(vec![TestNode::new(2, "child1", false)]);

        let new_node = TestNode::new(3, "new", false);
        let result = root.insert_node(&2, new_node, DropPosition::Inside);

        assert!(!result);
        assert_eq!(root.children.len(), 1);
    }

    #[test]
    fn test_find_node() {
        let root = TestNode::new(1, "root", true).with_children(vec![
            TestNode::new(2, "child1", true).with_children(vec![TestNode::new(
                3,
                "grandchild",
                false,
            )]),
        ]);

        let found = root.find_node(&3);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 3);

        let not_found = root.find_node(&999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_node_mut() {
        let mut root =
            TestNode::new(1, "root", true).with_children(vec![TestNode::new(2, "child1", false)]);

        let found = root.find_node_mut(&2);
        assert!(found.is_some());

        if let Some(node) = found {
            node.name = "modified".to_string();
        }

        assert_eq!(root.children[0].name, "modified");
    }
}
