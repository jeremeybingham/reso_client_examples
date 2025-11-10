//! Core library for interacting with RESO Web API using the reso_client library.
//!
//! This library provides convenient functions for:
//! - Creating and configuring RESO API clients
//! - Fetching metadata from RESO servers
//! - Building and executing queries
//! - Handling common use cases

use reso_client::{ResoClient, QueryBuilder, Query, ResoError, JsonValue, ReplicationQueryBuilder, ReplicationQuery, ReplicationResponse};
use std::result::Result;

/// Creates a ResoClient from environment variables.
///
/// # Environment Variables
///
/// Required:
/// - `RESO_BASE_URL`: Base API URL (e.g., "https://api.bridgedataoutput.com/api/v2/OData")
/// - `RESO_TOKEN`: Bearer authentication token
///
/// Optional:
/// - `RESO_DATASET_ID`: Dataset identifier (e.g., "actris_ref")
/// - `RESO_TIMEOUT`: Timeout in seconds (default: 30)
///
/// # Example
///
/// ```no_run
/// use reso_examples::create_client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     Ok(())
/// }
/// ```
pub fn create_client() -> Result<ResoClient, ResoError> {
    ResoClient::from_env()
}

/// Fetches the metadata XML document from the RESO server.
///
/// The metadata document describes the available resources, fields,
/// and their types in the RESO API.
///
/// # Arguments
///
/// * `client` - A reference to a configured ResoClient
///
/// # Returns
///
/// Returns the metadata as an XML string.
///
/// # Example
///
/// ```no_run
/// use reso_examples::{create_client, fetch_metadata};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     let metadata = fetch_metadata(&client).await?;
///     println!("Metadata length: {} bytes", metadata.len());
///     Ok(())
/// }
/// ```
pub async fn fetch_metadata(client: &ResoClient) -> Result<String, ResoError> {
    client.fetch_metadata().await
}

/// Builds a simple query for a given resource.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
/// * `top` - Optional limit on number of results
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_query("Property", Some("City eq 'Austin'"), Some(10))?;
/// # Ok(())
/// # }
/// ```
pub fn build_query(
    resource: &str,
    filter: Option<&str>,
    top: Option<u32>,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    if let Some(limit) = top {
        builder = builder.top(limit);
    }

    builder.build()
}

/// Builds a query with field selection.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
/// * `fields` - Array of field names to select
/// * `top` - Optional limit on number of results
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query_with_select;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_query_with_select(
///     "Property",
///     Some("City eq 'Austin'"),
///     &["ListingKey", "City", "ListPrice"],
///     Some(10)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn build_query_with_select(
    resource: &str,
    filter: Option<&str>,
    fields: &[&str],
    top: Option<u32>,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    builder = builder.select(fields);

    if let Some(limit) = top {
        builder = builder.top(limit);
    }

    builder.build()
}

/// Executes a query and returns the JSON response.
///
/// # Arguments
///
/// * `client` - A reference to a configured ResoClient
/// * `query` - A reference to the query to execute
///
/// # Returns
///
/// Returns a JSON value containing the response data.
///
/// # Example
///
/// ```no_run
/// use reso_examples::{create_client, build_query, execute_query};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     let query = build_query("Property", Some("City eq 'Austin'"), Some(10))?;
///     let response = execute_query(&client, &query).await?;
///
///     if let Some(records) = response["value"].as_array() {
///         println!("Found {} records", records.len());
///     }
///     Ok(())
/// }
/// ```
pub async fn execute_query(client: &ResoClient, query: &Query) -> Result<JsonValue, ResoError> {
    client.execute(query).await
}

/// Executes a count-only query to get the total number of matching records.
///
/// # Arguments
///
/// * `client` - A reference to a configured ResoClient
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
///
/// # Returns
///
/// Returns the count as a u64.
///
/// # Example
///
/// ```no_run
/// use reso_examples::{create_client, count_records};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     let count = count_records(&client, "Property", Some("City eq 'Austin'")).await?;
///     println!("Total records: {}", count);
///     Ok(())
/// }
/// ```
pub async fn count_records(
    client: &ResoClient,
    resource: &str,
    filter: Option<&str>,
) -> Result<u64, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    let query = builder.count().build()?;
    client.execute_count(&query).await
}

/// Loads environment variables from a .env file.
///
/// This should be called at the start of your application before
/// attempting to create a RESO client.
///
/// # Example
///
/// ```no_run
/// use reso_examples::load_env;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     load_env()?;
///     // Now you can create a client
///     Ok(())
/// }
/// ```
pub fn load_env() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    Ok(())
}

/// Prints formatted JSON records from a query response.
///
/// # Arguments
///
/// * `response` - The JSON response from a query
///
/// # Example
///
/// ```no_run
/// use reso_examples::{create_client, build_query, execute_query, print_records};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     let query = build_query("Property", Some("City eq 'Austin'"), Some(5))?;
///     let response = execute_query(&client, &query).await?;
///     print_records(&response)?;
///     Ok(())
/// }
/// ```
pub fn print_records(response: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(records) = response["value"].as_array() {
        println!("Found {} records\n", records.len());
        for (i, record) in records.iter().enumerate() {
            println!("Record {}:", i + 1);
            println!("{}", serde_json::to_string_pretty(record)?);
            println!();
        }
    } else {
        println!("No records found or invalid response format");
    }
    Ok(())
}

