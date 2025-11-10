//! Example: Advanced RESO Query Features
//!
//! This example demonstrates advanced query features from the reso_client library:
//! - Direct key-based lookups
//! - Expanding related entities
//! - Combining multiple query features
//!
//! ## Setup
//!
//! 1. Copy `.env.example` to `.env`
//! 2. Fill in your RESO credentials:
//!    - RESO_BASE_URL: Your RESO API base URL
//!    - RESO_TOKEN: Your bearer authentication token
//!    - RESO_DATASET_ID: (optional) Dataset identifier
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example advanced_queries
//! ```

use reso_examples::{
    load_env, create_client, build_query_by_key, build_query_with_expand,
    build_query_with_select, execute_query,
};
use serde_json::Value as JsonValue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    load_env()?;

    println!("=== RESO Advanced Query Examples ===\n");

    // Create RESO client from environment variables
    println!("Creating RESO client from environment...");
    let client = create_client()?;
    println!("✓ Client created successfully\n");

    // First, get a listing key to use for key-based lookup
    println!("Step 1: Finding a sample property to demonstrate key-based lookup...");
    println!("{}", "-".repeat(60));
    let sample_query = build_query_with_select(
        "Property",
        None,
        &["ListingKey", "City", "ListPrice"],
        Some(1),
    )?;
    let sample_response = execute_query(&client, &sample_query).await?;

    let listing_key = if let Some(records) = sample_response["value"].as_array() {
        if let Some(record) = records.first() {
            if let Some(key) = record["ListingKey"].as_str() {
                println!("✓ Found sample property with ListingKey: {}", key);
                Some(key.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Example 1: Direct key-based lookup
    if let Some(key) = listing_key {
        println!("\nExample 1: Direct key-based lookup (more efficient than filtering)...");
        println!("{}", "-".repeat(60));
        println!("Looking up property with key: {}", key);

        let key_query = build_query_by_key(
            "Property",
            &key,
            Some(&[
                "ListingKey",
                "City",
                "StateOrProvince",
                "ListPrice",
                "BedroomsTotal",
                "BathroomsTotalInteger",
                "StandardStatus",
                "PropertyType",
            ]),
        )?;

        match execute_query(&client, &key_query).await {
            Ok(response) => {
                println!("\n✓ Property retrieved successfully:");
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
            Err(e) => {
                println!("Note: Key-based lookup failed: {}", e);
                println!("This is normal if your RESO server doesn't support direct key access.");
            }
        }
    } else {
        println!("\nExample 1: Skipped (no sample property found)");
    }

    // Example 2: Query with expanded related entities
    println!("\nExample 2: Expanding related entities (ListOffice, ListAgent)...");
    println!("{}", "-".repeat(60));
    println!("Note: This requires server support for $expand navigation properties.");

    let expand_query = build_query_with_expand(
        "Property",
        Some("StandardStatus eq 'Active'"),
        &["ListingKey", "City", "ListPrice"],
        &["ListOffice", "ListAgent"],
        Some(3),
    )?;

    match execute_query(&client, &expand_query).await {
        Ok(response) => {
            println!("\n✓ Query with expansion executed successfully:");
            if let Some(records) = response["value"].as_array() {
                println!("Found {} records with expanded data\n", records.len());
                for (i, record) in records.iter().enumerate() {
                    println!("Property {}:", i + 1);
                    print_expanded_property(record);
                    println!();
                }
            }
        }
        Err(e) => {
            println!("\nNote: Expand query failed: {}", e);
            println!("This is normal if your RESO server doesn't support $expand.");
            println!("Many RESO servers, including the reference server, don't support expansion.");
        }
    }

    // Example 3: Complex query combining multiple features
    println!("\nExample 3: Complex query - Filter + Select + Order + Limit...");
    println!("{}", "-".repeat(60));
    let complex_query = reso_examples::build_query_with_order(
        "Property",
        Some("ListPrice gt 300000 and BedroomsTotal ge 3"),
        "ListPrice",
        "asc",  // Ascending order (lowest to highest)
        Some(5),
    )?;

    let complex_response = execute_query(&client, &complex_query).await?;
    if let Some(records) = complex_response["value"].as_array() {
        println!("✓ Found {} properties over $300k with 3+ bedrooms (lowest prices first)\n", records.len());
        for (i, record) in records.iter().enumerate() {
            println!("Property {}:", i + 1);
            if let Some(key) = record["ListingKey"].as_str() {
                println!("  ListingKey: {}", key);
            }
            if let Some(city) = record["City"].as_str() {
                println!("  City: {}", city);
            }
            if let Some(price) = record["ListPrice"].as_f64() {
                println!("  Price: ${:.0}", price);
            }
            if let Some(beds) = record["BedroomsTotal"].as_i64() {
                println!("  Bedrooms: {}", beds);
            }
            println!();
        }
    }

    println!("✓ Advanced query examples completed!");
    println!("\nKey Takeaways:");
    println!("  - Key-based lookups are more efficient for single records");
    println!("  - $expand support varies by RESO server implementation");
    println!("  - Complex queries can combine filters, ordering, and limits");
    println!("  - Always check server capabilities in metadata");
    println!("\nFor bulk data operations, see: cargo run --example replication_sync");

    Ok(())
}

fn print_expanded_property(property: &JsonValue) {
    if let Some(key) = property["ListingKey"].as_str() {
        println!("  ListingKey: {}", key);
    }
    if let Some(city) = property["City"].as_str() {
        println!("  City: {}", city);
    }
    if let Some(price) = property["ListPrice"].as_f64() {
        println!("  Price: ${:.0}", price);
    }

    // Check for expanded ListOffice
    if let Some(office) = property.get("ListOffice") {
        if !office.is_null() {
            println!("  ListOffice (expanded):");
            if let Some(office_name) = office["OfficeName"].as_str() {
                println!("    Name: {}", office_name);
            }
            if let Some(office_key) = office["OfficeKey"].as_str() {
                println!("    Key: {}", office_key);
            }
        }
    }

    // Check for expanded ListAgent
    if let Some(agent) = property.get("ListAgent") {
        if !agent.is_null() {
            println!("  ListAgent (expanded):");
            if let Some(agent_name) = agent["MemberFullName"].as_str() {
                println!("    Name: {}", agent_name);
            }
            if let Some(agent_key) = agent["MemberKey"].as_str() {
                println!("    Key: {}", agent_key);
            }
        }
    }
}
