// dom_api.rs - DOM API definitions with JavaScript-style camelCase method names
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use mojes_derive::{js_type, js_object};
use std::collections::HashMap;

/// JavaScript-compatible methods for Rust types
/// This trait provides JavaScript object method names that transpile directly
/// while mapping to appropriate Rust implementations
pub trait MojesMethods<T> {
    /// JavaScript hasOwnProperty() method - checks if object has a property
    /// Maps to HashMap::contains_key() in Rust
    fn hasOwnProperty(&self, key: &str) -> bool;
}

/// Implementation for HashMap to provide JavaScript object-like access
impl<T> MojesMethods<T> for HashMap<String, T> {
    fn hasOwnProperty(&self, key: &str) -> bool {
        // Transpiles to: this.hasOwnProperty(key)
        // Rust implementation: check if key exists
        self.contains_key(key)
    }
}

// JavaScript Promise type - NOT exposed to JS, uses native Promise
#[derive(Debug, Clone)]
pub struct Promise<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Promise<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn resolve(value: T) -> Self {
        println!("Promise.resolve() called");
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn reject(_reason: String) -> Self {
        println!("Promise.reject() called");
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
    
    // Basic Promise methods that would be available in JavaScript
    pub fn then<F>(&self, callback: F) -> Promise<T>
    where
        F: FnOnce(T),
    {
        // In JavaScript, this would be: promise.then(callback)
        Promise::new()
    }
    
    pub fn catch<F>(&self, callback: F) -> Promise<T>
    where
        F: FnOnce(String),
    {
        // In JavaScript, this would be: promise.catch(callback)
        Promise::new()
    }
}

#[linkme::distributed_slice]
pub static JS: [&str];

pub fn Number(s: &str) -> f64 {
    0.0
}

// Core DOM Element representation
// #[js_type]
// DOMTokenList represents classList in the DOM API
// #[js_type] - Browser built-in, do not export
#[derive(Clone, Debug)]
pub struct DOMTokenList {
    pub classes: Vec<String>,
}

impl DOMTokenList {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }
    
    pub fn from_class_string(class_string: &str) -> Self {
        let classes = if class_string.trim().is_empty() {
            Vec::new()
        } else {
            class_string.split_whitespace().map(String::from).collect()
        };
        Self { classes }
    }
    
    // Add a class to the list
    pub fn add(&mut self, class_name: &str) {
        let class_name = class_name.trim();
        if !class_name.is_empty() && !self.classes.contains(&class_name.to_string()) {
            self.classes.push(class_name.to_string());
        }
    }
    
    // Remove a class from the list
    pub fn remove(&mut self, class_name: &str) {
        let class_name = class_name.trim();
        if let Some(pos) = self.classes.iter().position(|c| c == class_name) {
            self.classes.remove(pos);
        }
    }
    
    // Check if a class exists in the list
    pub fn contains(&self, class_name: &str) -> bool {
        self.classes.contains(&class_name.to_string())
    }
    
    // Toggle a class (add if not present, remove if present)
    pub fn toggle(&mut self, class_name: &str) -> bool {
        if self.contains(class_name) {
            self.remove(class_name);
            false
        } else {
            self.add(class_name);
            true
        }
    }
    
    // Replace one class with another
    pub fn replace(&mut self, old_class: &str, new_class: &str) -> bool {
        if let Some(pos) = self.classes.iter().position(|c| c == old_class) {
            self.classes[pos] = new_class.to_string();
            true
        } else {
            false
        }
    }
    
    // Get the number of classes
    pub fn length(&self) -> usize {
        self.classes.len()
    }
    
    // Get class at specific index
    pub fn item(&self, index: usize) -> Option<String> {
        self.classes.get(index).cloned()
    }
    
    // Convert back to string representation
    pub fn to_string(&self) -> String {
        self.classes.join(" ")
    }
    
    // Get all classes as a space-separated string (for className property)
    pub fn value(&self) -> String {
        self.to_string()
    }
}

// Mutable wrapper that keeps Element.className in sync with class changes
pub struct DOMTokenListMut<'a> {
    element: &'a mut Element,
    token_list: DOMTokenList,
}

impl<'a> DOMTokenListMut<'a> {
    pub fn new(element: &'a mut Element) -> Self {
        let token_list = DOMTokenList::from_class_string(&element.className);
        Self { element, token_list }
    }
    
    // Add a class and update the element's className
    pub fn add(&mut self, class_name: &str) {
        self.token_list.add(class_name);
        self.element.className = self.token_list.to_string();
    }
    
    // Remove a class and update the element's className
    pub fn remove(&mut self, class_name: &str) {
        self.token_list.remove(class_name);
        self.element.className = self.token_list.to_string();
    }
    
    // Check if a class exists
    pub fn contains(&self, class_name: &str) -> bool {
        self.token_list.contains(class_name)
    }
    
    // Toggle a class and update the element's className
    pub fn toggle(&mut self, class_name: &str) -> bool {
        let result = self.token_list.toggle(class_name);
        self.element.className = self.token_list.to_string();
        result
    }
    
    // Replace one class with another and update the element's className
    pub fn replace(&mut self, old_class: &str, new_class: &str) -> bool {
        let result = self.token_list.replace(old_class, new_class);
        if result {
            self.element.className = self.token_list.to_string();
        }
        result
    }
    
    // Get the number of classes
    pub fn length(&self) -> usize {
        self.token_list.length()
    }
    
    // Get class at specific index
    pub fn item(&self, index: usize) -> Option<String> {
        self.token_list.item(index)
    }
    
    // Get all classes as a space-separated string
    pub fn value(&self) -> String {
        self.token_list.value()
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub id: String,
    // for inputs
    pub name: String,
    pub tagName: String,
    pub className: String,
    pub classList: DOMTokenList,
    pub innerHTML: String,
    pub innerText: String,
    pub outerHTML: String,
    pub textContent: String,
    pub value: String,
}

impl Element {
    pub fn new(tag_name: &str) -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            tagName: tag_name.to_string(),
            className: String::new(),
            classList: DOMTokenList::new(),
            innerHTML: String::new(),
            innerText: String::new(),
            outerHTML: String::new(),
            textContent: String::new(),
            value: String::new(),
        }
    }

    pub fn with_tag_name(tag_name: &str) -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            tagName: tag_name.to_string(),
            className: String::new(),
            classList: DOMTokenList::new(),
            innerHTML: String::new(),
            innerText: String::new(),
            outerHTML: String::new(),
            textContent: String::new(),
            value: String::new(),
        }
    }
    
    // Helper method to sync classList with className
    pub fn sync_classList_from_className(&mut self) {
        self.classList = DOMTokenList::from_class_string(&self.className);
    }
    
    // Helper method to sync className from classList
    pub fn sync_className_from_classList(&mut self) {
        self.className = self.classList.to_string();
    }

    pub fn getAttribute(&self, _name: &str) -> Option<String> {
        // Mock implementation for transpilation
        None
    }

    pub fn setAttribute(&mut self, _name: &str, _value: &str) {
        // Mock implementation for transpilation
    }

    pub fn removeAttribute(&mut self, _name: &str) {
        // Mock implementation for transpilation
    }

    pub fn hasAttribute(&self, _name: &str) -> bool {
        // Mock implementation for transpilation
        false
    }

    pub fn addEventListener(&self, _event_type: &str, _callback: fn(_event: Event)) {
        // Mock implementation for transpilation
    }

    pub fn removeEventListener(&self, _event_type: &str, _callback: fn()) {
        // Mock implementation for transpilation
    }

    pub fn appendChild(&mut self, _child: Element) {
        // Mock implementation for transpilation
    }

    pub fn removeChild(&mut self, _child: Element) {
        // Mock implementation for transpilation
    }

    pub fn insertBefore(&mut self, _new_node: Element, _reference_node: Option<Element>) {
        // Mock implementation for transpilation
    }

    pub fn replaceChild(&mut self, _new_child: Element, _old_child: Element) {
        // Mock implementation for transpilation
    }

    pub fn insertAdjacentHTML(&mut self, _position: &str, _text: &str) {
        // Mock implementation for transpilation
    }

    pub fn cloneNode(&self, _deep: bool) -> Element {
        self.clone()
    }

    pub fn contains(&self, _other: &Element) -> bool {
        // Mock implementation for transpilation
        false
    }

    pub fn querySelector(&self, _selector: &str) -> Option<Element> {
        Some(Element::with_tag_name("div"))
    }

    pub fn querySelectorAll(&self, _selector: &str) -> Vec<Element> {
        vec![Element::with_tag_name("div")]
    }

    pub fn getElementsByTagName(&self, tag_name: &str) -> Vec<Element> {
        vec![Element::with_tag_name(tag_name)]
    }

    pub fn getElementsByClassName(&self, _class_name: &str) -> Vec<Element> {
        vec![Element::with_tag_name("div")]
    }

    pub fn closest(&self, _selector: &str) -> Option<Element> {
        Some(self.clone())
    }

    pub fn matches(&self, _selector: &str) -> bool {
        // Mock implementation for transpilation
        true
    }

    pub fn focus(&self) {
        // Mock implementation for transpilation
    }

    pub fn blur(&self) {
        // Mock implementation for transpilation
    }

    pub fn click(&self) {
        // Mock implementation for transpilation
    }

    pub fn scrollIntoView(&self, _options: Option<bool>) {
        // Mock implementation for transpilation
    }

    pub fn getBoundingClientRect(&self) -> DOMRect {
        DOMRect::new()
    }

    // WebRTC and Media Element extensions for <video> and <audio> elements
    pub fn srcObject(&self) -> Option<MediaStream> {
        println!("Element.srcObject getter");
        None
    }

    pub fn set_srcObject(&mut self, stream: Option<MediaStream>) {
        println!("Element.set_srcObject()");
        // Mock implementation for transpilation
    }

    // Media element playback methods
    pub fn play(&mut self) -> Result<(), String> {
        println!("Element.play()");
        Ok(())
    }

    pub fn pause(&mut self) {
        println!("Element.pause()");
    }

    pub fn currentTime(&self) -> f64 {
        println!("Element.currentTime getter");
        0.0
    }

    pub fn set_currentTime(&mut self, time: f64) {
        println!("Element.set_currentTime({})", time);
    }

    pub fn duration(&self) -> f64 {
        println!("Element.duration getter");
        0.0
    }

    pub fn muted(&self) -> bool {
        println!("Element.muted getter");
        false
    }

    pub fn set_muted(&mut self, muted: bool) {
        println!("Element.set_muted({})", muted);
    }

    pub fn volume(&self) -> f64 {
        println!("Element.volume getter");
        1.0
    }

    pub fn set_volume(&mut self, volume: f64) {
        println!("Element.set_volume({})", volume);
    }
}