/// Builds a query for a single record by key.
///
/// This is more efficient than using a filter for single-record lookups.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `key` - The key value to look up
/// * `fields` - Optional array of field names to select
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query_by_key;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_query_by_key("Property", "12345", Some(&["ListingKey", "City", "ListPrice"]))?;
/// # Ok(())
/// # }
/// ```
pub fn build_query_by_key(
    resource: &str,
    key: &str,
    fields: Option<&[&str]>,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::by_key(resource, key);

    if let Some(field_list) = fields {
        builder = builder.select(field_list);
    }

    builder.build()
}

/// Builds a query with ordering.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
/// * `order_field` - Field name to order by
/// * `direction` - Sort direction ("asc" or "desc")
/// * `top` - Optional limit on number of results
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query_with_order;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_query_with_order(
///     "Property",
///     Some("City eq 'Austin'"),
///     "ListPrice",
///     "desc",
///     Some(10)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn build_query_with_order(
    resource: &str,
    filter: Option<&str>,
    order_field: &str,
    direction: &str,
    top: Option<u32>,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    builder = builder.order_by(order_field, direction);

    if let Some(limit) = top {
        builder = builder.top(limit);
    }

    builder.build()
}

/// Builds a query with pagination support.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
/// * `fields` - Array of field names to select
/// * `skip` - Number of records to skip (for pagination)
/// * `top` - Number of records to return
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query_with_pagination;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Get second page of 10 results
/// let query = build_query_with_pagination(
///     "Property",
///     Some("City eq 'Austin'"),
///     &["ListingKey", "City", "ListPrice"],
///     10,  // Skip first 10
///     10   // Take next 10
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn build_query_with_pagination(
    resource: &str,
    filter: Option<&str>,
    fields: &[&str],
    skip: u32,
    top: u32,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    builder = builder.select(fields).skip(skip).top(top);

    builder.build()
}

/// Builds a query with expanded related entities.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
/// * `fields` - Array of field names to select
/// * `expand` - Array of navigation properties to expand
/// * `top` - Optional limit on number of results
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_query_with_expand;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_query_with_expand(
///     "Property",
///     Some("City eq 'Austin'"),
///     &["ListingKey", "City", "ListPrice"],
///     &["ListOffice", "ListAgent"],
///     Some(10)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn build_query_with_expand(
    resource: &str,
    filter: Option<&str>,
    fields: &[&str],
    expand: &[&str],
    top: Option<u32>,
) -> Result<Query, ResoError> {
    let mut builder = QueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    builder = builder.select(fields).expand(expand);

    if let Some(limit) = top {
        builder = builder.top(limit);
    }

    builder.build()
}

/// Builds a replication query for bulk data synchronization.
///
/// # Arguments
///
/// * `resource` - The resource name (e.g., "Property", "Member", "Office")
/// * `filter` - Optional OData filter expression
///
/// # Example
///
/// ```no_run
/// use reso_examples::build_replication_query;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let query = build_replication_query("Property", Some("StandardStatus eq 'Active'"))?;
/// # Ok(())
/// # }
/// ```
pub fn build_replication_query(
    resource: &str,
    filter: Option<&str>,
) -> Result<ReplicationQuery, ResoError> {
    let mut builder = ReplicationQueryBuilder::new(resource);

    if let Some(filter_expr) = filter {
        builder = builder.filter(filter_expr);
    }

    builder.build()
}

/// Executes a replication query and returns the response.
///
/// # Arguments
///
/// * `client` - A reference to a configured ResoClient
/// * `query` - A reference to the replication query to execute
///
/// # Returns
///
/// Returns a ReplicationResponse containing records and pagination token.
///
/// # Example
///
/// ```no_run
/// use reso_examples::{create_client, build_replication_query, execute_replication_query};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     let query = build_replication_query("Property", None)?;
///     let response = execute_replication_query(&client, &query).await?;
///
///     println!("Retrieved {} records", response.records.len());
///     if let Some(link) = &response.next_link {
///         println!("More records available with link: {}", link);
///     }
///     Ok(())
/// }
/// ```
pub async fn execute_replication_query(
    client: &ResoClient,
    query: &ReplicationQuery,
) -> Result<ReplicationResponse, ResoError> {
    client.execute_replication(query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query_basic() {
        let query = build_query("Property", None, None);
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_with_filter() {
        let query = build_query("Property", Some("City eq 'Austin'"), Some(10));
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_with_select() {
        let query = build_query_with_select(
            "Property",
            Some("City eq 'Austin'"),
            &["ListingKey", "City", "ListPrice"],
            Some(10),
        );
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_by_key() {
        let query = build_query_by_key("Property", "12345", Some(&["ListingKey", "City"]));
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_with_order() {
        let query = build_query_with_order(
            "Property",
            Some("City eq 'Austin'"),
            "ListPrice",
            "desc",
            Some(10),
        );
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_with_pagination() {
        let query = build_query_with_pagination(
            "Property",
            Some("City eq 'Austin'"),
            &["ListingKey", "City", "ListPrice"],
            10,
            10,
        );
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_query_with_expand() {
        let query = build_query_with_expand(
            "Property",
            Some("City eq 'Austin'"),
            &["ListingKey", "City"],
            &["ListOffice", "ListAgent"],
            Some(10),
        );
        assert!(query.is_ok());
    }

    #[test]
    fn test_build_replication_query() {
        let query = build_replication_query("Property", Some("StandardStatus eq 'Active'"));
        assert!(query.is_ok());
    }
}
