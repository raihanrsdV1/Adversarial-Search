// This is a single-line comment. Rust also supports multi-line comments /* like this */.

// The `main` function is the entry point of every Rust executable program.
// When you run your compiled Rust code, the `main` function is what gets called first.
fn main() {
    // `println!` is a macro (you can tell by the `!`) that prints text to the console.
    // Macros are a way of writing code that writes other code (metaprogramming).
    // They are more powerful than functions in some ways.
    println!("Hello, Rust world!"); // Strings are UTF-8 encoded.
    println!("Wow my first line in rust");

    // --- Variables and Mutability ---
    // By default, variables in Rust are immutable (cannot be changed once assigned).
    let an_immutable_variable = 5;
    // let nice = "something";
    println!("This variable is immutable: {}", an_immutable_variable);

    // If you try to reassign it, the compiler will give an error:
    // an_immutable_variable = 10; // This would cause a compile-time error.

    // To make a variable mutable, use the `mut` keyword.
    let mut a_mutable_variable = 10;
    println!("This variable is mutable. Initial value: {}", a_mutable_variable);
    a_mutable_variable = 20;
    println!("Its value has been changed to: {}", a_mutable_variable);

    // Rust is statically typed, but it has good type inference.
    // The compiler can usually figure out the type of a variable.
    let an_integer = 42;         // Compiler infers type i32 (default integer type)
    let a_float = 3.14;          // Compiler infers type f64 (default float type)
    let is_true = true;          // Compiler infers type bool
    let a_character = 'c';       // Type char (single Unicode scalar value)
    let a_string_slice = "Rust"; // Type &str (string slice - a reference to a string)

    // You can also explicitly declare the type.
    let explicit_integer: i32 = -100;
    let explicit_float: f64 = 2.71828;
    println!("Explicitly typed integer: {} {}", explicit_integer, explicit_float);
    println!("Explicitly typed float: {}", explicit_float);


    // --- Functions ---
    // Let's call a function we define later.
    let sum_result = add_two_numbers(an_integer, 58); // 42 + 58
    println!("The sum from our function is: {}", sum_result);

    // Functions can also have no return value (implicitly returns a unit type `()`).
    greet("Alice");


    // --- Control Flow: if/else ---
    let number = 7;
    if number < 5 {
        println!("Condition was true: number is less than 5");
    } else if number == 5 {
        println!("Condition was true: number is exactly 5");
    } else {
        println!("Condition was false: number is greater than 5");
    }

    // `if` is an expression, so you can use it in a `let` statement.
    // All branches must return the same type.
    let condition_result = if number % 2 == 0 {
        "even" // This is a &str (string slice)
    } else {
        "odd"  // This is also a &str
    };
    println!("The number {} is {}.", number, condition_result);

    // --- Basic Struct Definition ---
    // Structs are like classes or objects in other languages, used to group related data.
    struct Point {
        x: f64, // field x of type f64 (64-bit float)
        y: f64, // field y of type f64
    }

    // Create an instance of the Point struct
    let origin = Point { x: 0.0, y: 0.0 };
    let some_point = Point { x: 1.2, y: 3.4 };

    println!("Origin is at ({}, {})", origin.x, origin.y);
    println!("Some point is at ({}, {})", some_point.x, some_point.y);

    // --- Basic Enum Definition ---
    // Enums (enumerations) allow you to define a type by enumerating its possible variants.
    enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    let player_move = Direction::Up;

    // We can use `match` (a powerful control flow construct) to handle enums.
    // `match` is similar to a switch statement but more versatile.
    match player_move {
        Direction::Up => println!("Player moves Up!"),
        Direction::Down => println!("Player moves Down!"),
        Direction::Left => println!("Player moves Left!"),
        Direction::Right => println!("Player moves Right!"),
        // `match` statements must be exhaustive, meaning all possible values
        // of the type being matched must be covered.
    }

    println!("\nEnd of the boilerplate demonstration!");
} // End of the `main` function