// Document interface
// #[js_type] - Browser built-in, do not export
pub struct Document {
   pub readyState: &'static str,
}

impl Document {
    pub const fn new() -> Self {
        Self { readyState: "ready" }
    }

    pub fn getElementById(&self, id: &str) -> Option<Element> {
        let mut element = Element::with_tag_name("div");
        element.id = id.to_string();
        Some(element)
    }

    pub fn getElementsByTagName(&self, tag_name: &str) -> Vec<Element> {
        vec![Element::with_tag_name(tag_name)]
    }
    pub fn getElementsByName(&self, name: &str) -> Vec<Element> {
        vec![Element::with_tag_name("input")]
    }

    pub fn getElementsByClassName(&self, _class_name: &str) -> Vec<Element> {
        vec![Element::with_tag_name("div")]
    }

    pub fn querySelector(&self, _selector: &str) -> Option<Element> {
        Some(Element::with_tag_name("div"))
    }

    pub fn querySelectorAll(&self, _selector: &str) -> Vec<Element> {
        vec![Element::with_tag_name("div")]
    }

    pub fn createElement(&self, tag_name: &str) -> Element {
        Element::with_tag_name(tag_name)
    }

    pub fn createTextNode(&self, text: &str) -> Element {
        let mut node = Element::with_tag_name("text");
        node.textContent = text.to_string();
        node
    }

    pub fn createDocumentFragment(&self) -> Element {
        Element::with_tag_name("documentFragment")
    }

    pub fn adoptNode(&self, node: Element) -> Element {
        node
    }

    pub fn importNode(&self, node: Element, deep: bool) -> Element {
        if deep {
            node.cloneNode(true)
        } else {
            node.cloneNode(false)
        }
    }

    pub fn write(&self, text: &str) {
        println!("DOCUMENT.WRITE: {}", text);
    }

    pub fn writeln(&self, text: &str) {
        println!("DOCUMENT.WRITELN: {}", text);
    }

    pub fn open(&self) {
        println!("DOCUMENT.OPEN");
    }

    pub fn close(&self) {
        println!("DOCUMENT.CLOSE");
    }

    // Properties as methods for compatibility
    pub fn body(&self) -> Option<Element> {
        Some(Element::with_tag_name("body"))
    }

    pub fn head(&self) -> Option<Element> {
        Some(Element::with_tag_name("head"))
    }

    pub fn documentElement(&self) -> Option<Element> {
        Some(Element::with_tag_name("html"))
    }

    pub fn title(&self) -> String {
        "Mock Document Title".to_string()
    }

    pub fn setTitle(&mut self, title: &str) {
        println!("DOCUMENT.TITLE = {}", title);
    }

    pub fn URL(&self) -> String {
        "http://localhost:3000".to_string()
    }
}

// Console interface
// #[js_type] - Browser built-in, do not export  
pub struct Console {}

impl Console {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn log(&self, message: &str) {
        println!("CONSOLE.LOG: {}", message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("CONSOLE.ERROR: {}", message);
    }

    pub fn warn(&self, message: &str) {
        println!("CONSOLE.WARN: {}", message);
    }

    pub fn info(&self, message: &str) {
        println!("CONSOLE.INFO: {}", message);
    }

    pub fn debug(&self, message: &str) {
        println!("CONSOLE.DEBUG: {}", message);
    }

    pub fn trace(&self) {
        println!("CONSOLE.TRACE: [trace output]");
    }

    pub fn group(&self, label: &str) {
        println!("CONSOLE.GROUP: {}", label);
    }

    pub fn groupEnd(&self) {
        println!("CONSOLE.GROUP_END");
    }

    pub fn time(&self, label: &str) {
        println!("CONSOLE.TIME: {} [timer started]", label);
    }

    pub fn timeEnd(&self, label: &str) {
        println!("CONSOLE.TIME_END: {} [timer ended]", label);
    }

    pub fn clear(&self) {
        println!("CONSOLE.CLEAR");
    }

    pub fn count(&self, label: &str) {
        println!("CONSOLE.COUNT: {}", label);
    }

    pub fn countReset(&self, label: &str) {
        println!("CONSOLE.COUNT_RESET: {}", label);
    }

    pub fn table(&self, data: &str) {
        println!("CONSOLE.TABLE: {}", data);
    }
}

// Window interface
// #[js_type] - Browser built-in, do not export
pub struct Window {}

impl Window {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn alert(&self, message: &str) {
        println!("ALERT: {}", message);
    }

    pub fn confirm(&self, message: &str) -> bool {
        println!("CONFIRM: {}", message);
        true // Mock confirmation
    }

    pub fn prompt(&self, message: &str, _default_value: Option<&str>) -> Option<String> {
        println!("PROMPT: {} (default: {:?})", message, _default_value);
        Some("mock input".to_string())
    }
    /*
        pub fn setTimeout(&self, _callback: fn(), delay: u32) -> u32 {
            println!("SET_TIMEOUT: callback scheduled for {}ms", delay);
            1 // Mock timer ID
        }

        pub fn setInterval(&self, _callback: fn(), delay: u32) -> u32 {
            println!("SET_INTERVAL: callback scheduled every {}ms", delay);
            1 // Mock timer ID
        }
    */
    pub fn setTimeout<F>(&self, callback: F, delay: u32) -> u32
    where
        F: FnOnce() + 'static,
    {
        println!("SET_TIMEOUT: callback scheduled for {}ms", delay);
        1 // Mock timer ID
    }

    pub fn setInterval<F>(&self, callback: F, delay: u32) -> u32
    where
        F: Fn() + 'static, // Note: Fn (not FnOnce) since intervals can fire multiple times
    {
        println!("SET_INTERVAL: callback scheduled every {}ms", delay);
        1 // Mock timer ID
    }

    pub fn clearTimeout(&self, timer_id: u32) {
        println!("CLEAR_TIMEOUT: timer {} cleared", timer_id);
    }

    pub fn clearInterval(&self, timer_id: u32) {
        println!("CLEAR_INTERVAL: timer {} cleared", timer_id);
    }

    /*
    does not accept closures!

        pub fn requestAnimationFrame(&self, _callback: fn()) -> u32 {
            println!("REQUEST_ANIMATION_FRAME: callback scheduled");
            1 // Mock frame ID
        }
    */

    pub fn requestAnimationFrame<F>(&self, callback: F) -> u32
    where
        F: FnOnce() + 'static,
    {
        // Mock implementation - in real browser this would be handled differently
        println!("REQUEST_ANIMATION_FRAME: callback scheduled");
        1
    }

    pub fn cancelAnimationFrame(&self, frame_id: u32) {
        println!("CANCEL_ANIMATION_FRAME: frame {} cancelled", frame_id);
    }

    pub fn getComputedStyle(&self, _element: &Element) -> CSSStyleDeclaration {
        CSSStyleDeclaration::new()
    }

