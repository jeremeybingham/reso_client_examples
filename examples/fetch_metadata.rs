//! Example: Fetch RESO Metadata XML
//!
//! This example demonstrates how to connect to a RESO Web API server
//! and fetch the metadata XML document.
//!
//! The metadata document describes all available resources (Property, Member, Office, etc.)
//! and their fields, data types, and relationships.
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
//! cargo run --example fetch_metadata
//! ```

use reso_examples::{load_env, create_client, fetch_metadata};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    load_env()?;

    println!("=== RESO Metadata Fetcher ===\n");

    // Create RESO client from environment variables
    println!("Creating RESO client from environment...");
    let client = create_client()?;
    println!("✓ Client created successfully\n");

    // Fetch metadata XML document
    println!("Fetching metadata from server...");
    let metadata_xml = fetch_metadata(&client).await?;
    println!("✓ Metadata fetched successfully");
    println!("  Metadata size: {} bytes", metadata_xml.len());
    println!("  Metadata size: {:.2} KB\n", metadata_xml.len() as f64 / 1024.0);

    // Save metadata to file
    let output_file = "metadata.xml";
    println!("Saving metadata to {}...", output_file);
    fs::write(output_file, &metadata_xml)?;
    println!("✓ Metadata saved successfully\n");

    // Display first few lines of metadata
    println!("First 500 characters of metadata:");
    println!("{}", "-".repeat(60));
    let preview = if metadata_xml.len() > 500 {
        &metadata_xml[..500]
    } else {
        &metadata_xml
    };
    println!("{}", preview);
    if metadata_xml.len() > 500 {
        println!("...\n(truncated)");
    }
    println!("{}", "-".repeat(60));

    println!("\n✓ Complete! Metadata saved to {}", output_file);
    println!("\nYou can now:");
    println!("  - Open {} to view the full metadata XML", output_file);
    println!("  - Use the metadata to discover available resources and fields");
    println!("  - Run 'cargo run --example query_properties' to query property data");

    Ok(())
}