// --- Function Definitions ---
// Functions are defined using the `fn` keyword.
// You must declare the types of function parameters.
// The arrow `->` indicates the return type of the function.
// The last expression in a function is implicitly returned (if no semicolon).
// Or you can use the `return` keyword explicitly.

fn add_two_numbers(a: i32, b: i32) -> i32 {
    a + b // No semicolon means this expression's value is returned.
          // Alternatively: return a + b;
}

// A function that takes a string slice (`&str`) and doesn't return anything.
// (Implicitly returns the unit type `()`).
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// To compile and run this code:
// 1. Save it as `main.rs` (or any other name like `boilerplate.rs`) in a new directory.
// 2. Open your terminal, navigate to that directory.
// 3. Run `rustc main.rs` to compile. This will create an executable.
// 4. Run the executable: `./main` (on Linux/macOS) or `main.exe` (on Windows).
//
// For a more structured project, especially as it grows, you should use Cargo,
// Rust's build system and package manager:
// 1. In your terminal, run `cargo new my_rust_project --bin`
//    (this creates a new project folder named `my_rust_project`).
// 2. Navigate into the project: `cd my_rust_project`.
// 3. Replace the content of `src/main.rs` with the code above.
// 4. Run `cargo run` from within the `my_rust_project` directory.
//    Cargo will compile and run your program.
//    Other useful Cargo commands:
//    - `cargo build`: Compiles your project.
//    - `cargo check`: Quickly checks your code for errors without producing an executable.

// main.rs

// To use HashMap and HashSet, we need to import them from the standard library's collections module.
use std::collections::{HashMap, HashSet};

// --- Structs: Defining Custom Data Types (like simple classes) ---
// Structs are used to group related data together.
#[derive(Debug)] // This allows us to print the struct using {:?} or {:#?}
struct Character {
    name: String, // String is an owned, growable string type
    level: u32,   // u32 is an unsigned 32-bit integer
    is_active: bool,
    inventory: Vec<String>, // A vector of strings for the character's items
}

// --- Impl Blocks: Adding Methods to Structs (like class methods) ---
// `impl` blocks are where you define methods associated with a struct.
impl Character {
    // This is an "associated function" often used as a constructor.
    // It doesn't take `&self`, so it's called like `Character::new(...)`.
    fn new(name: &str, level: u32) -> Self { // `Self` refers to the type `Character`
        Character {
            name: String::from(name), // Create an owned String from a string slice
            level, // Shorthand for `level: level`
            is_active: true,
            inventory: Vec::new(), // Initialize with an empty vector
        }
    }

    // This is a "method" because it takes `&self` as its first parameter.
    // `&self` is a reference to the instance of the struct the method is called on.
    // It's like `this` or `self` in other languages.
    fn describe(&self) {
        println!(
            "Name: {}, Level: {}, Active: {}",
            self.name, self.level, self.is_active
        );
        if !self.inventory.is_empty() {
            println!("Inventory: {:?}", self.inventory);
        }
    }

    // A method that modifies the struct instance (takes `&mut self`).
    fn level_up(&mut self) {
        self.level += 1;
        println!("{} leveled up to {}!", self.name, self.level);
    }

    fn add_item(&mut self, item: &str) {
        self.inventory.push(String::from(item));
        println!("{} picked up {}.", self.name, item);
    }
}

// --- Traits: Defining Shared Behavior (like interfaces or abstract classes) ---
// A trait defines a set of methods that a type must implement to claim it has that behavior.
trait CanAttack {
    // A method signature: takes `&self` and an i32 for damage.
    fn attack(&self, target_name: &str, damage: i32);

    // Traits can also have default method implementations.
    fn special_attack(&self, target_name: &str) {
        println!(
            "{} performs a generic special attack on {}!",
            self.get_name(), // Assumes a get_name method will be available
            target_name
        );
    }

    // A method that the implementor must provide (no default)
    fn get_name(&self) -> &String;
}