    pub fn scrollTo(&self, x: f64, y: f64) {
        println!("SCROLL_TO: ({}, {})", x, y);
    }

    pub fn scrollBy(&self, x: f64, y: f64) {
        println!("SCROLL_BY: ({}, {})", x, y);
    }

    pub fn open(&self, url: &str, target: &str, _features: &str) -> Option<Window> {
        println!("WINDOW.OPEN: {} in {}", url, target);
        Some(Window::new())
    }

    pub fn close(&self) {
        println!("WINDOW.CLOSE");
    }

    pub fn print(&self) {
        println!("WINDOW.PRINT");
    }

    pub fn focus(&self) {
        println!("WINDOW.FOCUS");
    }

    pub fn blur(&self) {
        println!("WINDOW.BLUR");
    }

    pub fn addEventListener(&self, _event_type: &str, _callback: fn(_event: Event)) {
        // Mock implementation for transpilation
    }

    // Properties as methods
    pub fn innerWidth(&self) -> u32 {
        1920
    }

    pub fn innerHeight(&self) -> u32 {
        1080
    }

    pub fn outerWidth(&self) -> u32 {
        1920
    }

    pub fn outerHeight(&self) -> u32 {
        1080
    }

    pub fn pageXOffset(&self) -> f64 {
        0.0
    }

    pub fn pageYOffset(&self) -> f64 {
        0.0
    }
}

// CSS Style Declaration
// #[js_type] - Browser built-in, do not export
pub struct CSSStyleDeclaration {
    pub color: String,
    pub backgroundColor: String,
    pub fontSize: String,
    pub width: String,
    pub height: String,
    pub margin: String,
    pub padding: String,
    pub border: String,
    pub display: String,
    pub position: String,
    pub top: String,
    pub left: String,
    pub right: String,
    pub bottom: String,
    pub zIndex: String,
    pub opacity: String,
    pub visibility: String,
    pub overflow: String,
}

impl CSSStyleDeclaration {
    pub const fn new() -> Self {
        Self {
            color: String::new(),
            backgroundColor: String::new(),
            fontSize: String::new(),
            width: String::new(),
            height: String::new(),
            margin: String::new(),
            padding: String::new(),
            border: String::new(),
            display: String::new(),
            position: String::new(),
            top: String::new(),
            left: String::new(),
            right: String::new(),
            bottom: String::new(),
            zIndex: String::new(),
            opacity: String::new(),
            visibility: String::new(),
            overflow: String::new(),
        }
    }

    pub fn getPropertyValue(&self, property: &str) -> String {
        match property {
            "color" => self.color.clone(),
            "background-color" => self.backgroundColor.clone(),
            "font-size" => self.fontSize.clone(),
            "width" => self.width.clone(),
            "height" => self.height.clone(),
            "margin" => self.margin.clone(),
            "padding" => self.padding.clone(),
            "border" => self.border.clone(),
            "display" => self.display.clone(),
            "position" => self.position.clone(),
            "top" => self.top.clone(),
            "left" => self.left.clone(),
            "right" => self.right.clone(),
            "bottom" => self.bottom.clone(),
            "z-index" => self.zIndex.clone(),
            "opacity" => self.opacity.clone(),
            "visibility" => self.visibility.clone(),
            "overflow" => self.overflow.clone(),
            _ => String::new(),
        }
    }

    pub fn setProperty(&mut self, property: &str, value: &str) {
        match property {
            "color" => self.color = value.to_string(),
            "background-color" => self.backgroundColor = value.to_string(),
            "font-size" => self.fontSize = value.to_string(),
            "width" => self.width = value.to_string(),
            "height" => self.height = value.to_string(),
            "margin" => self.margin = value.to_string(),
            "padding" => self.padding = value.to_string(),
            "border" => self.border = value.to_string(),
            "display" => self.display = value.to_string(),
            "position" => self.position = value.to_string(),
            "top" => self.top = value.to_string(),
            "left" => self.left = value.to_string(),
            "right" => self.right = value.to_string(),
            "bottom" => self.bottom = value.to_string(),
            "z-index" => self.zIndex = value.to_string(),
            "opacity" => self.opacity = value.to_string(),
            "visibility" => self.visibility = value.to_string(),
            "overflow" => self.overflow = value.to_string(),
            _ => {}
        }
    }

    pub fn removeProperty(&mut self, property: &str) -> String {
        let old_value = self.getPropertyValue(property);
        self.setProperty(property, "");
        old_value
    }
}

// DOMRect interface
// #[js_type] - Browser built-in, do not export
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl DOMRect {
    pub const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            top: 0.0,
            right: 100.0,
            bottom: 100.0,
            left: 0.0,
        }
    }
}

// Location interface
// #[js_type] - Browser built-in, do not export
pub struct Location {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub origin: String,
}

impl Location {
    pub const fn new() -> Self {
        Self {
            href: String::new(),
            protocol: String::new(),
            host: String::new(),
            hostname: String::new(),
            port: String::new(),
            pathname: String::new(),
            search: String::new(),
            hash: String::new(),
            origin: String::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            href: "http://localhost:3000".to_string(),
            protocol: "http:".to_string(),
            host: "localhost:3000".to_string(),
            hostname: "localhost".to_string(),
            port: "3000".to_string(),
            pathname: "/".to_string(),
            search: String::new(),
            hash: String::new(),
            origin: "http://localhost:3000".to_string(),
        }
    }

    pub fn reload(&self) {
        println!("LOCATION.RELOAD: page reloading");
    }

    pub fn assign(&mut self, url: &str) {
        println!("LOCATION.ASSIGN: navigating to {}", url);
        self.href = url.to_string();
    }

    pub fn replace(&mut self, url: &str) {
        println!("LOCATION.REPLACE: replacing with {}", url);
        self.href = url.to_string();
    }

    pub fn toString(&self) -> String {
        self.href.clone()
    }
}

// Navigator interface
// #[js_type] - Browser built-in, do not export
pub struct Navigator {
    pub userAgent: String,
    pub language: String,
    pub platform: String,
    pub cookieEnabled: bool,
    pub onLine: bool,
    pub appName: String,
    pub appVersion: String,
    // WebRTC MediaDevices access as a property
    pub mediaDevices: MediaDevices,
}

impl Navigator {
    pub const fn new() -> Self {
        Self {
            userAgent: String::new(),
            language: String::new(),
            platform: String::new(),
            cookieEnabled: true,
            onLine: true,
            appName: String::new(),
            appVersion: String::new(),
            mediaDevices: MediaDevices,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            userAgent: "Rust-to-JS Transpiler".to_string(),
            language: "en-US".to_string(),
            platform: "Rust".to_string(),
            cookieEnabled: true,
            onLine: true,
            appName: "Rust Browser".to_string(),
            appVersion: "1.0".to_string(),
            mediaDevices: MediaDevices,
        }
    }

    pub fn javaEnabled(&self) -> bool {
        false
    }

    pub fn taintEnabled(&self) -> bool {
        false
    }

}

// History interface
// #[js_type] - Browser built-in, do not export
pub struct History {
    pub length: u32,
    pub state: String,
}

impl History {
    pub const fn new() -> Self {
        Self {
            length: 1,
            state: String::new(),
        }
    }

    pub fn back(&self) {
        println!("HISTORY.BACK: going back");
    }

    pub fn forward(&self) {
        println!("HISTORY.FORWARD: going forward");
    }

    pub fn go(&self, delta: i32) {
        println!("HISTORY.GO: going {} steps", delta);
    }

    pub fn pushState(&mut self, state: &str, title: &str, url: &str) {
        println!("HISTORY.PUSH_STATE: {} -> {}", title, url);
        self.length += 1;
        self.state = state.to_string();
    }

    pub fn replaceState(&mut self, state: &str, title: &str, url: &str) {
        println!("HISTORY.REPLACE_STATE: {} -> {}", title, url);
        self.state = state.to_string();
    }
}

// Event interface
// #[js_type] - Browser built-in, do not export
#[derive(Debug, Clone)]
pub struct Event {
    pub r#type: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub defaultPrevented: bool,
    pub target: Option<Element>,
    pub currentTarget: Option<Element>,
    pub timeStamp: f64,
}

use std::fmt::Display;

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event")
    }
}
impl Event {
    pub fn new(event_type: &str) -> Self {
        Self {
            r#type: event_type.to_string(),
            bubbles: false,
            cancelable: false,
            defaultPrevented: false,
            target: None,
            currentTarget: None,
            timeStamp: 0.0,
        }
    }

    pub fn preventDefault(&self) {
        // Mock implementation for transpilation
        println!("EVENT.PREVENT_DEFAULT: default action prevented");
    }

    pub fn stopPropagation(&self) {
        println!("EVENT.STOP_PROPAGATION: event propagation stopped");
    }

    pub fn stopImmediatePropagation(&self) {
        println!("EVENT.STOP_IMMEDIATE_PROPAGATION: immediate propagation stopped");
    }
}

