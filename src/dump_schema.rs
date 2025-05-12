fn main() {
    println!("rust_mcp_schema modules:");
    for entry in rust_mcp_schema::SCHEMA_MODULES {
        println!("  {}", entry);
    }
}