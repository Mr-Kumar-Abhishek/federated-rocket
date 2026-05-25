use crate::component::RocketComponent;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

// ============================================================================
// Key Type
// ============================================================================

new_key_type! {
    /// A unique key for referencing a component node in the tree.
    pub struct ComponentKey;
}

// ============================================================================
// Tree Node
// ============================================================================

/// A single node in the component tree, holding a component along with
/// its parent and child relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    pub component: RocketComponent,
    pub parent: Option<ComponentKey>,
    pub children: Vec<ComponentKey>,
}

// ============================================================================
// Tree Error
// ============================================================================

/// Errors that can occur during component tree operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TreeError {
    /// The specified component key was not found in the tree.
    #[error("Component not found: {0:?}")]
    NotFound(ComponentKey),

    /// The requested operation is invalid (e.g., circular reference).
    #[error("Cannot add component: {0}")]
    InvalidOperation(String),

    /// A cycle was detected that would make the tree invalid.
    #[error("Cycle detected")]
    CycleDetected,
}

// ============================================================================
// Component Tree
// ============================================================================

/// A tree structure for organizing rocket components using a slotmap.
///
/// `SlotMap` provides stable, `O(1)` key-based access to nodes and allows
/// efficient insertion and removal without invalidating existing keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTree {
    nodes: SlotMap<ComponentKey, ComponentNode>,
    root: Option<ComponentKey>,
}

impl ComponentTree {
    /// Creates a new empty component tree.
    pub fn new() -> Self {
        ComponentTree {
            nodes: SlotMap::with_key(),
            root: None,
        }
    }