// MessageEvent interface for WebSocket messages
// #[js_type] - Browser built-in, do not export
#[derive(Clone, Debug)]
pub struct MessageEvent {
    pub data: String,
}

impl MessageEvent {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

// Global instances that will be available in Rust code - using const constructors
pub static document: Document = Document::new();
pub static console: Console = Console::new();
pub static window: Window = Window::new();
pub static location: Location = Location::new();
pub static navigator: Navigator = Navigator::new();
pub static history: History = History::new();

// Helper functions to match JavaScript global functions
pub fn alert(message: &str) {
    window.alert(message);
}

pub fn confirm(message: &str) -> bool {
    window.confirm(message)
}

pub fn prompt(message: &str) -> Option<String> {
    window.prompt(message, None)
}
/*
pub fn setTimeout(callback: fn(), delay: u32) -> u32 {
    window.setTimeout(callback, delay)
}

pub fn setInterval(callback: fn(), delay: u32) -> u32 {
    window.setInterval(callback, delay)
}
*/

pub fn setTimeout<F>(callback: F, delay: u32) -> u32
where
    F: FnOnce() + 'static,
{
    window.setTimeout(callback, delay)
}

pub fn setInterval<F>(callback: F, delay: u32) -> u32
where
    F: Fn() + 'static, // Note: Fn (not FnOnce) since intervals can fire multiple times
{
    window.setInterval(callback, delay)
}

pub fn clearTimeout(timer_id: u32) {
    window.clearTimeout(timer_id)
}

pub fn clearInterval(timer_id: u32) {
    window.clearInterval(timer_id)
}

/*
pub fn requestAnimationFrame(callback: fn()) -> u32 {
    window.requestAnimationFrame(callback)
}
*/
pub fn requestAnimationFrame<F>(callback: F) -> u32
where
    F: FnOnce() + 'static,
{
    window.requestAnimationFrame(callback)
}

pub fn cancelAnimationFrame(frame_id: u32) {
    window.cancelAnimationFrame(frame_id)
}

// Additional DOM utilities
pub fn parseFloat(value: &str) -> f64 {
    value.parse().unwrap_or(0.0)
}

pub fn parseInt(value: &str, radix: Option<u32>) -> i32 {
    let radix = radix.unwrap_or(10);
    i32::from_str_radix(value, radix).unwrap_or(0)
}

pub fn isNaN(value: f64) -> bool {
    value.is_nan()
}

pub fn isFinite(value: f64) -> bool {
    value.is_finite()
}

pub fn encodeURIComponent(uri: &str) -> String {
    // Mock implementation
    uri.to_string()
}

pub fn decodeURIComponent(uri: &str) -> String {
    // Mock implementation
    uri.to_string()
}

// localStorage API implementation for mojes-dom-api/src/lib.rs

use std::sync::Mutex;

// Global mock storage for localStorage (in real implementation this would be browser storage)
lazy_static::lazy_static! {
    static ref LOCAL_STORAGE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// localStorage object that provides Web Storage interface for storing data locally
pub struct LocalStorage;

impl LocalStorage {
    /// Sets the value of the pair identified by key to value, creating a new key/value pair if none existed for key previously.
    ///
    /// # Examples
    /// ```javascript
    /// localStorage.setItem("username", "john_doe");
    /// localStorage.setItem("theme", "dark");
    /// ```
    pub fn setItem(&self, key: &str, value: &str) {
        let mut storage = LOCAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), value.to_string());
    }

    /// Returns the current value associated with the given key, or null if the given key does not exist.
    ///
    /// # Examples
    /// ```javascript
    /// let username = localStorage.getItem("username"); // Some("john_doe")
    /// let missing = localStorage.getItem("nonexistent"); // None
    /// ```
    pub fn getItem(&self, key: &str) -> Option<String> {
        let storage = LOCAL_STORAGE.lock().unwrap();
        storage.get(key).cloned()
    }

    /// Removes the key/value pair with the given key, if a key/value pair with the given key exists.
    ///
    /// # Examples
    /// ```javascript
    /// localStorage.removeItem("username");
    /// ```
    pub fn removeItem(&self, key: &str) {
        let mut storage = LOCAL_STORAGE.lock().unwrap();
        storage.remove(key);
    }

    /// Removes all key/value pairs, if there are any.
    ///
    /// # Examples
    /// ```javascript
    /// localStorage.clear();
    /// ```
    pub fn clear(&self) {
        let mut storage = LOCAL_STORAGE.lock().unwrap();
        storage.clear();
    }

    /// Returns the name of the nth key, or null if n is greater than or equal to the number of key/value pairs.
    ///
    /// # Examples
    /// ```javascript
    /// let first_key = localStorage.key(0); // Some("username")
    /// let invalid = localStorage.key(999); // None
    /// ```
    pub fn key(&self, index: usize) -> Option<String> {
        let storage = LOCAL_STORAGE.lock().unwrap();
        storage.keys().nth(index).cloned()
    }

    /// Returns the number of key/value pairs.
    ///
    /// # Examples
    /// ```javascript
    /// let count = localStorage.length(); // 2
    /// ```
    pub fn length(&self) -> usize {
        let storage = LOCAL_STORAGE.lock().unwrap();
        storage.len()
    }

    /// Store a JSON-serializable value in localStorage
    ///
    /// # Examples
    /// ```javascript
    /// localStorage.setJSON("user_prefs", userPreferences);
    /// ```
    pub fn setJSON<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        match serde_json::to_string(value) {
            Ok(json_string) => {
                self.setItem(key, &json_string);
                Ok(())
            }
            Err(e) => Err(format!("Failed to serialize to JSON: {}", e)),
        }
    }

    /// Retrieve and deserialize a JSON value from localStorage
    ///
    /// # Examples
    /// ```javascript
    /// let userPrefs = localStorage.getJSON("user_prefs");
    /// ```
    pub fn getJSON<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        match self.getItem(key) {
            Some(json_string) => match serde_json::from_str(&json_string) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
            },
            None => Ok(None),
        }
    }
}

/// Global localStorage instance - use this in your code
pub static localStorage: LocalStorage = LocalStorage;

/// sessionStorage object (similar to localStorage but session-scoped)
pub struct SessionStorage;

impl SessionStorage {
    pub fn setItem(&self, key: &str, value: &str) {
        // For now, just use the same storage as localStorage
        // In a real implementation, this would be separate session storage
        localStorage.setItem(key, value);
    }

    pub fn getItem(&self, key: &str) -> Option<String> {
        localStorage.getItem(key)
    }

    pub fn removeItem(&self, key: &str) {
        localStorage.removeItem(key);
    }

    pub fn clear(&self) {
        localStorage.clear();
    }

    pub fn key(&self, index: usize) -> Option<String> {
        localStorage.key(index)
    }

    pub fn length(&self) -> usize {
        localStorage.length()
    }

    pub fn setJSON<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        localStorage.setJSON(key, value)
    }

    pub fn getJSON<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        localStorage.getJSON(key)
    }
}

/// Global sessionStorage instance
pub static sessionStorage: SessionStorage = SessionStorage;

#[cfg(test_not_needed_now)]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_basic_operations() {
        // Clear any existing data
        localStorage.clear();

        // Test setItem and getItem
        localStorage.setItem("test_key", "test_value");
        assert_eq!(
            localStorage.getItem("test_key"),
            Some("test_value".to_string())
        );

        // Test length
        assert_eq!(localStorage.length(), 1);

        // Test key
        assert_eq!(localStorage.key(0), Some("test_key".to_string()));
        assert_eq!(localStorage.key(1), None);

        // Test removeItem
        localStorage.removeItem("test_key");
        assert_eq!(localStorage.getItem("test_key"), None);
        assert_eq!(localStorage.length(), 0);
    }

    #[test]
    fn test_local_storage_multiple_items() {
        localStorage.clear();

        // Add multiple items
        localStorage.setItem("user", "john");
        localStorage.setItem("theme", "dark");
        localStorage.setItem("lang", "en");

        assert_eq!(localStorage.length(), 3);

        // Test getting all keys
        let mut keys = Vec::new();
        for i in 0..localStorage.length() {
            if let Some(key) = localStorage.key(i) {
                keys.push(key);
            }
        }

        assert!(keys.contains(&"user".to_string()));
        assert!(keys.contains(&"theme".to_string()));
        assert!(keys.contains(&"lang".to_string()));

        // Test clear
        localStorage.clear();
        assert_eq!(localStorage.length(), 0);
    }

    #[test]
    fn test_session_storage() {
        sessionStorage.clear();

        sessionStorage.setItem("session_test", "value");
        assert_eq!(
            sessionStorage.getItem("session_test"),
            Some("value".to_string())
        );
        assert_eq!(sessionStorage.length(), 1);

        sessionStorage.removeItem("session_test");
        assert_eq!(sessionStorage.length(), 0);
    }

    #[test]
    fn test_json_helpers() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct User {
            name: String,
            age: u32,
        }

        localStorage.clear();

        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };

        // Store JSON
        localStorage.setJSON("user_data", &user).unwrap();

        // Retrieve JSON
        let retrieved: User = localStorage.getJSON("user_data").unwrap().unwrap();
        assert_eq!(retrieved, user);

        // Test non-existent key
        let missing: Result<Option<User>, String> = localStorage.getJSON("missing");
        assert_eq!(missing.unwrap(), None);
    }
}

