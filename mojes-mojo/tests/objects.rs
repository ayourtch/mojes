// js_object_tests.rs - Comprehensive tests for the new js_object macro and self->this conversion
// Fixed for correct Boa API usage

#[cfg(test)]
mod js_object_tests {
    use boa_engine::{Context, JsResult, JsValue, Source};
    use mojes_mojo::*;
    use syn::{Block, Expr, ItemImpl, ItemStruct, parse_quote};

    // Helper to evaluate JavaScript and get result
    fn eval_js(code: &str) -> JsResult<JsValue> {
        let mut context = Context::default();
        context.eval(Source::from_bytes(code))
    }

    // Helper to evaluate JavaScript with context and get result
    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        let result = context.eval(Source::from_bytes(code))?;
        Ok((result, context))
    }

    // Helper to test if generated JavaScript is syntactically valid
    fn is_valid_js(code: &str) -> bool {
        eval_js(code).is_ok()
    }

    // Helper to extract string from JsValue
    fn js_to_string(value: &JsValue, context: &mut Context) -> String {
        value.to_string(context).unwrap().to_std_string().unwrap()
    }

    // Helper to extract number from JsValue
    fn js_to_number(value: &JsValue, context: &mut Context) -> f64 {
        value.to_number(context).unwrap()
    }

    // Helper to extract boolean from JsValue
    fn js_to_boolean(value: &JsValue, context: &mut Context) -> bool {
        value.to_boolean()
    }

    fn js_class_person() -> String {
        r#"
class Person {
  constructor(name, age) {
    this.name = name;
    this.age = age;
  }

  toJSON() {
    return {
      name: this.name,
      age: this.age,
    };
  }

  static fromJSON(json) {
    return new Person(json.name, json.age);
  }
}
"#
        .to_string()
    }

    // ==================== 1. BASIC js_object MACRO TESTS ====================

    #[test]
    fn test_js_object_macro_basic() {
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn new(name: String, age: u32) -> Self {
                    Person { name, age }
                }

                fn greet(&self) -> String {
                    format!("Hello, I'm {}", self.name)
                }

                fn get_age(&self) -> u32 {
                    self.age
                }
            }
        };

        let js_methods = generate_js_methods_for_impl(&impl_block);
        println!("Generated methods:\n{}", js_methods);
        let js_person = js_class_person();

        // Should contain static method
        assert!(js_methods.contains("Person.new = function"));

        // Should contain instance methods
        assert!(js_methods.contains("Person.prototype.greet = function"));
        assert!(js_methods.contains("Person.prototype.get_age = function"));
        let js_code = format!("{}\n{}\n", &js_person, &js_methods);

        // Should be valid JavaScript - need to have it with class though
        eval_js(&js_code).unwrap();
        assert!(is_valid_js(&js_code));
    }

    #[test]
    fn test_self_to_this_conversion_in_methods() {
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn greet(&self) -> String {
                    format!("Hello, I'm {}", self.name)
                }

                fn get_info(&self) -> String {
                    format!("Name: {}, Age: {}", self.name, self.age)
                }

                fn birthday(&mut self) {
                    self.age = self.age + 1;
                }
            }
        };

        let js_methods = generate_js_methods_for_impl(&impl_block);
        println!("Generated methods with self->this:\n{}", js_methods);

        // Should NOT contain 'self' anywhere
        assert!(!js_methods.contains("self.name"));
        assert!(!js_methods.contains("self.age"));
        assert!(!js_methods.contains(" self"));

        // Should contain 'this' instead
        assert!(js_methods.contains("this.name"));
        assert!(js_methods.contains("this.age"));

        // Should be valid JavaScript
        assert!(is_valid_js(&js_methods));
    }

    // ==================== 2. FUNCTIONAL TESTING WITH BOA ====================

    #[test]
    fn test_complete_class_with_methods_execution() {
        // Generate struct
        let struct_def: ItemStruct = parse_quote! {
            struct Person {
                name: String,
                age: u32,
            }
        };
        let struct_js = generate_js_class_for_struct(&struct_def);

        // Generate methods
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn new(name: String, age: u32) -> Person {
                    Person { name: name, age: age }
                }

                fn greet(&self) -> String {
                    format!("Hello, I'm {}", self.name)
                }

                fn get_age(&self) -> u32 {
                    self.age
                }

                fn describe(&self) -> String {
                    format!("{} is {} years old", self.name, self.age)
                }
            }
        };
        let methods_js = generate_js_methods_for_impl(&impl_block);

        // Test individual methods separately for easier debugging
        let test_name = format!(
            r#"
            {}
            {}
            
            const instance = new Person("Alice", 30);
            instance.name;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_name).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Alice");

        let test_age = format!(
            r#"
            {}
            {}
            
            const instance = new Person("Alice", 30);
            instance.age;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_age).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 30.0);

        let test_greet = format!(
            r#"
            {}
            {}
            
            const instance = new Person("Alice", 30);
            instance.greet();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_greet).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Hello, I'm Alice");

        let test_get_age = format!(
            r#"
            {}
            {}
            
            const instance = new Person("Alice", 30);
            instance.get_age();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_get_age).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 30.0);

        let test_describe = format!(
            r#"
            {}
            {}
            
            const instance = new Person("Alice", 30);
            instance.describe();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_describe).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Alice is 30 years old");
    }

    #[test]
    fn test_mutable_methods_execution() {
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct Counter {
                value: i32,
            }
        });

        let impl_block: ItemImpl = parse_quote! {
            impl Counter {
                fn new(initial: i32) -> Counter {
                    Counter { value: initial }
                }

                fn increment(&mut self) {
                    self.value = self.value + 1;
                }

                fn get_value(&self) -> i32 {
                    self.value
                }

                fn add(&mut self, amount: i32) {
                    self.value = self.value + amount;
                }
            }
        };
        let methods_js = generate_js_methods_for_impl(&impl_block);

        // Test initial value
        let test_initial = format!(
            r#"
            {}
            {}
            
            const counter = Counter.new(5);
            counter.get_value();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_initial).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 5.0);

        // Test increment
        let test_increment = format!(
            r#"
            {}
            {}
            
            const counter = Counter.new(5);
            counter.increment();
            counter.get_value();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_increment).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 6.0);

        // Test add
        let test_add = format!(
            r#"
            {}
            {}
            
            const counter = Counter.new(5);
            counter.increment();
            counter.add(10);
            counter.get_value();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_add).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 16.0);
    }

    // ==================== 3. FORMAT MACRO WITH SELF REFERENCES ====================

    #[test]
    fn test_format_macro_self_conversion() {
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn introduce(&self) -> String {
                    format!("Hi, I'm {} and I'm {} years old", self.name, self.age)
                }

                fn nested_format(&self) -> String {
                    let greeting = format!("Hello {}", self.name);
                    format!("{}, you are {}", greeting, self.age)
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Format macro with self:\n{}", methods_js);

        // Should not contain 'self' in the generated code
        assert!(!methods_js.contains("${self."));
        assert!(!methods_js.contains("self.name"));
        assert!(!methods_js.contains("self.age"));

        // Should contain 'this' in template literals
        assert!(methods_js.contains("${this.name}"));
        assert!(methods_js.contains("${this.age}"));

        // Test execution
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct Person { name: String, age: u32 }
        });

        let test_introduce = format!(
            r#"
            {}
            {}
            
            const person = new Person("Alice", 25);
            person.introduce();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_introduce).unwrap();
        assert_eq!(
            js_to_string(&result, &mut ctx),
            "Hi, I'm Alice and I'm 25 years old"
        );

        let test_nested = format!(
            r#"
            {}
            {}
            
            const person = new Person("Bob", 30);
            person.nested_format();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_nested).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Hello Bob, you are 30");
    }

    // ==================== 4. STATIC VS INSTANCE METHODS ====================

    #[test]
    fn test_static_vs_instance_methods() {
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn new(name: String, age: u32) -> Person {
                    Person { name: name, age: age }
                }

                fn default() -> Person {
                    Person { name: "Unknown".to_string(), age: 0 }
                }

                fn greet(&self) -> String {
                    format!("Hello from {}", self.name)
                }

                fn is_adult(&self) -> bool {
                    self.age >= 18
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Static vs instance methods:\n{}", methods_js);

        // Static methods should be on the constructor
        assert!(methods_js.contains("Person.new = function"));
        assert!(methods_js.contains("Person.default = function"));

        // Instance methods should be on prototype
        assert!(methods_js.contains("Person.prototype.greet = function"));
        assert!(methods_js.contains("Person.prototype.is_adult = function"));

        // Test execution
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct Person { name: String, age: u32 }
        });

        // Test static constructor
        let test_static_new = format!(
            r#"
            {}
            {}
            
            const person1 = Person.new("Alice", 25);
            person1.name;
            "#,
            struct_js, methods_js
        );
        println!(
            "DEBUG test_static_vs_instance_methods test_static_new: {}",
            &test_static_new
        );

        let (result, mut ctx) = eval_js_with_context(&test_static_new).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Alice");

        // Test static default
        let test_static_default = format!(
            r#"
            {}
            {}
            
            const person2 = Person.default();
            person2.name;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_static_default).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Unknown");

        // Test instance methods
        let test_instance_greet = format!(
            r#"
            {}
            {}
            
            const person = Person.new("Alice", 25);
            person.greet();
            "#,
            struct_js, methods_js
        );
        println!(
            "DEBUG test_static_vs_instance_methods test_instance_greet: {}",
            test_instance_greet
        );

        let (result, mut ctx) = eval_js_with_context(&test_instance_greet).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Hello from Alice");

        let test_instance_adult = format!(
            r#"
            {}
            {}
            
            const person = Person.new("Alice", 25);
            person.is_adult();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_instance_adult).unwrap();
        assert_eq!(js_to_boolean(&result, &mut ctx), true);

        // Test child
        let test_instance_child = format!(
            r#"
            {}
            {}
            
            const child = Person.new("Bobby", 10);
            child.is_adult();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_instance_child).unwrap();
        assert_eq!(js_to_boolean(&result, &mut ctx), false);
    }

    // ==================== 5. COMPLEX SELF REFERENCE PATTERNS ====================

    #[test]
    fn test_complex_self_patterns() {
        let impl_block: ItemImpl = parse_quote! {
            impl Person {
                fn chain_methods(&self) -> String {
                    let temp = self.name.clone();
                    let result = if self.age > 18 {
                        format!("Adult: {}", temp)
                    } else {
                        format!("Minor: {}", temp)
                    };
                    result
                }

                fn conditional_self(&self) -> i32 {
                    if self.age > 65 {
                        self.age * 2
                    } else if self.age > 18 {
                        self.age + 10
                    } else {
                        self.age - 5
                    }
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Complex self patterns:\n{}", methods_js);

        // Should not contain any 'self' references
        assert!(!methods_js.contains("self."));
        assert!(!methods_js.contains(" self"));

        // Should contain proper 'this' references
        assert!(methods_js.contains("this.name"));
        assert!(methods_js.contains("this.age"));

        // Test execution
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct Person { name: String, age: u32 }
        });

        let test_adult_chain = format!(
            r#"
            {}
            {}
            
            const adult = new Person("Alice", 25);
            adult.chain_methods();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_adult_chain).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Adult: Alice");

        let test_minor_chain = format!(
            r#"
            {}
            {}
            
            const minor = new Person("Charlie", 15);
            minor.chain_methods();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_minor_chain).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Minor: Charlie");

        let test_adult_conditional = format!(
            r#"
            {}
            {}
            
            const adult = new Person("Alice", 25);
            adult.conditional_self();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_adult_conditional).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 35.0); // 25 + 10

        let test_senior_conditional = format!(
            r#"
            {}
            {}
            
            const senior = new Person("Bob", 70);
            senior.conditional_self();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_senior_conditional).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 140.0); // 70 * 2

        let test_minor_conditional = format!(
            r#"
            {}
            {}
            
            const minor = new Person("Charlie", 15);
            minor.conditional_self();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_minor_conditional).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 10.0); // 15 - 5
    }

    // ==================== 6. COMPLEX DATA STRUCTURES ====================

    #[test]
    fn test_complex_struct_with_methods() {
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct Rectangle {
                width: f64,
                height: f64,
                name: String,
            }
        });

        let impl_block: ItemImpl = parse_quote! {
            impl Rectangle {
                fn new(width: f64, height: f64, name: String) -> Rectangle {
                    Rectangle { width: width, height: height, name: name }
                }

                fn area(&self) -> f64 {
                    self.width * self.height
                }

                fn perimeter(&self) -> f64 {
                    2.0 * (self.width + self.height)
                }

                fn scale(&mut self, factor: f64) {
                    self.width = self.width * factor;
                    self.height = self.height * factor;
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);

        let test_area = format!(
            r#"
            {}
            {}
            
            const rect = Rectangle.new(10.0, 5.0, "MyRect");
            rect.area();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_area).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 50.0);

        let test_perimeter = format!(
            r#"
            {}
            {}
            
            const rect = Rectangle.new(10.0, 5.0, "MyRect");
            rect.perimeter();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_perimeter).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 30.0);

        let test_scale = format!(
            r#"
            {}
            {}
            
            const rect = Rectangle.new(10.0, 5.0, "MyRect");
            rect.scale(2.0);
            rect.area();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_scale).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 200.0); // (10*2) * (5*2) = 20 * 10 = 200
    }

    // ==================== 7. ERROR HANDLING AND EDGE CASES ====================

    #[test]
    fn test_empty_impl_block() {
        let impl_block: ItemImpl = parse_quote! {
            impl EmptyStruct {
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Empty impl block:\n{}", methods_js);

        // Should generate valid but minimal JavaScript
        assert!(methods_js.contains("Methods for EmptyStruct"));
        assert!(is_valid_js(&methods_js));
    }

    #[test]
    fn test_methods_without_self() {
        let impl_block: ItemImpl = parse_quote! {
            impl Utility {
                fn add(a: i32, b: i32) -> i32 {
                    a + b
                }

                fn multiply(x: f64, y: f64) -> f64 {
                    x * y
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Static methods only:\n{}", methods_js);

        // Should all be static methods
        assert!(methods_js.contains("Utility.add = function"));
        assert!(methods_js.contains("Utility.multiply = function"));
        assert!(!methods_js.contains("prototype"));

        let test_add = format!(
            r#"
            {}
            
            Utility.add(5, 3);
            "#,
            methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_add).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 8.0);

        let test_multiply = format!(
            r#"
            {}
            
            Utility.multiply(4.5, 2.0);
            "#,
            methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_multiply).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 9.0);
    }

    // ==================== 8. PERFORMANCE AND VALIDATION ====================

    #[test]
    fn test_large_impl_block() {
        // Create a large impl block to test performance
        let impl_block: ItemImpl = parse_quote! {
            impl LargeStruct {
                fn method_01(&self) -> String { format!("Method 01: {}", self.field) }
                fn method_02(&self) -> String { format!("Method 02: {}", self.field) }
                fn method_03(&self) -> String { format!("Method 03: {}", self.field) }
                fn method_04(&self) -> String { format!("Method 04: {}", self.field) }
                fn method_05(&self) -> String { format!("Method 05: {}", self.field) }

                fn static_01() -> i32 { 1 }
                fn static_02() -> i32 { 2 }
                fn static_03() -> i32 { 3 }

                fn mutating_01(&mut self) { self.field = "updated_01".to_string(); }
                fn mutating_02(&mut self) { self.field = "updated_02".to_string(); }
            }
        };

        let start = std::time::Instant::now();
        let methods_js = generate_js_methods_for_impl(&impl_block);
        let duration = start.elapsed();

        println!("Large impl block processed in: {:?}", duration);

        // Should contain all methods
        assert!(methods_js.contains("method_01"));
        assert!(methods_js.contains("method_05"));
        assert!(methods_js.contains("static_01"));
        assert!(methods_js.contains("static_03"));
        assert!(methods_js.contains("mutating_01"));
        assert!(methods_js.contains("mutating_02"));

        // Should not contain any self references
        assert!(!methods_js.contains("self.field"));
        assert!(methods_js.contains("this.field"));

        // Should be valid JavaScript
        assert!(is_valid_js(&methods_js));

        // Performance should be reasonable (less than 100ms for this size)
        assert!(duration.as_millis() < 100);
    }

    // ==================== 9. COMPREHENSIVE REAL-WORLD TEST ====================

    #[test]
    fn test_comprehensive_real_world_scenario() {
        // Create a realistic task manager class
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct TaskManager {
                tasks: Vec<String>,
                completed: Vec<bool>,
                current_id: u32,
                name: String,
            }
        });

        let impl_block: ItemImpl = parse_quote! {
            impl TaskManager {
                fn new(name: String) -> TaskManager {
                    TaskManager {
                        tasks: vec![],
                        completed: vec![],
                        current_id: 0,
                        name: name,
                    }
                }

                fn add_task(&mut self, task: String) -> u32 {
                    self.tasks.push(task);
                    self.completed.push(false);
                    let id = self.current_id;
                    self.current_id = self.current_id + 1;
                    id
                }

                fn complete_task(&mut self, id: u32) -> bool {
                    if (id as usize) < self.completed.len() {
                        self.completed[id as usize] = true;
                        true
                    } else {
                        false
                    }
                }

                fn get_progress(&self) -> f64 {
                    if self.tasks.len() == 0 {
                        0.0
                    } else {
                        let completed_count = self.completed.iter()
                            .filter(|&&completed| completed)
                            .count();
                        (completed_count as f64) / (self.tasks.len() as f64) * 100.0
                    }
                }

                fn get_summary(&self) -> String {
                    let total = self.tasks.len();
                    let completed_count = self.completed.iter()
                        .filter(|&&completed| completed)
                        .count();
                    let pending = total - completed_count;

                    format!(
                        "{}: {} total, {} completed, {} pending",
                        self.name,
                        total,
                        completed_count,
                        pending
                    )
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);

        // Test creating task manager
        let test_create = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.name;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_create).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "My Project");

        // Test initial progress
        let test_initial_progress = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.get_progress();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_initial_progress).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 0.0);

        // Test adding tasks and progress
        let test_add_tasks = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.add_task("Setup project");
            tm.add_task("Write code");
            tm.add_task("Write tests");
            tm.get_progress();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_add_tasks).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 0.0); // No completed tasks yet

        // Test completing tasks and progress
        let test_complete_tasks = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.add_task("Setup project");
            tm.add_task("Write code");
            tm.add_task("Write tests");
            tm.complete_task(0);
            tm.complete_task(1);
            tm.get_progress();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_complete_tasks).unwrap();
        let progress = js_to_number(&result, &mut ctx);
        assert!((progress - 66.66666666666667).abs() < 0.01); // 2/3 completed

        // Test summary
        let test_summary = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.add_task("Setup project");
            tm.add_task("Write code");
            tm.complete_task(0);
            tm.get_summary();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_summary).unwrap();
        let summary = js_to_string(&result, &mut ctx);
        assert!(summary.contains("My Project"));
        assert!(summary.contains("2 total"));
        assert!(summary.contains("1 completed"));
        assert!(summary.contains("1 pending"));

        // Test complete task return value
        let test_complete_valid = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.add_task("Setup project");
            tm.complete_task(0);
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_complete_valid).unwrap();
        assert_eq!(js_to_boolean(&result, &mut ctx), true);

        // Test complete invalid task
        let test_complete_invalid = format!(
            r#"
            {}
            {}
            
            const tm = TaskManager.new("My Project");
            tm.add_task("Setup project");
            tm.complete_task(999);
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_complete_invalid).unwrap();
        assert_eq!(js_to_boolean(&result, &mut ctx), false);

        println!("✅ Complete real-world TaskManager class test passed!");
        println!("   - All methods work correctly");
        println!("   - Self->this conversion successful");
        println!("   - Complex calculations and state management functional");
        println!("   - Generated JavaScript executes perfectly");
    }

    // ==================== 10. INTEGRATION WITH EXISTING FEATURES ====================

    #[test]
    fn test_compatibility_with_existing_features() {
        // Test that the new js_object functionality works with all existing features
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct TestStruct {
                name: String,
                items: Vec<i32>,
                active: bool,
            }
        });

        let impl_block: ItemImpl = parse_quote! {
            impl TestStruct {
                fn new() -> TestStruct {
                    TestStruct {
                        name: "Test".to_string(),
                        items: vec![1, 2, 3],
                        active: true,
                    }
                }

                fn process_all(&self) -> String {
                    // Test format! with self
                    let name_info = format!("Name: {}", self.name);

                    // Test method calls on self fields
                    let item_count = self.items.len();

                    // Test conditional with self
                    let status = if self.active { "active" } else { "inactive" };

                    // Test complex expression with self
                    let result = format!("{} ({}) - {} items", name_info, status, item_count);

                    result
                }

                fn get_item_count(&self) -> u32 {
                    self.items.len() as u32
                }

                fn toggle_active(&mut self) {
                    self.active = !self.active;
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);

        let test_new = format!(
            r#"
            {}
            {}
            
            const instance = TestStruct.new();
            instance.name;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_new).unwrap();
        assert_eq!(js_to_string(&result, &mut ctx), "Test");

        let test_process = format!(
            r#"
            {}
            {}
            
            const instance = TestStruct.new();
            instance.process_all();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_process).unwrap();
        let processed = js_to_string(&result, &mut ctx);
        assert!(processed.contains("Name: Test"));
        assert!(processed.contains("active"));
        assert!(processed.contains("3 items"));

        let test_item_count = format!(
            r#"
            {}
            {}
            
            const instance = TestStruct.new();
            instance.get_item_count();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_item_count).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 3.0);

        let test_toggle = format!(
            r#"
            {}
            {}
            
            const instance = TestStruct.new();
            instance.toggle_active();
            instance.active;
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_toggle).unwrap();
        assert_eq!(js_to_boolean(&result, &mut ctx), false); // Should be toggled to false
    }

    // ==================== 11. DEEPLY NESTED SELF REFERENCES ====================

    #[test]
    fn test_deeply_nested_self_references() {
        let impl_block: ItemImpl = parse_quote! {
            impl ComplexStruct {
                fn deeply_nested(&self) -> String {
                    let result = {
                        let inner = {
                            let deep = {
                                format!("Deep: {}", self.name)
                            };
                            format!("Inner: {} - {}", deep, self.age)
                        };
                        format!("Outer: {} - {}", inner, self.active)
                    };
                    result
                }

                fn nested_conditions(&self) -> i32 {
                    if self.age > 50 {
                        if self.active {
                            self.age * 2
                        } else {
                            self.age
                        }
                    } else {
                        if self.active {
                            self.age + 10
                        } else {
                            self.age - 10
                        }
                    }
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!("Deeply nested self references:\n{}", methods_js);

        // Ensure NO self references remain anywhere
        assert!(!methods_js.contains("self."));
        assert!(!methods_js.contains(" self"));

        // Should contain proper this references
        assert!(methods_js.contains("this.name"));
        assert!(methods_js.contains("this.age"));
        assert!(methods_js.contains("this.active"));

        // Should be valid JavaScript
        assert!(is_valid_js(&methods_js));

        // Test with a struct for execution
        let struct_js = generate_js_class_for_struct(&parse_quote! {
            struct ComplexStruct { name: String, age: u32, active: bool }
        });

        let test_nested = format!(
            r#"
            {}
            {}
            
            const obj = new ComplexStruct("Test", 25, true);
            obj.deeply_nested();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_nested).unwrap();
        let nested_result = js_to_string(&result, &mut ctx);
        assert!(nested_result.contains("Deep: Test"));
        assert!(nested_result.contains("25"));
        assert!(nested_result.contains("true"));

        let test_conditions = format!(
            r#"
            {}
            {}
            
            const young_active = new ComplexStruct("Test", 25, true);
            young_active.nested_conditions();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_conditions).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 35.0); // 25 + 10 (young and active)

        let test_old_active = format!(
            r#"
            {}
            {}
            
            const old_active = new ComplexStruct("Test", 60, true);
            old_active.nested_conditions();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_old_active).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 120.0); // 60 * 2 (old and active)

        let test_young_inactive = format!(
            r#"
            {}
            {}
            
            const young_inactive = new ComplexStruct("Test", 25, false);
            young_inactive.nested_conditions();
            "#,
            struct_js, methods_js
        );

        let (result, mut ctx) = eval_js_with_context(&test_young_inactive).unwrap();
        assert_eq!(js_to_number(&result, &mut ctx), 15.0); // 25 - 10 (young and inactive)
    }

    // ==================== 12. FINAL VALIDATION ====================

    #[test]
    fn test_no_self_references_in_generated_code() {
        // This test specifically validates that NO self references make it through
        let impl_block: ItemImpl = parse_quote! {
            impl TestClass {
                fn method1(&self) -> String {
                    format!("Hello {}", self.name)
                }

                fn method2(&self) -> i32 {
                    self.value + self.other_value
                }

                fn method3(&self) -> bool {
                    self.flag && self.other_flag
                }

                fn method4(&self) -> String {
                    let temp = self.name.clone();
                    if self.active {
                        format!("Active: {}", temp)
                    } else {
                        format!("Inactive: {}", temp)
                    }
                }

                fn method5(&mut self) {
                    self.value = self.value + 1;
                    self.name = format!("Updated: {}", self.name);
                }
            }
        };

        let methods_js = generate_js_methods_for_impl(&impl_block);
        println!(
            "Final validation - checking for self references:\n{}",
            methods_js
        );

        // Count occurrences of self vs this
        let self_count = methods_js.matches("self.").count();
        let this_count = methods_js.matches("this.").count();

        println!("Self references found: {}", self_count);
        println!("This references found: {}", this_count);

        // Should have ZERO self references
        assert_eq!(
            self_count, 0,
            "Found {} self references in generated JavaScript!",
            self_count
        );

        // Should have multiple this references
        assert!(this_count > 0, "Should have some this references");

        // Should be valid JavaScript
        assert!(is_valid_js(&methods_js));
    }
}