// Now, let's implement the `CanAttack` trait for our `Character` struct.
impl CanAttack for Character {
    fn attack(&self, target_name: &str, damage: i32) {
        println!(
            "{} attacks {} for {} damage!",
            self.name, target_name, damage
        );
    }

    // We could override special_attack here if we wanted a specific one for Character.
    // fn special_attack(&self, target_name: &str) {
    //     println!("{} unleashes a MIGHTY BLOW on {}!", self.name, target_name);
    // }

    fn get_name(&self) -> &String {
        &self.name // Return a reference to the character's name
    }
}

// A function that accepts any type that implements the `CanAttack` trait.
// This is a form of polymorphism.
fn perform_combat_action(attacker: &impl CanAttack, target: &str) {
    attacker.attack(target, 10); // Assuming a default damage
    attacker.special_attack(target);
}


fn main() {
    println!("--- Structs, Impl Blocks, and Traits (OOP-like features) ---");
    let mut hero = Character::new("Aragorn", 5);
    hero.describe();
    hero.add_item("Sword");
    hero.add_item("Shield");
    hero.level_up();
    hero.describe();

    let villain = Character::new("Saruman", 10); // Create another character

    // Using the trait methods
    hero.attack("an Orc", 15);
    villain.special_attack("Gandalf");

    // Using the generic function with a type that implements CanAttack
    perform_combat_action(&hero, "a Troll");
    perform_combat_action(&villain, "Frodo");


    println!("\n--- Vectors (Dynamic Arrays, like Python Lists) ---");
    // `Vec<T>` is a growable array type.
    let mut numbers: Vec<i32> = Vec::new(); // Create an empty vector of i32
    numbers.push(10); // Add an element
    numbers.push(20);
    numbers.push(30);
    println!("Vector: {:?}", numbers); // {:?} is a debug print format

    let first_number = numbers[0]; // Access elements by index (panics if out of bounds)
    println!("First number: {}", first_number);

    // Safe access using .get(), which returns an Option<&T>
    match numbers.get(1) {
        Some(val) => println!("Second number (safe access): {}", val),
        None => println!("No element at index 1"),
    }
    match numbers.get(100) { // Index out of bounds
        Some(val) => println!("Element at 100: {}", val), // This won't run
        None => println!("No element at index 100 (safe access)"),
    }


    // Iterating over a vector
    println!("Iterating over numbers:");
    for num in &numbers { // `&numbers` iterates over references to elements
        println!("- {}", num);
    }

    // Modifying elements while iterating
    for num in numbers.iter_mut() { // `iter_mut()` gives mutable references
        *num *= 2; // Dereference `num` to modify the value
    }
    println!("Vector after doubling: {:?}", numbers);

    // Initialize a vector with `vec!` macro
    let names = vec![String::from("Alice"), String::from("Bob"), String::from("Charlie")];
    println!("Names: {:?}", names);


    println!("\n--- HashMaps (like Python Dictionaries) ---");
    // `HashMap<K, V>` stores key-value pairs.
    let mut scores: HashMap<String, i32> = HashMap::new();

    scores.insert(String::from("RedTeam"), 100);
    scores.insert(String::from("BlueTeam"), 95);
    scores.insert(String::from("GreenTeam"), 110);
    println!("Scores HashMap: {:?}", scores);

    // Accessing values
    let red_score = scores.get("RedTeam"); // Returns Option<&i32>
    match red_score {
        Some(score) => println!("Red Team's score: {}", score),
        None => println!("Red Team not found."),
    }

    // Overwriting a value
    scores.insert(String::from("RedTeam"), 105);
    println!("Updated Red Team score: {:?}", scores.get("RedTeam").unwrap()); // .unwrap() panics if None

    // Iterating over a HashMap
    println!("Iterating over scores:");
    for (team, score) in &scores { // Iterates over (&Key, &Value)
        println!("- {}: {}", team, score);
    }

    // Checking for a key
    if scores.contains_key("BlueTeam") {
        println!("BlueTeam exists in scores.");
    }

    // Entry API (useful for complex logic like updating or inserting if not present)
    let yellow_score = scores.entry(String::from("YellowTeam")).or_insert(80);
    *yellow_score += 5; // Modify the value directly
    println!("Yellow Team score (added/updated via entry): {}", scores["YellowTeam"]);


    println!("\n--- HashSets (like Python Sets) ---");
    // `HashSet<T>` stores unique elements.
    let mut unique_tags: HashSet<String> = HashSet::new();
    unique_tags.insert(String::from("rust"));
    unique_tags.insert(String::from("programming"));
    unique_tags.insert(String::from("rust")); // This will not be added again
    println!("Unique Tags HashSet: {:?}", unique_tags);

    if unique_tags.contains("rust") {
        println!("'rust' tag is present.");
    }

    // Iterating over a HashSet (order is not guaranteed)
    println!("Iterating over tags:");
    for tag in &unique_tags {
        println!("- {}", tag);
    }


    println!("\n--- Array/Slice Manipulation (Iterators) ---");
    // Rust's iterators are very powerful and are the primary way to do complex sequence operations.
    // They are lazy, meaning they don't do work until you consume them (e.g., with `collect`).

    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Original data: {:?}", data);

    // `map`: Transform each element (like Python's map)
    let squared: Vec<i32> = data.iter() // Create an iterator over references
                                .map(|x| x * x) // Closure that squares each element
                                .collect();     // Collect results into a new Vec
    println!("Squared: {:?}", squared);

    // `filter`: Keep elements that satisfy a condition (like Python's filter)
    let even_numbers: Vec<&i32> = data.iter()
                                    .filter(|&x| x % 2 == 0) // `&x` because iter() gives &i32
                                    .collect();
    println!("Even numbers (as references): {:?}", even_numbers);
    // To get owned values if you need them (e.g. if original data goes out of scope):
    let owned_even_numbers: Vec<i32> = data.iter()
                                        .filter(|&x| x % 2 == 0)
                                        .cloned() // or .map(|&x| x) if type is not Clone
                                        .collect();
    println!("Owned even numbers: {:?}", owned_even_numbers);


    // Chaining operations
    let sum_of_squares_of_odds: i32 = data.iter()
                                          .filter(|&x| x % 2 != 0) // Keep odd numbers
                                          .map(|x| x * x)          // Square them
                                          .sum();                  // Sum the results
    println!("Sum of squares of odds: {}", sum_of_squares_of_odds);

    // `find`: Get the first element satisfying a condition
    let first_gt_5 = data.iter().find(|&&x| x > 5); // `&&x` because find gives `&&i32`
    match first_gt_5 {
        Some(val) => println!("First number greater than 5: {}", val),
        None => println!("No number greater than 5 found."),
    }

    // Slices: These are references to a contiguous sequence of elements in a collection.
    // They don't own data.
    let some_numbers = [10, 20, 30, 40, 50]; // Fixed-size array
    let slice_of_numbers: &[i32] = &some_numbers[1..4]; // Elements at index 1, 2, 3 (exclusive end)
    println!("Slice [1..4] of some_numbers: {:?}", slice_of_numbers); // Output: [20, 30, 40]

    // While Rust doesn't have Python's direct slice assignment for modification (e.g., `my_list[1:3] = [9,8]`),
    // you can get mutable slices and modify elements, or use iterator methods for more complex transformations.
    let mut mutable_array = [1, 2, 3, 4, 5];
    let mutable_slice = &mut mutable_array[1..3]; // Slice is [2, 3]
    mutable_slice[0] = 99; // Modifies the original array: mutable_array becomes [1, 99, 3, 4, 5]
    mutable_slice[1] = 88; // mutable_array becomes [1, 99, 88, 4, 5]
    println!("Modified array via mutable slice: {:?}", mutable_array);

    // For more complex replacements or insertions, you'd typically use Vec methods like `splice`,
    // `drain`, or build a new Vec.

    println!("\n--- End of Collections & OOP-like Features Demo ---");
}