// XMLHttpRequest implementation for mojes-dom-api
// XMLHttpRequest implementation for mojes-dom-api with closure handlers
// Add this to your lib.rs file

// use std::collections::HashMap;

// XMLHttpRequest ReadyState constants
pub mod xhr_ready_state {
    pub const UNSENT: u16 = 0;
    pub const OPENED: u16 = 1;
    pub const HEADERS_RECEIVED: u16 = 2;
    pub const LOADING: u16 = 3;
    pub const DONE: u16 = 4;
}

// XMLHttpRequest interface
// #[js_type]
// #[derive(Copy)]
// #[derive(Clone)]
pub struct XMLHttpRequest {
    // Properties
    pub readyState: u16,
    pub response: String,
    pub responseText: String,
    pub responseType: String,
    pub responseURL: String,
    pub responseXML: Option<String>,
    pub status: u16,
    pub statusText: String,
    pub timeout: u32,
    pub upload: Option<XMLHttpRequestUpload>,
    pub withCredentials: bool,

    // Internal state
    method: String,
    url: String,
    async_request: bool,
    headers: HashMap<String, String>,
    request_body: Option<String>,
    /*    // Event handlers using Box<dyn Fn()> for closure support
        onreadystatechange: Option<Box<dyn Fn()>>,
        onload: Option<Box<dyn Fn()>>,
        onerror: Option<Box<dyn Fn()>>,
        onabort: Option<Box<dyn Fn()>>,
        onloadstart: Option<Box<dyn Fn()>>,
        onloadend: Option<Box<dyn Fn()>>,
        onprogress: Option<Box<dyn Fn()>>,
        ontimeout: Option<Box<dyn Fn()>>,
    */
}

impl XMLHttpRequest {
    pub fn new() -> Self {
        Self {
            readyState: xhr_ready_state::UNSENT,
            response: String::new(),
            responseText: String::new(),
            responseType: "text".to_string(),
            responseURL: String::new(),
            responseXML: None,
            status: 0,
            statusText: String::new(),
            timeout: 0,
            upload: None,
            withCredentials: false,

            method: String::new(),
            url: String::new(),
            async_request: true,
            headers: HashMap::new(),
            request_body: None,
            /*
                        onreadystatechange: None,
                        onload: None,
                        onerror: None,
                        onabort: None,
                        onloadstart: None,
                        onloadend: None,
                        onprogress: None,
                        ontimeout: None,
            */
        }
    }

    /// Aborts the request if it has already been sent
    pub fn abort(&mut self) {
        println!("XMLHttpRequest.abort(): Aborting request to {}", self.url);

        if self.readyState != xhr_ready_state::UNSENT && self.readyState != xhr_ready_state::DONE {
            self.readyState = xhr_ready_state::DONE;
            self.status = 0;
            self.statusText = String::new();
            /*
                        // Trigger abort event
                        if let Some(ref callback) = self.onabort {
                            callback();
                        }

                        // Trigger readystatechange
                        if let Some(ref callback) = self.onreadystatechange {
                            callback();
                        }

                        // Trigger loadend
                        if let Some(ref callback) = self.onloadend {
                            callback();
                        }
            */
        }
    }

    /// Returns all response headers as a string
    pub fn getAllResponseHeaders(&self) -> String {
        println!("XMLHttpRequest.getAllResponseHeaders()");

        if self.readyState < xhr_ready_state::HEADERS_RECEIVED {
            return String::new();
        }

        // Mock response headers
        "content-type: application/json\r\ncontent-length: 1234\r\nserver: mock-server\r\n"
            .to_string()
    }

    /// Returns the value of the specified response header
    pub fn getResponseHeader(&self, name: &str) -> Option<String> {
        println!("XMLHttpRequest.getResponseHeader({})", name);

        if self.readyState < xhr_ready_state::HEADERS_RECEIVED {
            return None;
        }

        // Mock implementation - in real browser this would return actual headers
        match name.to_lowercase().as_str() {
            "content-type" => Some("application/json".to_string()),
            "content-length" => Some("1234".to_string()),
            "server" => Some("mock-server".to_string()),
            _ => None,
        }
    }

    /// Initializes a newly-created request, or re-initializes an existing one
    pub fn open(&mut self, method: &str, url: &str) {
        self.open_with_async(method, url, true);
    }

    /// Initializes a request with async parameter
    pub fn open_with_async(&mut self, method: &str, url: &str, async_request: bool) {
        self.open_with_credentials(method, url, async_request, None, None);
    }

    /// Initializes a request with full parameters
    pub fn open_with_credentials(
        &mut self,
        method: &str,
        url: &str,
        async_request: bool,
        user: Option<&str>,
        password: Option<&str>,
    ) {
        println!(
            "XMLHttpRequest.open({}, {}, {}, {:?}, {:?})",
            method, url, async_request, user, password
        );

        // Reset state
        self.readyState = xhr_ready_state::OPENED;
        self.method = method.to_uppercase();
        self.url = url.to_string();
        self.async_request = async_request;
        self.status = 0;
        self.statusText = String::new();
        self.response = String::new();
        self.responseText = String::new();
        self.responseURL = String::new();
        self.headers.clear();

        // Trigger readystatechange
        /*
                if let Some(ref callback) = self.onreadystatechange {
                    callback();
                }
        */
    }

    /// Overrides the MIME type returned by the server
    pub fn overrideMimeType(&mut self, mime_type: &str) {
        println!("XMLHttpRequest.overrideMimeType({})", mime_type);

        if self.readyState != xhr_ready_state::OPENED {
            println!("Warning: overrideMimeType called in invalid state");
            return;
        }

        // In a real implementation, this would affect response parsing
        println!("Overriding MIME type to: {}", mime_type);
    }

    /// Sends the request to the server
    pub fn send(&mut self) {
        self.send_with_body(None);
    }

    /// Sends the request with a body
    pub fn send_with_body(&mut self, body: Option<&str>) {
        println!(
            "XMLHttpRequest.send({:?}) to {} {}",
            body, self.method, self.url
        );

        if self.readyState != xhr_ready_state::OPENED {
            println!("Error: send() called in invalid state");
            return;
        }

        self.request_body = body.map(|s| s.to_string());
        /*
                // Trigger loadstart
                if let Some(ref callback) = self.onloadstart {
                    callback();
                }

        */

        // Mock the request lifecycle
        self.mock_request_lifecycle();
    }

    /// Sets the value of an HTTP request header
    pub fn setRequestHeader(&mut self, header: &str, value: &str) {
        println!("XMLHttpRequest.setRequestHeader({}, {})", header, value);

        if self.readyState != xhr_ready_state::OPENED {
            println!("Error: setRequestHeader called in invalid state");
            return;
        }

        // Check for forbidden headers (in real implementation)
        let forbidden_headers = [
            "accept-charset",
            "accept-encoding",
            "access-control-request-headers",
            "access-control-request-method",
            "connection",
            "content-length",
            "cookie",
            "cookie2",
            "date",
            "dnt",
            "expect",
            "host",
            "keep-alive",
            "origin",
            "referer",
            "te",
            "trailer",
            "transfer-encoding",
            "upgrade",
            "via",
        ];

        let header_lower = header.to_lowercase();
        if forbidden_headers.contains(&header_lower.as_str()) {
            println!("Warning: Attempt to set forbidden header: {}", header);
            return;
        }

        self.headers.insert(header.to_string(), value.to_string());
    }

    // Alternative addEventListener method for more flexibility
    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn() + 'static,
    {
        println!(
            "XMLHttpRequest.addEventListener({}, [function])",
            event_type
        );
        /*
                let boxed_listener = Box::new(listener) as Box<dyn Fn()>;

                match event_type {
                    "readystatechange" => self.onreadystatechange = Some(boxed_listener),
                    "load" => self.onload = Some(boxed_listener),
                    "error" => self.onerror = Some(boxed_listener),
                    "abort" => self.onabort = Some(boxed_listener),
                    "loadstart" => self.onloadstart = Some(boxed_listener),
                    "loadend" => self.onloadend = Some(boxed_listener),
                    "progress" => self.onprogress = Some(boxed_listener),
                    "timeout" => self.ontimeout = Some(boxed_listener),
                    _ => println!("Unknown event type: {}", event_type),
                }
        */
    }

