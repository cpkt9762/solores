fn extract_array_parts(value: &str) -> Option<(String, String)> {
    println!("extract_array_parts called with: '{}'", value);
    if !value.starts_with("[") || !value.ends_with("]") {
        println!("  -> Not a valid array format");
        return None;
    }
    
    let inner = &value[1..value.len()-1];
    println!("  -> inner: '{}'", inner);
    if let Some(semicolon_pos) = inner.find(';') {
        let type_part = inner[..semicolon_pos].trim().to_string();
        let size_part = inner[semicolon_pos+1..].trim().to_string();
        println!("  -> type_part: '{}', size_part: '{}'", type_part, size_part);
        Some((type_part, size_part))
    } else {
        println!("  -> No semicolon found");
        None
    }
}

fn main() {
    println!("Testing extract_array_parts with '[[u64; 8]; 12]':");
    if let Some((inner_type, size)) = extract_array_parts("[[u64; 8]; 12]") {
        println!("Result: inner_type='{}', size='{}'", inner_type, size);
    }
}
