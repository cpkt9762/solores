fn main() { 
    let json = std::fs::read_to_string("idls/spl/spl-token.json").unwrap();
    let result: Result<solores::idl_format::non_anchor_idl::NonAnchorIdl, _> = serde_json::from_str(&json);
    match result {
        Ok(_) => println!("Parsed successfully"),
        Err(e) => println!("Parse error: {}", e),
    }
}