    /// Remove event listener
    pub fn removeEventListener(&mut self, event_type: &str) {
        println!("XMLHttpRequest.removeEventListener({})", event_type);
        /*
                match event_type {
                    "readystatechange" => self.onreadystatechange = None,
                    "load" => self.onload = None,
                    "error" => self.onerror = None,
                    "abort" => self.onabort = None,
                    "loadstart" => self.onloadstart = None,
                    "loadend" => self.onloadend = None,
                    "progress" => self.onprogress = None,
                    "timeout" => self.ontimeout = None,
                    _ => println!("Unknown event type: {}", event_type),
                }
        */
    }

    // Mock implementation of request lifecycle
    fn mock_request_lifecycle(&mut self) {
        // Simulate headers received
        self.readyState = xhr_ready_state::HEADERS_RECEIVED;
        self.status = 200;
        self.statusText = "OK".to_string();
        self.responseURL = self.url.clone();
        /*

                if let Some(ref callback) = self.onreadystatechange {
                    callback();
                }

                // Simulate loading
                self.readyState = xhr_ready_state::LOADING;
                if let Some(ref callback) = self.onreadystatechange {
                    callback();
                }

                if let Some(ref callback) = self.onprogress {
                    callback();
                }

                // Simulate completion
                self.readyState = xhr_ready_state::DONE;
                self.response = r#"{"message": "Mock response", "status": "success"}"#.to_string();
                self.responseText = self.response.clone();

                if let Some(ref callback) = self.onreadystatechange {
                    callback();
                }

                if let Some(ref callback) = self.onload {
                    callback();
                }

                if let Some(ref callback) = self.onloadend {
                    callback();
                }
        */
    }
}

// XMLHttpRequestUpload interface for upload progress tracking
// #[js_type] - Browser built-in, do not export
pub struct XMLHttpRequestUpload {
    // Event handlers for upload events using Box<dyn Fn()>
    onloadstart: Option<Box<dyn Fn()>>,
    onload: Option<Box<dyn Fn()>>,
    onloadend: Option<Box<dyn Fn()>>,
    onprogress: Option<Box<dyn Fn()>>,
    onerror: Option<Box<dyn Fn()>>,
    onabort: Option<Box<dyn Fn()>>,
    ontimeout: Option<Box<dyn Fn()>>,
}

impl XMLHttpRequestUpload {
    pub fn new() -> Self {
        Self {
            onloadstart: None,
            onload: None,
            onloadend: None,
            onprogress: None,
            onerror: None,
            onabort: None,
            ontimeout: None,
        }
    }

    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn() + 'static,
    {
        println!(
            "XMLHttpRequestUpload.addEventListener({}, [function])",
            event_type
        );

        let boxed_listener = Box::new(listener) as Box<dyn Fn()>;

        match event_type {
            "loadstart" => self.onloadstart = Some(boxed_listener),
            "load" => self.onload = Some(boxed_listener),
            "loadend" => self.onloadend = Some(boxed_listener),
            "progress" => self.onprogress = Some(boxed_listener),
            "error" => self.onerror = Some(boxed_listener),
            "abort" => self.onabort = Some(boxed_listener),
            "timeout" => self.ontimeout = Some(boxed_listener),
            _ => println!("Unknown upload event type: {}", event_type),
        }
    }

    pub fn removeEventListener(&mut self, event_type: &str) {
        println!("XMLHttpRequestUpload.removeEventListener({})", event_type);

        match event_type {
            "loadstart" => self.onloadstart = None,
            "load" => self.onload = None,
            "loadend" => self.onloadend = None,
            "progress" => self.onprogress = None,
            "error" => self.onerror = None,
            "abort" => self.onabort = None,
            "timeout" => self.ontimeout = None,
            _ => println!("Unknown upload event type: {}", event_type),
        }
    }
}

// ProgressEvent interface for progress tracking
// #[js_type] - Browser built-in, do not export
pub struct ProgressEvent {
    pub lengthComputable: bool,
    pub loaded: u64,
    pub total: u64,
    pub target: Option<XMLHttpRequest>,
}

impl ProgressEvent {
    pub fn new(length_computable: bool, loaded: u64, total: u64) -> Self {
        Self {
            lengthComputable: length_computable,
            loaded,
            total,
            target: None,
        }
    }
}

// Global factory function to create XMLHttpRequest instances
pub fn create_xhr() -> XMLHttpRequest {
    XMLHttpRequest::new()
}

// Alternative constructor that matches JavaScript's new XMLHttpRequest()
impl Default for XMLHttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test_not_needed_now)]
mod tests_xhr {
    use super::*;

    #[test]
    fn test_xhr_lifecycle() {
        let mut xhr = XMLHttpRequest::new();

        // Initial state
        assert_eq!(xhr.readyState, xhr_ready_state::UNSENT);

        // Open request
        xhr.open("GET", "https://api.example.com/data");
        assert_eq!(xhr.readyState, xhr_ready_state::OPENED);
        assert_eq!(xhr.method, "GET");
        assert_eq!(xhr.url, "https://api.example.com/data");

        // Set headers
        xhr.setRequestHeader("Content-Type", "application/json");
        xhr.setRequestHeader("Authorization", "Bearer token123");

        // Send request
        xhr.send();

        // After mock lifecycle, should be DONE
        assert_eq!(xhr.readyState, xhr_ready_state::DONE);
        assert_eq!(xhr.status, 200);
        assert_eq!(xhr.statusText, "OK");
        assert!(!xhr.responseText.is_empty());
    }

    /* fails
        #[test]
        fn test_xhr_abort() {
            let mut xhr = XMLHttpRequest::new();
            xhr.open("POST", "https://api.example.com/upload");
            xhr.send_with_body(Some(r#"{"data": "test"}"#));

            // Abort the request
            xhr.abort();
            assert_eq!(xhr.readyState, xhr_ready_state::DONE);
            assert_eq!(xhr.status, 0);
        }
    */

    #[test]
    fn test_xhr_headers() {
        let xhr = XMLHttpRequest::new();

        // Before headers received
        assert!(xhr.getResponseHeader("content-type").is_none());
        assert!(xhr.getAllResponseHeaders().is_empty());
    }
}

// WebSocket implementation
#[derive(Clone, Debug)]
pub struct WebSocket {
    pub url: String,
}

impl WebSocket {
    pub fn new(url: &str) -> Result<Self, String> {
        // Mock implementation for transpilation
        Ok(Self { url: url.to_string() })
    }

    pub fn send(&self, data: &str) {
        // Mock implementation for transpilation
        println!("WebSocket.send: {}", data);
    }

    pub fn close(&self) {
        // Mock implementation for transpilation
        println!("WebSocket.close");
    }

    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn(MessageEvent) + 'static,
    {
        // Mock implementation for transpilation
    }
}

// =============================================================================
// WebRTC API Implementation
// =============================================================================

// Core WebRTC Configuration Types
#[js_type] // Safe to export - configuration dictionary, not a browser constructor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
    pub credential_type: Option<String>, // "password", "oauth"
}

#[js_object] // Safe to export - helper constructors for configuration dictionary
impl RTCIceServer {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            urls,
            username: None,
            credential: None,
            credential_type: None,
        }
    }

    pub fn with_credentials(urls: Vec<String>, username: String, credential: String) -> Self {
        Self {
            urls,
            username: Some(username),
            credential: Some(credential),
            credential_type: Some("password".to_string()),
        }
    }
}

#[js_type] // Safe to export - configuration dictionary, not a browser constructor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCConfiguration {
    pub iceServers: Vec<RTCIceServer>,
    pub iceTransportPolicy: Option<String>, // "all", "relay"
    pub bundlePolicy: Option<String>, // "balanced", "max-compat", "max-bundle"
    pub rtcpMuxPolicy: Option<String>, // "negotiate", "require"
    pub iceCandidatePoolSize: Option<u32>, 
}

#[js_object] // Safe to export - helper constructors for configuration dictionary
impl RTCConfiguration {
    pub fn new() -> Self {
        Self {
            iceServers: vec![],
            iceTransportPolicy: Some("all".to_string()),
            bundlePolicy: Some("balanced".to_string()),
            rtcpMuxPolicy: Some("require".to_string()),
            iceCandidatePoolSize: Some(50),
        }
    }

    pub fn with_ice_servers(ice_servers: Vec<RTCIceServer>) -> Self {
        Self {
            iceServers: ice_servers,
            iceTransportPolicy: Some("all".to_string()),
            bundlePolicy: Some("balanced".to_string()),
            rtcpMuxPolicy: Some("require".to_string()),
            iceCandidatePoolSize: Some(50),
        }
    }
}

// Session Description Types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCSessionDescription {
    pub r#type: String, // "offer", "answer", "pranswer", "rollback"
    pub sdp: String,   // SDP content
}

