// dom_api.rs - DOM API definitions with JavaScript-style camelCase method names
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use mojes_derive::js_type;

#[linkme::distributed_slice]
pub static JS: [&str];

// Core DOM Element representation
#[js_type]
#[derive(Clone, Debug)]
pub struct Element {
    pub id: String,
    pub tagName: String,
    pub className: String,
    pub innerHTML: String,
    pub innerText: String,
    pub outerHTML: String,
    pub textContent: String,
    pub value: String,
}

impl Element {
    pub const fn new(_tag_name: &str) -> Self {
        Self {
            id: String::new(),
            tagName: String::new(), // We'll set this separately since we can't call to_string() in const
            className: String::new(),
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
            tagName: tag_name.to_string(),
            className: String::new(),
            innerHTML: String::new(),
            innerText: String::new(),
            outerHTML: String::new(),
            textContent: String::new(),
            value: String::new(),
        }
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

    pub fn addEventListener(&self, _event_type: &str, _callback: fn()) {
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
}

// Document interface
#[js_type]
pub struct Document {}

impl Document {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn getElementById(&self, id: &str) -> Option<Element> {
        let mut element = Element::with_tag_name("div");
        element.id = id.to_string();
        Some(element)
    }

    pub fn getElementsByTagName(&self, tag_name: &str) -> Vec<Element> {
        vec![Element::with_tag_name(tag_name)]
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

    pub fn readyState(&self) -> String {
        "complete".to_string()
    }
}

// Console interface
#[js_type]
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
#[js_type]
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

    pub fn addEventListener(&self, _event_type: &str, _callback: fn()) {
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
#[js_type]
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
#[js_type]
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
#[js_type]
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
#[js_type]
pub struct Navigator {
    pub userAgent: String,
    pub language: String,
    pub platform: String,
    pub cookieEnabled: bool,
    pub onLine: bool,
    pub appName: String,
    pub appVersion: String,
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
#[js_type]
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
#[js_type]
pub struct Event {
    pub eventType: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub defaultPrevented: bool,
    pub target: Option<Element>,
    pub currentTarget: Option<Element>,
    pub timeStamp: f64,
}

impl Event {
    pub fn new(event_type: &str) -> Self {
        Self {
            eventType: event_type.to_string(),
            bubbles: false,
            cancelable: false,
            defaultPrevented: false,
            target: None,
            currentTarget: None,
            timeStamp: 0.0,
        }
    }

    pub fn preventDefault(&mut self) {
        self.defaultPrevented = true;
        println!("EVENT.PREVENT_DEFAULT: default action prevented");
    }

    pub fn stopPropagation(&self) {
        println!("EVENT.STOP_PROPAGATION: event propagation stopped");
    }

    pub fn stopImmediatePropagation(&self) {
        println!("EVENT.STOP_IMMEDIATE_PROPAGATION: immediate propagation stopped");
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

use std::collections::HashMap;
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
    #[cfg(feature = "serde")]
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
    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
    pub fn setJSON<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        localStorage.setJSON(key, value)
    }

    #[cfg(feature = "serde")]
    pub fn getJSON<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        localStorage.getJSON(key)
    }
}

/// Global sessionStorage instance
pub static sessionStorage: SessionStorage = SessionStorage;

#[cfg(test)]
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

    #[cfg(feature = "serde")]
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
