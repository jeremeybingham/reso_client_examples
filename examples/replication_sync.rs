//! Example: RESO Data Replication
//!
//! This example demonstrates how to use the replication endpoint for bulk
//! data synchronization from a RESO Web API server.
//!
//! Replication queries are designed for efficiently downloading large datasets
//! with pagination support via continuation tokens.
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
//! cargo run --example replication_sync
//! ```
//!
//! ## Note
//!
//! Not all RESO servers support replication endpoints. If your server doesn't
//! support replication, you'll see an error message explaining this.

use reso_examples::{
    load_env, create_client, build_replication_query, execute_replication_query,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    load_env()?;

    println!("=== RESO Replication Example ===\n");

    // Create RESO client from environment variables
    println!("Creating RESO client from environment...");
    let client = create_client()?;
    println!("✓ Client created successfully\n");

    println!("Important: Replication endpoints require server support and authorization.");
    println!("If your server doesn't support replication, you'll see an error.\n");

    // Example 1: Basic replication query
    println!("Example 1: Replicating all active properties...");
    println!("{}", "-".repeat(60));

    let replication_query = build_replication_query(
        "Property",
        Some("StandardStatus eq 'Active'"),
    )?;

    match execute_replication_query(&client, &replication_query).await {
        Ok(response) => {
            println!("✓ Replication query executed successfully!");
            println!("  Records in this batch: {}", response.records.len());

            if let Some(link) = &response.next_link {
                println!("  Next link available: {}", link);
                println!("  More records are available - use link for next batch");
            } else {
                println!("  No next link - this is the complete dataset");
            }

            // Display first few records
            if !response.records.is_empty() {
                println!("\nFirst 3 records:");
                for (i, record) in response.records.iter().take(3).enumerate() {
                    println!("\nRecord {}:", i + 1);
                    println!("{}", serde_json::to_string_pretty(record)?);
                }

                if response.records.len() > 3 {
                    println!("\n... and {} more records in this batch", response.records.len() - 3);
                }
            }

            // Demonstrate pagination concept
            if let Some(link) = &response.next_link {
                println!("\n{}", "=".repeat(60));
                println!("PAGINATION CONCEPT");
                println!("{}", "=".repeat(60));
                println!("To get the next batch of records, you would:");
                println!("1. Store the next_link: {}", link);
                println!("2. Parse the link to extract pagination parameters");
                println!("3. Create a new query with those parameters");
                println!("4. Execute the query to get the next batch");
                println!("5. Repeat until next_link is None");
                println!("\nNote: The exact pagination mechanism depends on");
                println!("      your RESO server's implementation.");
            }
        }
        Err(e) => {
            println!("\n❌ Replication query failed: {}", e);
            println!("\nThis is normal if:");
            println!("  - Your RESO server doesn't support replication endpoints");
            println!("  - Your credentials don't have replication permissions");
            println!("  - The server requires additional authorization for replication");
            println!("\nAlternative: Use standard queries with pagination:");
            println!("  cargo run --example query_properties");
        }
    }

    // Example 2: Replication without filter (all records)
    println!("\n\nExample 2: Replicating all properties (no filter)...");
    println!("{}", "-".repeat(60));
    println!("Note: This can return a very large dataset!");

    let full_replication_query = build_replication_query("Property", None)?;

    match execute_replication_query(&client, &full_replication_query).await {
        Ok(response) => {
            println!("✓ Full replication query executed successfully!");
            println!("  Records in this batch: {}", response.records.len());

            if response.next_link.is_some() {
                println!("  ⚠️  More batches available - full dataset would require pagination");
            } else {
                println!("  Total dataset size: {} records", response.records.len());
            }
        }
        Err(e) => {
            println!("Note: Full replication query failed: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("REPLICATION BEST PRACTICES");
    println!("{}", "=".repeat(60));
    println!("1. Use filters to limit data volume when possible");
    println!("2. Implement pagination to handle large datasets");
    println!("3. Store records incrementally to avoid memory issues");
    println!("4. Use ModificationTimestamp filters for incremental updates");
    println!("5. Check server documentation for rate limits");
    println!("6. Consider using standard queries if replication isn't available");

    println!("\n✓ Replication example completed!");
    println!("\nFor standard query examples, see:");
    println!("  - cargo run --example query_properties");
    println!("  - cargo run --example advanced_queries");

    Ok(())
}