impl RTCSessionDescription {
    pub fn new(r#type: String, sdp: String) -> Self {
        Self { r#type, sdp }
    }
}

// ICE Candidate Types
#[js_type] // Configuration dictionary - safe to export
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCIceCandidateInit {
    pub candidate: String,
    pub sdpMid: Option<String>,
    pub sdpMLineIndex: Option<u16>,
}

#[js_object] // Configuration dictionary methods - safe to export
impl RTCIceCandidateInit {
    pub fn new(candidate: String) -> Self {
        Self {
            candidate,
            sdpMid: Some("0".to_string()), // Default to first media line
            sdpMLineIndex: Some(0), // Default to first media line index
        }
    }
    
    pub fn with_sdp_mid(candidate: String, sdp_mid: String) -> Self {
        Self {
            candidate,
            sdpMid: Some(sdp_mid),
            sdpMLineIndex: None,
        }
    }
    
    pub fn with_mline_index(candidate: String, sdp_mline_index: u16) -> Self {
        Self {
            candidate,
            sdpMid: None,
            sdpMLineIndex: Some(sdp_mline_index),
        }
    }
}

// #[js_type] - Browser built-in WebRTC type, do not export
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCIceCandidate {
    pub candidate: String,        // ICE candidate string
    pub sdpMid: Option<String>,   // Media stream identification
    pub sdpMLineIndex: Option<u16>, // Media line index
    pub foundation: Option<String>,
    pub component: Option<u16>,
    pub priority: Option<u32>,
    pub address: Option<String>,
    pub protocol: Option<String>, // "udp", "tcp"
    pub port: Option<u16>,
    pub r#type: Option<String>,    // "host", "srflx", "prflx", "relay"
}

// #[js_object] - Browser built-in constructor, do not export
impl RTCIceCandidate {
    pub fn new(init: RTCIceCandidateInit) -> Self {
        Self {
            candidate: init.candidate,
            sdpMid: init.sdpMid,
            sdpMLineIndex: init.sdpMLineIndex,
            foundation: None,
            component: None,
            priority: None,
            address: None,
            protocol: None,
            port: None,
            r#type: None,
        }
    }
    
    // Convert back to RTCIceCandidateInit for browser compatibility
    pub fn to_init(&self) -> RTCIceCandidateInit {
        RTCIceCandidateInit {
            candidate: self.candidate.clone(),
            sdpMid: self.sdpMid.clone(),
            sdpMLineIndex: self.sdpMLineIndex,
        }
    }
}

// Media Stream Types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaStream {
    pub id: String,
    pub active: bool,
}

impl MediaStream {
    pub fn new() -> Self {
        Self {
            id: "mock-stream-id".to_string(),
            active: true,
        }
    }

    pub fn getTracks(&self) -> Vec<MediaStreamTrack> {
        println!("MediaStream.getTracks()");
        vec![]
    }

    pub fn getAudioTracks(&self) -> Vec<MediaStreamTrack> {
        println!("MediaStream.getAudioTracks()");
        vec![]
    }

    pub fn getVideoTracks(&self) -> Vec<MediaStreamTrack> {
        println!("MediaStream.getVideoTracks()");
        vec![]
    }

    pub fn addTrack(&mut self, track: MediaStreamTrack) {
        println!("MediaStream.addTrack({})", track.id);
    }

    pub fn removeTrack(&mut self, track: MediaStreamTrack) {
        println!("MediaStream.removeTrack({})", track.id);
    }

    pub fn clone(&self) -> Self {
        Self {
            id: format!("{}-clone", self.id),
            active: self.active,
        }
    }

    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn(Event) + 'static,
    {
        println!("MediaStream.addEventListener({})", event_type);
        // Events: "addtrack", "removetrack"
    }
}

#[derive(Debug, Clone)]
pub struct MediaStreamTrack {
    pub id: String,
    pub kind: String,     // "audio" or "video"
    pub label: String,
    pub enabled: bool,
    pub muted: bool,
    pub readOnly: bool,
    pub readyState: String, // "live", "ended"
}

impl MediaStreamTrack {
    pub fn new(kind: &str) -> Self {
        Self {
            id: format!("mock-{}-track", kind),
            kind: kind.to_string(),
            label: format!("Mock {} Track", kind),
            enabled: true,
            muted: false,
            readOnly: false,
            readyState: "live".to_string(),
        }
    }

    pub fn stop(&mut self) {
        println!("MediaStreamTrack.stop()");
        self.readyState = "ended".to_string();
    }

    pub fn clone(&self) -> Self {
        Self {
            id: format!("{}-clone", self.id),
            kind: self.kind.clone(),
            label: self.label.clone(),
            enabled: self.enabled,
            muted: self.muted,
            readOnly: self.readOnly,
            readyState: self.readyState.clone(),
        }
    }

    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn(Event) + 'static,
    {
        println!("MediaStreamTrack.addEventListener({})", event_type);
        // Events: "ended", "mute", "unmute"
    }
}

// Media Constraints Types
#[js_type] // Safe to export - configuration dictionary, not a browser constructor
#[derive(Debug, Clone)]
pub enum MediaTrackConstraints {
    Bool(bool),
    Object {
        width: Option<u32>,
        height: Option<u32>,
        frame_rate: Option<f64>,
        device_id: Option<String>,
    }
}

impl MediaTrackConstraints {
    pub fn new_bool(enabled: bool) -> Self {
        Self::Bool(enabled)
    }

    pub fn new_video(width: Option<u32>, height: Option<u32>) -> Self {
        Self::Object {
            width,
            height,
            frame_rate: None,
            device_id: None,
        }
    }
}

#[js_type] // Safe to export - configuration dictionary, not a browser constructor  
#[derive(Debug, Clone)]
pub struct MediaStreamConstraints {
    pub video: MediaTrackConstraints,
    pub audio: MediaTrackConstraints,
}

impl MediaStreamConstraints {
    pub fn new() -> Self {
        Self {
            video: MediaTrackConstraints::Bool(false),
            audio: MediaTrackConstraints::Bool(false),
        }
    }

