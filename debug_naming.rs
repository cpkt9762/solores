use heck::ToShoutySnakeCase;

fn main() {
    let name = "collectFeesV2";
    println\!("Input: {}", name);
    println\!("ToShoutySnakeCase: {}", name.to_shouty_snake_case());
    println\!("Expected: COLLECT_FEES_V2_IX_ACCOUNTS_LEN");
    println\!("Generated: {}_IX_ACCOUNTS_LEN", name.to_shouty_snake_case());
}
EOF < /dev/null