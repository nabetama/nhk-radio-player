use serde_json;
use std::fs;

#[path = "src/types.rs"]
mod types;

fn main() {
    let json_data = fs::read_to_string("/tmp/nhk_response.json")
        .expect("Failed to read file");

    match serde_json::from_str::<types::Root>(&json_data) {
        Ok(root) => {
            println!("Success!");
            println!("R1 present: {}", root.r1.present.about.name);
        }
        Err(e) => {
            println!("Error: {}", e);
            println!("Line: {}", e.line());
            println!("Column: {}", e.column());
        }
    }
}
