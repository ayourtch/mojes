use mojes_dom_api::*;

#[test]
fn test_classList_basic_operations() {
    let mut element = Element::with_tag_name("div");
    element.className = "initial existing".to_string();
    
    // Sync classList from className
    element.sync_classList_from_className();
    
    // Test reading classList as field
    assert_eq!(element.classList.length(), 2);
    assert!(element.classList.contains("initial"));
    assert!(element.classList.contains("existing"));
    assert!(!element.classList.contains("missing"));
    
    // Test modification through direct field access
    element.classList.add("new-class");
    assert!(element.classList.contains("new-class"));
    assert_eq!(element.classList.length(), 3);
    
    // Remove a class
    element.classList.remove("existing");
    assert!(!element.classList.contains("existing"));
    assert_eq!(element.classList.length(), 2);
    
    // Toggle a class (should add it)
    let result = element.classList.toggle("toggled");
    assert!(result); // Should return true when adding
    assert!(element.classList.contains("toggled"));
    
    // Toggle the same class again (should remove it)
    let result = element.classList.toggle("toggled");
    assert!(!result); // Should return false when removing
    assert!(!element.classList.contains("toggled"));
    
    // Replace a class
    let result = element.classList.replace("initial", "replaced");
    assert!(result); // Should return true when replacement succeeds
    assert!(element.classList.contains("replaced"));
    assert!(!element.classList.contains("initial"));
    
    // Sync className from classList
    element.sync_className_from_classList();
    
    // Verify that className property was updated
    assert_eq!(element.className, "replaced new-class");
}

#[test]
fn test_classList_edge_cases() {
    let mut element = Element::with_tag_name("div");
    
    // Test with empty className
    element.className = "".to_string();
    element.sync_classList_from_className();
    assert_eq!(element.classList.length(), 0);
    
    // Test adding to empty classList
    element.classList.add("first-class");
    assert_eq!(element.classList.length(), 1);
    element.sync_className_from_classList();
    assert_eq!(element.className, "first-class");
    
    // Test duplicate addition (should not add duplicates)
    element.classList.add("first-class"); // Should not add duplicate
    assert_eq!(element.classList.length(), 1);
    element.sync_className_from_classList();
    assert_eq!(element.className, "first-class");
    
    // Test removing non-existent class
    element.classList.remove("non-existent");
    assert_eq!(element.classList.length(), 1); // Should remain unchanged
    element.sync_className_from_classList();
    assert_eq!(element.className, "first-class");
}

#[test]
fn test_classList_whitespace_handling() {
    let mut element = Element::with_tag_name("div");
    element.className = "  spaced   classes  with   whitespace  ".to_string();
    element.sync_classList_from_className();
    
    assert_eq!(element.classList.length(), 4);
    assert!(element.classList.contains("spaced"));
    assert!(element.classList.contains("classes"));
    assert!(element.classList.contains("with"));
    assert!(element.classList.contains("whitespace"));
    
    // Test that adding preserves proper spacing
    element.classList.add("new");
    element.sync_className_from_classList();
    
    // Should clean up to proper spacing
    assert_eq!(element.className, "spaced classes with whitespace new");
}

#[test]
fn test_classList_item_access() {
    let mut element = Element::with_tag_name("div");
    element.className = "first second third".to_string();
    element.sync_classList_from_className();
    
    assert_eq!(element.classList.item(0), Some("first".to_string()));
    assert_eq!(element.classList.item(1), Some("second".to_string()));
    assert_eq!(element.classList.item(2), Some("third".to_string()));
    assert_eq!(element.classList.item(3), None); // Out of bounds
}

#[test]
fn test_classList_value() {
    let mut element = Element::with_tag_name("div");
    element.className = "one two three".to_string();
    element.sync_classList_from_className();
    
    assert_eq!(element.classList.value(), "one two three");
    assert_eq!(element.classList.to_string(), "one two three");
}

/*
This test suite verifies that the classList implementation matches JavaScript behavior:

1. Reading classes from className
2. Adding classes without duplicates
3. Removing classes
4. Toggling classes (returns true when adding, false when removing)
5. Replacing classes
6. Proper synchronization with Element.className
7. Handling empty className
8. Handling whitespace in className
9. Item access by index
10. Value/toString functionality

The implementation provides:
- DOMTokenList: Read-only view of classes
- DOMTokenListMut: Mutable wrapper that keeps className in sync
- Full JavaScript classList API compatibility
*/