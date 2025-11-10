//! Example: Query RESO Property Data
//!
//! This example demonstrates how to query property data from a RESO Web API server
//! using various filter conditions and field selections.
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
//! cargo run --example query_properties
//! ```

use reso_examples::{
    load_env, create_client, build_query_with_select, build_query_with_order,
    build_query_with_pagination, execute_query, print_records,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    load_env()?;

    println!("=== RESO Property Query Example ===\n");

    // Create RESO client from environment variables
    println!("Creating RESO client from environment...");
    let client = create_client()?;
    println!("✓ Client created successfully\n");

    // Example 1: Query properties with specific fields
    println!("Example 1: Querying first 5 properties with selected fields...");
    println!("{}", "-".repeat(60));
    let query = build_query_with_select(
        "Property",
        None,
        &["ListingKey", "City", "ListPrice", "StandardStatus"],
        Some(5),
    )?;
    let response = execute_query(&client, &query).await?;
    print_records(&response)?;

    // Example 2: Query properties in a specific city
    println!("Example 2: Querying active properties in Austin...");
    println!("{}", "-".repeat(60));
    let austin_query = build_query_with_select(
        "Property",
        Some("City eq 'Austin' and StandardStatus eq 'Active'"),
        &["ListingKey", "City", "ListPrice", "BedroomsTotal", "StandardStatus"],
        Some(3),
    )?;
    let austin_response = execute_query(&client, &austin_query).await?;
    print_records(&austin_response)?;

    // Example 3: Query properties with price filter
    println!("Example 3: Querying properties over $500,000...");
    println!("{}", "-".repeat(60));
    let price_query = build_query_with_select(
        "Property",
        Some("ListPrice gt 500000"),
        &["ListingKey", "City", "ListPrice", "BedroomsTotal", "StandardStatus"],
        Some(3),
    )?;
    let price_response = execute_query(&client, &price_query).await?;
    print_records(&price_response)?;

    // Example 4: Complex query with multiple conditions
    println!("Example 4: Complex query - Active properties in Austin with 3+ bedrooms...");
    println!("{}", "-".repeat(60));
    let complex_query = build_query_with_select(
        "Property",
        Some("City eq 'Austin' and StandardStatus eq 'Active' and BedroomsTotal ge 3"),
        &["ListingKey", "City", "ListPrice", "BedroomsTotal", "BathroomsTotalInteger", "StandardStatus"],
        Some(3),
    )?;
    let complex_response = execute_query(&client, &complex_query).await?;
    print_records(&complex_response)?;

    // Example 5: Query with ordering by price (highest first)
    println!("Example 5: Properties ordered by price (highest to lowest)...");
    println!("{}", "-".repeat(60));
    let order_query = build_query_with_order(
        "Property",
        Some("StandardStatus eq 'Active'"),
        "ListPrice",
        "desc",
        Some(5),
    )?;
    let order_response = execute_query(&client, &order_query).await?;
    print_records(&order_response)?;

    // Example 6: Pagination - get second page of results
    println!("Example 6: Pagination - Second page of properties in Austin...");
    println!("{}", "-".repeat(60));
    println!("(Showing records 6-10 of Austin properties)");
    let pagination_query = build_query_with_pagination(
        "Property",
        Some("City eq 'Austin'"),
        &["ListingKey", "City", "ListPrice", "StandardStatus"],
        5,  // Skip first 5
        5,  // Take next 5
    )?;
    let pagination_response = execute_query(&client, &pagination_query).await?;
    print_records(&pagination_response)?;

    println!("✓ All queries completed successfully!");
    println!("\nNext steps:");
    println!("  - Modify the filters to match your specific needs");
    println!("  - Add more fields to the select array");
    println!("  - Try different ordering fields and directions");
    println!("  - Explore other resources like Member, Office, Media");
    println!("  - See reso_client-USAGE.md for more query examples");
    println!("  - Run 'cargo run --example advanced_queries' for more advanced features");

    Ok(())
}
