#[test]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("Program arguments: {:?}", args.len());
    println!("test number: {:?}", 42);
}