    /// Adds a new component to the tree, optionally as a child of `parent`.
    ///
    /// Returns the newly allocated `ComponentKey`.
    ///
    /// # Errors
    ///
    /// Returns `TreeError::NotFound` if `parent` is `Some` but does not exist.
    pub fn add_component(
        &mut self,
        component: RocketComponent,
        parent: Option<ComponentKey>,
    ) -> Result<ComponentKey, TreeError> {
        // Validate parent exists if specified
        if let Some(parent_key) = parent {
            if !self.nodes.contains_key(parent_key) {
                return Err(TreeError::NotFound(parent_key));
            }
        }

        let key = self.nodes.insert(ComponentNode {
            component,
            parent,
            children: Vec::new(),
        });

        // Add this key to the parent's children list
        if let Some(parent_key) = parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_key) {
                parent_node.children.push(key);
            }
        }

        // If no root is set, this becomes the root
        if self.root.is_none() {
            self.root = Some(key);
        }

        Ok(key)
    }

    /// Removes a component and all its descendants from the tree.
    ///
    /// # Errors
    ///
    /// Returns `TreeError::NotFound` if the key does not exist.
    pub fn remove_component(&mut self, key: ComponentKey) -> Result<(), TreeError> {
        if !self.nodes.contains_key(key) {
            return Err(TreeError::NotFound(key));
        }

        // Collect all descendant keys for removal (depth-first)
        let to_remove = self.collect_descendants(key);

        // Remove the node from its parent's children list
        if let Some(node) = self.nodes.get(key) {
            if let Some(parent_key) = node.parent {
                if let Some(parent_node) = self.nodes.get_mut(parent_key) {
                    parent_node.children.retain(|&k| k != key);
                }
            }
        }

        // Remove all descendant nodes
        for descendant in &to_remove {
            self.nodes.remove(*descendant);
        }

        // Remove the node itself
        self.nodes.remove(key);

        // Clear root if we removed it
        if self.root == Some(key) {
            self.root = None;
        }

        Ok(())
    }

    /// Returns a reference to the node at the given key.
    pub fn get(&self, key: ComponentKey) -> Option<&ComponentNode> {
        self.nodes.get(key)
    }

    /// Returns a mutable reference to the node at the given key.
    pub fn get_mut(&mut self, key: ComponentKey) -> Option<&mut ComponentNode> {
        self.nodes.get_mut(key)
    }

    /// Returns the root component key, if one exists.
    pub fn root(&self) -> Option<ComponentKey> {
        self.root
    }

    /// Sets the root component key.
    ///
    /// # Errors
    ///
    /// Returns `TreeError::NotFound` if the key does not exist.
    pub fn set_root(&mut self, key: ComponentKey) -> Result<(), TreeError> {
        if !self.nodes.contains_key(key) {
            return Err(TreeError::NotFound(key));
        }
        self.root = Some(key);
        Ok(())
    }

    /// Returns the parent of the given component, if any.
    pub fn parent(&self, key: ComponentKey) -> Option<ComponentKey> {
        self.nodes.get(key).and_then(|node| node.parent)
    }

    /// Returns a list of children for the given component.
    pub fn children(&self, key: ComponentKey) -> Vec<ComponentKey> {
        self.nodes
            .get(key)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    }

    /// Returns an iterator over all `(key, node)` pairs in the tree.
    ///
    /// Iteration order is not guaranteed.
    pub fn iter(&self) -> impl Iterator<Item = (ComponentKey, &ComponentNode)> {
        self.nodes.iter()
    }

    /// Returns the total number of components in the tree.
    pub fn component_count(&self) -> usize {
        self.nodes.len()
    }

    /// Applies a closure to every descendant (children, grandchildren, etc.)
    /// of the given component, visiting them in depth-first pre-order.
    pub fn for_each_descendant(
        &self,
        key: ComponentKey,
        f: &mut impl FnMut(ComponentKey, &ComponentNode),
    ) {
        if let Some(node) = self.nodes.get(key) {
            for &child_key in &node.children {
                if let Some(child) = self.nodes.get(child_key) {
                    f(child_key, child);
                    self.for_each_descendant(child_key, f);
                }
            }
        }
    }

    /// Returns the depth (number of ancestors) of the given component.
    ///
    /// The root has depth 0.
    pub fn depth(&self, key: ComponentKey) -> usize {
        let mut depth = 0;
        let mut current = key;
        while let Some(node) = self.nodes.get(current) {
            match node.parent {
                Some(parent) => {
                    depth += 1;
                    current = parent;
                }
                None => break,
            }
        }
        depth
    }

    /// Returns the path from the given component up to the root.
    ///
    /// The first element is `key` itself; the last is the root.
    pub fn path_to_root(&self, key: ComponentKey) -> Vec<ComponentKey> {
        let mut path = Vec::new();
        let mut current = key;
        loop {
            path.push(current);
            match self.nodes.get(current) {
                Some(node) => match node.parent {
                    Some(parent) => current = parent,
                    None => break,
                },
                None => break,
            }
        }
        path
    }

    // ---- Internal helpers ----

    /// Collects all descendant keys (depth-first) for a given node.
    fn collect_descendants(&self, key: ComponentKey) -> Vec<ComponentKey> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(key) {
            for &child_key in &node.children {
                result.push(child_key);
                result.extend(self.collect_descendants(child_key));
            }
        }
        result
    }
}