    pub fn video_audio() -> Self {
        Self {
            video: MediaTrackConstraints::Bool(true),
            audio: MediaTrackConstraints::Bool(true),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaDeviceInfo {
    pub device_id: String,
    pub kind: String,     // "audioinput", "audiooutput", "videoinput"
    pub label: String,
    pub group_id: String,
}

impl MediaDeviceInfo {
    pub fn new(device_id: String, kind: String, label: String) -> Self {
        Self {
            device_id,
            kind,
            label,
            group_id: "mock-group".to_string(),
        }
    }
}

pub struct MediaDevices;

impl MediaDevices {
    pub fn getUserMedia(&self, constraints: &MediaStreamConstraints) -> Promise<MediaStream> {
        // This will transpile to: navigator.mediaDevices.getUserMedia(constraints)
        // which returns a Promise<MediaStream> in JavaScript
        Promise::new()
    }

    pub fn getDisplayMedia(&self, constraints: &MediaStreamConstraints) -> Result<MediaStream, String> {
        println!("MediaDevices.getDisplayMedia()");
        Ok(MediaStream::new())
    }

    pub fn enumerateDevices(&self) -> Result<Vec<MediaDeviceInfo>, String> {
        println!("MediaDevices.enumerateDevices()");
        Ok(vec![
            MediaDeviceInfo::new("mock-audio-input".to_string(), "audioinput".to_string(), "Mock Microphone".to_string()),
            MediaDeviceInfo::new("mock-video-input".to_string(), "videoinput".to_string(), "Mock Camera".to_string()),
        ])
    }
}

// RTP Sender/Receiver Types
#[derive(Debug, Clone)]
pub struct RTCRtpSender {
    pub track: Option<MediaStreamTrack>,
}

impl RTCRtpSender {
    pub fn new(track: Option<MediaStreamTrack>) -> Self {
        Self { track }
    }

    pub async fn replaceTrack(&mut self, track: Option<MediaStreamTrack>) -> Result<(), String> {
        println!("RTCRtpSender.replaceTrack()");
        self.track = track;
        Ok(())
    }

    pub async fn getStats(&self) -> Result<RTCStatsReport, String> {
        println!("RTCRtpSender.getStats()");
        Ok(RTCStatsReport::new())
    }
}

#[derive(Debug, Clone)]
pub struct RTCRtpReceiver {
    pub track: MediaStreamTrack,
}

impl RTCRtpReceiver {
    pub fn new(track: MediaStreamTrack) -> Self {
        Self { track }
    }
}

#[derive(Debug, Clone)]
pub struct RTCRtpTransceiver {
    pub sender: RTCRtpSender,
    pub receiver: RTCRtpReceiver,
    pub direction: String, // "sendrecv", "sendonly", "recvonly", "inactive"
}

impl RTCRtpTransceiver {
    pub fn new(sender: RTCRtpSender, receiver: RTCRtpReceiver) -> Self {
        Self {
            sender,
            receiver,
            direction: "sendrecv".to_string(),
        }
    }
}

// Stats API Types
#[derive(Debug, Clone)]
pub struct RTCStats {
    pub id: String,
    pub timestamp: f64,
    pub r#type: String, // "inbound-rtp", "outbound-rtp", "candidate-pair", etc.
}

impl RTCStats {
    pub fn new(id: String, r#type: String) -> Self {
        Self {
            id,
            timestamp: 0.0,
            r#type,
        }
    }
}

pub struct RTCStatsReport {
    // Map-like interface for stats
}

impl RTCStatsReport {
    pub fn new() -> Self {
        Self { }
    }

    pub fn get(&self, id: &str) -> Option<RTCStats> {
        println!("RTCStatsReport.get({})", id);
        None
    }

    pub fn values(&self) -> Vec<RTCStats> {
        println!("RTCStatsReport.values()");
        vec![]
    }
}

// WebRTC-specific Event Types
// #[js_type] - Browser built-in, do not export
#[derive(Debug, Clone)]
pub struct RTCPeerConnectionIceEvent {
    pub candidate: Option<RTCIceCandidate>,
    pub url: Option<String>, // ICE server URL (deprecated but may still be present)
    // Inherits from Event
    pub r#type: String,
    pub target: Option<Element>,
}

impl RTCPeerConnectionIceEvent {
    pub fn new(candidate: Option<RTCIceCandidate>) -> Self {
        Self {
            candidate,
            url: None, // Usually None unless specified
            r#type: "icecandidate".to_string(),
            target: None,
        }
    }
}

// #[js_type] - Browser built-in, do not export
#[derive(Debug, Clone)]
pub struct RTCTrackEvent {
    pub receiver: RTCRtpReceiver,
    pub track: MediaStreamTrack,
    pub streams: Vec<MediaStream>,
    pub transceiver: Option<RTCRtpTransceiver>,
    // Inherits from Event
    pub r#type: String,
    pub target: Option<Element>,
}

impl RTCTrackEvent {
    pub fn new(receiver: RTCRtpReceiver, track: MediaStreamTrack, streams: Vec<MediaStream>) -> Self {
        Self {
            receiver,
            track,
            streams,
            transceiver: None,
            r#type: "track".to_string(),
            target: None,
        }
    }
}

// #[js_type] - Browser built-in, do not export
// NOTE: This conflates two different JavaScript events for Rust convenience:
// 1. "datachannel" event on RTCPeerConnection (has channel property)
// 2. "message" event on RTCDataChannel (uses MessageEvent with data property)
// In practice, we need one type for addEventListener() since Rust can't have different
// event types for different event names on the same method.
#[derive(Debug, Clone)]
pub struct RTCDataChannelEvent {
    pub channel: RTCDataChannel, // For "datachannel" event on RTCPeerConnection
    pub data: String,           // For "message" event on RTCDataChannel (MessageEvent.data)
    // Inherits from Event
    pub r#type: String,
    pub target: Option<Element>,
}

impl RTCDataChannelEvent {
    pub fn new(channel: RTCDataChannel) -> Self {
        Self {
            channel,
            data: String::new(),
            r#type: "datachannel".to_string(),
            target: None,
        }
    }
    
    pub fn new_with_data(channel: RTCDataChannel, data: String) -> Self {
        Self {
            channel,
            data,
            r#type: "message".to_string(),
            target: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RTCDataChannel {
    pub label: String,
    pub ordered: bool,
    pub maxRetransmits: Option<u16>,
    pub maxPacketLifeTime: Option<u16>,
    pub protocol: String,
    pub negotiated: bool,
    pub id: Option<u16>,
    pub readyState: String, // "connecting", "open", "closing", "closed"
}

impl RTCDataChannel {
    pub fn new(label: String) -> Self {
        Self {
            label,
            ordered: true,
            maxRetransmits: None,
            maxPacketLifeTime: None,
            protocol: String::new(),
            negotiated: false,
            id: None,
            readyState: "connecting".to_string(),
        }
    }

    pub fn send(&self, data: &str) -> Result<(), String> {
        println!("RTCDataChannel.send({})", data);
        Ok(())
    }

    pub fn close(&mut self) {
        println!("RTCDataChannel.close()");
        self.readyState = "closed".to_string();
    }

    pub fn addEventListener<F>(&mut self, event_type: &str, listener: F)
    where
        F: Fn(RTCDataChannelEvent) + 'static,
    {
        println!("RTCDataChannel.addEventListener({})", event_type);
        // Events: "open", "message", "error", "close"
    }
}

// RTCPeerConnection - The main WebRTC interface  
// #[js_type] - Browser built-in WebRTC type, do not export
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RTCPeerConnection {
    // Core state properties
    pub connectionState: String, // "new", "connecting", "connected", "disconnected", "failed", "closed"
    pub iceConnectionState: String,
    pub signalingState: String,
    pub iceGatheringState: String, // "new", "gathering", "complete"
}

// WebRTC event struct - superset of all possible WebRTC event properties
// #[js_type] - Internal Rust workaround, do not export to avoid browser conflicts
#[derive(Debug, Clone, Default)]
pub struct RTCPeerConnectionEvent {
      // For icecandidate events (RTCPeerConnectionIceEvent)
      pub candidate: Option<RTCIceCandidate>,
      pub url: Option<String>, // ICE server URL (deprecated)

      // For track events (RTCTrackEvent)
      pub receiver: Option<RTCRtpReceiver>,
      pub streams: Option<Vec<MediaStream>>,
      pub track: Option<MediaStreamTrack>,
      pub transceiver: Option<RTCRtpTransceiver>,

      // For datachannel events (RTCDataChannelEvent) 
      pub channel: Option<RTCDataChannel>,

      // For message events on DataChannel (DataChannelMessageEvent)
      pub data: Option<String>,

      // Common Event properties
      pub r#type: Option<String>,
      pub target: Option<Element>,
  }


impl RTCPeerConnection {
    pub fn new(configuration: &RTCConfiguration) -> Self {
        println!("RTCPeerConnection.new() with {} ICE servers", configuration.iceServers.len());
        Self {
            connectionState: "new".to_string(),
            iceConnectionState: "new".to_string(),
            signalingState: "stable".to_string(),
            iceGatheringState: "new".to_string(),
        }
    }

    // Session description methods - return Promises in real JavaScript
    pub fn createOffer(&self) -> Promise<RTCSessionDescription> {
        println!("RTCPeerConnection.createOffer()");
        Promise::new()
    }

    pub fn createAnswer(&self) -> Promise<RTCSessionDescription> {
        println!("RTCPeerConnection.createAnswer()");
        Promise::new()
    }

    pub fn setLocalDescription(&self, description: &RTCSessionDescription) -> Promise<()> {
        println!("RTCPeerConnection.setLocalDescription({})", description.r#type);
        // Note: In real WebRTC, this would update the connection state
        // but for Mojes transpilation, the browser handles state changes
        Promise::new()
    }

    pub fn setRemoteDescription(&self, description: &RTCSessionDescription) -> Promise<()> {
        println!("RTCPeerConnection.setRemoteDescription({})", description.r#type);
        // Note: In real WebRTC, this would update the connection state
        // but for Mojes transpilation, the browser handles state changes
        Promise::new()
    }

    // ICE candidate methods
    pub fn addIceCandidate(&self, candidate: &RTCIceCandidateInit) -> Promise<()> {
        println!("RTCPeerConnection.addIceCandidate({:?})", candidate);
        Promise::new()
    }

    // Media track methods
    pub fn addTrack(&self, track: MediaStreamTrack, stream: MediaStream) -> RTCRtpSender {
        println!("RTCPeerConnection.addTrack({}, {})", track.id, stream.id);
        RTCRtpSender::new(Some(track))
    }

    pub fn removeTrack(&mut self, sender: &RTCRtpSender) -> Result<(), String> {
        println!("RTCPeerConnection.removeTrack()");
        Ok(())
    }

    // Data channel methods
    pub fn createDataChannel(&self, label: &str) -> RTCDataChannel {
        println!("RTCPeerConnection.createDataChannel({})", label);
        RTCDataChannel::new(label.to_string())
    }

    // Event handlers (addEventListener with specific event types)
    pub fn addEventListener<F>(&self, event_type: &str, listener: F)
    where
        F: Fn(RTCPeerConnectionEvent) + 'static,
    {
        println!("RTCPeerConnection.addEventListener({})", event_type);
        // Events: "icecandidate", "track", "connectionstatechange", "icecandidateerror"
    }

    // Connection management
    pub fn close(&mut self) {
        println!("RTCPeerConnection.close()");
        self.connectionState = "closed".to_string();
        self.iceConnectionState = "closed".to_string();
        self.signalingState = "closed".to_string();
        self.iceGatheringState = "complete".to_string();
    }
}