impl Default for ComponentTree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::*;
    use crate::coordinate::Coordinate;
    use crate::material::{Material, MaterialType};
    use crate::units::{Quantity, Unit};

    fn test_material() -> Material {
        Material::new(
            "Test",
            MaterialType::Bulk,
            Quantity::new(1000.0, Unit::Kilogram),
        )
    }

    fn make_body_tube(name: &str, x: f64) -> RocketComponent {
        RocketComponent::BodyTube(BodyTubeData {
            name: name.to_string(),
            position: Coordinate::new(x, 0.0, 0.0),
            length: Quantity::new(50.0, Unit::Centimeter),
            outer_radius: Quantity::new(2.0, Unit::Centimeter),
            inner_radius: Quantity::new(1.8, Unit::Centimeter),
            material: test_material(),
            color: None,
            has_motor_mount: false,
        })
    }

    fn make_nose_cone(name: &str, x: f64) -> RocketComponent {
        RocketComponent::NoseCone(NoseConeData {
            name: name.to_string(),
            position: Coordinate::new(x, 0.0, 0.0),
            length: Quantity::new(15.0, Unit::Centimeter),
            base_radius: Quantity::new(2.0, Unit::Centimeter),
            shape: NoseConeShape::Ogive,
            thickness: Quantity::new(0.2, Unit::Centimeter),
            material: test_material(),
            color: None,
            shoulder_length: Quantity::new(2.0, Unit::Centimeter),
            shoulder_radius: Quantity::new(1.8, Unit::Centimeter),
            is_blunted: false,
            blunt_radius: Quantity::new(0.0, Unit::Centimeter),
        })
    }

    fn make_fin_set(name: &str, x: f64) -> RocketComponent {
        RocketComponent::FinSet(FinSetData {
            name: name.to_string(),
            position: Coordinate::new(x, 0.0, 0.0),
            fin_count: 4,
            root_chord: Quantity::new(8.0, Unit::Centimeter),
            tip_chord: Quantity::new(4.0, Unit::Centimeter),
            span: Quantity::new(5.0, Unit::Centimeter),
            sweep_length: Quantity::new(1.5, Unit::Centimeter),
            thickness: Quantity::new(0.3, Unit::Centimeter),
            cross_section: AirfoilType::Square,
            material: test_material(),
            color: None,
            cant_angle: Quantity::new(0.0, Unit::Degree),
            fin_placement: FinPlacement::Normal,
        })
    }

    fn make_parachute(name: &str, x: f64) -> RocketComponent {
        RocketComponent::Parachute(ParachuteData {
            name: name.to_string(),
            position: Coordinate::new(x, 0.0, 0.0),
            diameter: Quantity::new(24.0, Unit::Inch),
            cd: 2.2,
            material: test_material(),
            color: None,
        })
    }

    #[test]
    fn test_empty_tree() {
        let tree = ComponentTree::new();
        assert_eq!(tree.component_count(), 0);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_add_single_component() {
        let mut tree = ComponentTree::new();
        let nose = make_nose_cone("Nose", 0.0);
        let key = tree.add_component(nose, None).unwrap();
        assert_eq!(tree.component_count(), 1);
        assert_eq!(tree.root(), Some(key));
        assert_eq!(tree.depth(key), 0);
    }

    #[test]
    fn test_build_simple_rocket() {
        let mut tree = ComponentTree::new();

        // Root: body tube
        let body_key = tree
            .add_component(make_body_tube("Main Tube", 0.0), None)
            .unwrap();

        // Nose cone attached to body tube
        let nose_key = tree
            .add_component(make_nose_cone("Nose Cone", 0.0), Some(body_key))
            .unwrap();

        // Fin set attached to body tube
        let fin_key = tree
            .add_component(make_fin_set("Fins", 0.0), Some(body_key))
            .unwrap();

        // Parachute attached to body tube
        let chute_key = tree
            .add_component(make_parachute("Main Chute", 0.0), Some(body_key))
            .unwrap();

        assert_eq!(tree.component_count(), 4);
        assert_eq!(tree.root(), Some(body_key));
        assert_eq!(tree.depth(body_key), 0);
        assert_eq!(tree.depth(nose_key), 1);
        assert_eq!(tree.depth(fin_key), 1);
        assert_eq!(tree.depth(chute_key), 1);

        // Verify parent-child relationships
        assert!(tree.parent(nose_key) == Some(body_key));
        assert!(tree.parent(fin_key) == Some(body_key));
        assert!(tree.parent(chute_key) == Some(body_key));
        assert!(tree.parent(body_key).is_none());

        // Verify children
        let children = tree.children(body_key);
        assert_eq!(children.len(), 3);
        assert!(children.contains(&nose_key));
        assert!(children.contains(&fin_key));
        assert!(children.contains(&chute_key));

        // Verify component names
        assert_eq!(tree.get(body_key).unwrap().component.name(), "Main Tube");
        assert_eq!(tree.get(nose_key).unwrap().component.name(), "Nose Cone");
        assert_eq!(tree.get(fin_key).unwrap().component.name(), "Fins");
        assert_eq!(tree.get(chute_key).unwrap().component.name(), "Main Chute");
    }

    #[test]
    fn test_add_component_invalid_parent() {
        let mut tree = ComponentTree::new();
        let missing_key = ComponentKey::default(); // an un-inserted key
        let result = tree.add_component(make_body_tube("Tube", 0.0), Some(missing_key));
        assert!(result.is_err());
        match result {
            Err(TreeError::NotFound(key)) => assert_eq!(key, missing_key),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_remove_leaf_component() {
        let mut tree = ComponentTree::new();
        let body_key = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let nose_key = tree
            .add_component(make_nose_cone("Nose", 0.0), Some(body_key))
            .unwrap();

        assert_eq!(tree.component_count(), 2);

        // Remove leaf node
        tree.remove_component(nose_key).unwrap();
        assert_eq!(tree.component_count(), 1);

        // Verify it's gone from parent's children
        assert!(!tree.children(body_key).contains(&nose_key));
    }

    #[test]
    fn test_remove_internal_component_removes_descendants() {
        let mut tree = ComponentTree::new();
        let body_key = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let pod_key = tree
            .add_component(
                RocketComponent::Pod(PodData {
                    name: "Pod".to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(10.0, Unit::Centimeter),
                    radius: Quantity::new(1.0, Unit::Centimeter),
                    color: None,
                }),
                Some(body_key),
            )
            .unwrap();
        let _inner_key = tree
            .add_component(make_body_tube("Inner", 0.0), Some(pod_key))
            .unwrap();

        assert_eq!(tree.component_count(), 3);

        // Remove pod (internal node) — should also remove inner tube
        tree.remove_component(pod_key).unwrap();
        assert_eq!(tree.component_count(), 1); // only body remains
    }

    #[test]
    fn test_remove_not_found() {
        let mut tree = ComponentTree::new();
        let key = ComponentKey::default();
        let result = tree.remove_component(key);
        assert!(result.is_err());
        match result {
            Err(TreeError::NotFound(k)) => assert_eq!(k, key),
            _ => panic!("Expected NotFound"),
        }
    }

    #[test]
    fn test_set_root() {
        let mut tree = ComponentTree::new();
        let body_key = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let nose_key = tree
            .add_component(make_nose_cone("Nose", 0.0), Some(body_key))
            .unwrap();

        // Root should be body
        assert_eq!(tree.root(), Some(body_key));

        // Set root to nose (should work even though it has a parent)
        tree.set_root(nose_key).unwrap();
        assert_eq!(tree.root(), Some(nose_key));

        // Invalid key
        let result = tree.set_root(ComponentKey::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_depth() {
        let mut tree = ComponentTree::new();
        let body = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        assert_eq!(tree.depth(body), 0);

        let pod = tree
            .add_component(
                RocketComponent::Pod(PodData {
                    name: "Pod".to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(10.0, Unit::Centimeter),
                    radius: Quantity::new(1.0, Unit::Centimeter),
                    color: None,
                }),
                Some(body),
            )
            .unwrap();
        assert_eq!(tree.depth(pod), 1);

        let inner = tree
            .add_component(make_body_tube("Inner", 0.0), Some(pod))
            .unwrap();
        assert_eq!(tree.depth(inner), 2);
    }

    #[test]
    fn test_path_to_root() {
        let mut tree = ComponentTree::new();
        let body = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let pod = tree
            .add_component(
                RocketComponent::Pod(PodData {
                    name: "Pod".to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(10.0, Unit::Centimeter),
                    radius: Quantity::new(1.0, Unit::Centimeter),
                    color: None,
                }),
                Some(body),
            )
            .unwrap();
        let inner = tree
            .add_component(make_body_tube("Inner", 0.0), Some(pod))
            .unwrap();

        let path = tree.path_to_root(inner);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], inner);
        assert_eq!(path[1], pod);
        assert_eq!(path[2], body);

        // Root's path is just itself
        let root_path = tree.path_to_root(body);
        assert_eq!(root_path, vec![body]);
    }

    #[test]
    fn test_for_each_descendant() {
        let mut tree = ComponentTree::new();
        let body = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let _nose = tree
            .add_component(make_nose_cone("Nose", 0.0), Some(body))
            .unwrap();
        let _fin = tree
            .add_component(make_fin_set("Fin", 0.0), Some(body))
            .unwrap();
        let _pod = tree
            .add_component(
                RocketComponent::Pod(PodData {
                    name: "Pod".to_string(),
                    position: Coordinate::origin(),
                    length: Quantity::new(10.0, Unit::Centimeter),
                    radius: Quantity::new(1.0, Unit::Centimeter),
                    color: None,
                }),
                Some(body),
            )
            .unwrap();

        let mut names: Vec<String> = Vec::new();
        tree.for_each_descendant(body, &mut |_, node| {
            names.push(node.component.name().to_string());
        });

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Nose".to_string()));
        assert!(names.contains(&"Fin".to_string()));
        assert!(names.contains(&"Pod".to_string()));
    }

    #[test]
    fn test_iter() {
        let mut tree = ComponentTree::new();
        tree.add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        tree.add_component(make_nose_cone("Nose", 0.0), None)
            .unwrap();

        let count = tree.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_mut() {
        let mut tree = ComponentTree::new();
        let key = tree
            .add_component(make_body_tube("Original", 0.0), None)
            .unwrap();

        // Mutate the component
        if let Some(node) = tree.get_mut(key) {
            if let RocketComponent::BodyTube(ref mut data) = node.component {
                data.name = "Renamed".to_string();
            }
        }

        assert_eq!(tree.get(key).unwrap().component.name(), "Renamed");
    }

    #[test]
    fn test_remove_root_clears_root() {
        let mut tree = ComponentTree::new();
        let key = tree
            .add_component(make_body_tube("Root", 0.0), None)
            .unwrap();
        assert_eq!(tree.root(), Some(key));

        tree.remove_component(key).unwrap();
        assert!(tree.root().is_none());
        assert_eq!(tree.component_count(), 0);
    }

    #[test]
    fn test_serialize_deserialize_component_tree() {
        let mut tree = ComponentTree::new();
        let body = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let _nose = tree
            .add_component(make_nose_cone("Nose", 0.0), Some(body))
            .unwrap();
        let _fin = tree
            .add_component(make_fin_set("Fin", 0.0), Some(body))
            .unwrap();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&tree).expect("Serialization failed");
        assert!(!json.is_empty());
        assert!(json.contains("Body"));
        assert!(json.contains("Nose"));
        assert!(json.contains("Fin"));

        // Deserialize back
        let deserialized: ComponentTree =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.component_count(), 3);
        assert!(deserialized.root().is_some());

        // Verify structure is preserved
        let root = deserialized.root().unwrap();
        assert_eq!(deserialized.get(root).unwrap().component.name(), "Body");
    }

    #[test]
    fn test_serialize_deserialize_roundtrip_empty() {
        let tree = ComponentTree::new();
        let json = serde_json::to_string(&tree).expect("Serialization failed");
        let deserialized: ComponentTree =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.component_count(), 0);
        assert!(deserialized.root().is_none());
    }

    #[test]
    fn test_serialize_deserialize_nested_structure() {
        let mut tree = ComponentTree::new();
        let body = tree
            .add_component(make_body_tube("Body", 0.0), None)
            .unwrap();
        let pod = tree
            .add_component(
                RocketComponent::Pod(PodData {
                    name: "Side Pod".to_string(),
                    position: Coordinate::new(0.1, 0.03, 0.0),
                    length: Quantity::new(20.0, Unit::Centimeter),
                    radius: Quantity::new(1.0, Unit::Centimeter),
                    color: Some("#00FF00".to_string()),
                }),
                Some(body),
            )
            .unwrap();
        tree.add_component(make_nose_cone("Pod Nose", 0.0), Some(pod))
            .unwrap();

        let json = serde_json::to_string_pretty(&tree).unwrap();
        let restored: ComponentTree = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.component_count(), 3);
        let root = restored.root().unwrap();
        assert_eq!(restored.get(root).unwrap().component.name(), "Body");

        // Verify the pod depth
        let restored_pod_key = restored.children(root)[0];
        assert_eq!(restored.depth(restored_pod_key), 1);
        assert_eq!(
            restored.get(restored_pod_key).unwrap().component.name(),
            "Side Pod"
        );

        // Verify pod's child (pod nose)
        let pod_children = restored.children(restored_pod_key);
        assert_eq!(pod_children.len(), 1);
        assert_eq!(
            restored.get(pod_children[0]).unwrap().component.name(),
            "Pod Nose"
        );
        assert_eq!(restored.depth(pod_children[0]), 2);
    }

    #[test]
    fn test_default_impl() {
        let tree: ComponentTree = Default::default();
        assert!(tree.root().is_none());
        assert_eq!(tree.component_count(), 0);
    }
}
