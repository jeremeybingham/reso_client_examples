//! Example: Axum HTTP Service for Property Search
//!
//! This example demonstrates a simple HTTP service using Axum that provides
//! a form-based interface for searching RESO property data.
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
//! cargo run --example axum_property_search
//! ```
//!
//! Then open your browser to: http://localhost:3000

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use reso_client::ResoClient;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

// Fields we want to query and display
const PROPERTY_FIELDS: &[&str] = &[
    "ListingKey",
    "ListingId",
    "StandardStatus",
    "MlsStatus",
    "ListPrice",
    "UnparsedAddress",
    "StreetNumber",
    "StreetName",
    "City",
    "StateOrProvince",
    "PostalCode",
    "PropertyType",
    "PropertySubType",
    "BedroomsTotal",
    "BathroomsTotalInteger",
    "LivingArea",
    "LotSizeSquareFeet",
    "LotSizeAcres",
    "YearBuilt",
    "ListingContractDate",
    "ModificationTimestamp",
    "PhotosCount",
    "PublicRemarks",
];

#[derive(Clone)]
struct AppState {
    client: Arc<ResoClient>,
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    #[serde(default)]
    city: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    min_price: String,
    #[serde(default)]
    max_price: String,
    #[serde(default)]
    min_beds: String,
    #[serde(default)]
    max_beds: String,
    #[serde(default)]
    min_baths: String,
    #[serde(default)]
    property_type: String,
    #[serde(default)]
    limit: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    reso_examples::load_env()?;

    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== RESO Property Search Web Service ===\n");

    // Create RESO client
    println!("Creating RESO client from environment...");
    let client = reso_examples::create_client()?;
    println!("✓ Client created successfully\n");

    // Create shared state
    let state = AppState {
        client: Arc::new(client),
    };

    // Build the router
    let app = Router::new()
        .route("/", get(home_page))
        .route("/search", get(search_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await?;
    println!("🚀 Server running at http://127.0.0.1:3030");
    println!("   Press Ctrl+C to stop\n");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn home_page() -> Html<String> {
    Html(render_search_form(None, None))
}

async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Response {
    // Build filter expression from search parameters
    let mut filters = Vec::new();

    if !params.city.is_empty() {
        filters.push(format!("City eq '{}'", params.city));
    }

    if !params.state.is_empty() {
        filters.push(format!("StateOrProvince eq '{}'", params.state));
    }

    if !params.status.is_empty() {
        filters.push(format!("StandardStatus eq '{}'", params.status));
    }

    if !params.min_price.is_empty() {
        if let Ok(price) = params.min_price.parse::<i64>() {
            filters.push(format!("ListPrice ge {}", price));
        }
    }

    if !params.max_price.is_empty() {
        if let Ok(price) = params.max_price.parse::<i64>() {
            filters.push(format!("ListPrice le {}", price));
        }
    }

    if !params.min_beds.is_empty() {
        if let Ok(beds) = params.min_beds.parse::<i64>() {
            filters.push(format!("BedroomsTotal ge {}", beds));
        }
    }

    if !params.max_beds.is_empty() {
        if let Ok(beds) = params.max_beds.parse::<i64>() {
            filters.push(format!("BedroomsTotal le {}", beds));
        }
    }

    if !params.min_baths.is_empty() {
        if let Ok(baths) = params.min_baths.parse::<i64>() {
            filters.push(format!("BathroomsTotalInteger ge {}", baths));
        }
    }

    if !params.property_type.is_empty() {
        filters.push(format!("PropertyType eq '{}'", params.property_type));
    }

    let filter_str = if filters.is_empty() {
        None
    } else {
        Some(filters.join(" and "))
    };

    // Parse limit or default to 10
    let limit = params
        .limit
        .parse::<u32>()
        .unwrap_or(10)
        .min(100); // Cap at 100 results

    // Build and execute query
    let query = match reso_examples::build_query_with_select(
        "Property",
        filter_str.as_deref(),
        PROPERTY_FIELDS,
        Some(limit),
    ) {
        Ok(q) => q,
        Err(e) => {
            return Html(render_search_form(
                None,
                Some(&format!("Error building query: {}", e)),
            ))
            .into_response();
        }
    };

    match reso_examples::execute_query(&state.client, &query).await {
        Ok(response) => {
            Html(render_search_form(Some(&response), None)).into_response()
        }
        Err(e) => {
            Html(render_search_form(
                None,
                Some(&format!("Error executing query: {}", e)),
            ))
            .into_response()
        }
    }
}

fn render_search_form(results: Option<&JsonValue>, error: Option<&str>) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RESO Property Search</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: #f5f5f5;
            padding: 20px;
            line-height: 1.6;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            margin-bottom: 30px;
            padding-bottom: 15px;
            border-bottom: 3px solid #007bff;
        }
        .search-form {
            margin-bottom: 30px;
        }
        .form-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }
        .form-group {
            display: flex;
            flex-direction: column;
        }
        label {
            font-weight: 600;
            margin-bottom: 5px;
            color: #555;
            font-size: 14px;
        }
        input, select {
            padding: 8px 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
            transition: border-color 0.2s;
        }
        input:focus, select:focus {
            outline: none;
            border-color: #007bff;
        }
        button {
            background: #007bff;
            color: white;
            padding: 10px 30px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            font-weight: 600;
            transition: background 0.2s;
        }
        button:hover {
            background: #0056b3;
        }
        .results {
            margin-top: 30px;
        }
        .result-count {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 20px;
            color: #333;
        }
        .property-card {
            border: 1px solid #e0e0e0;
            border-radius: 6px;
            padding: 20px;
            margin-bottom: 20px;
            background: #fafafa;
            transition: box-shadow 0.2s;
        }
        .property-card:hover {
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
        }
        .property-header {
            display: flex;
            justify-content: space-between;
            align-items: start;
            margin-bottom: 15px;
            padding-bottom: 15px;
            border-bottom: 2px solid #e0e0e0;
        }
        .property-address {
            font-size: 18px;
            font-weight: 600;
            color: #333;
        }
        .property-price {
            font-size: 24px;
            font-weight: 700;
            color: #28a745;
        }
        .property-details {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 10px;
            margin-bottom: 15px;
        }
        .detail-item {
            padding: 8px;
            background: white;
            border-radius: 4px;
        }
        .detail-label {
            font-size: 12px;
            color: #666;
            font-weight: 600;
            text-transform: uppercase;
        }
        .detail-value {
            font-size: 14px;
            color: #333;
            margin-top: 2px;
        }
        .property-remarks {
            padding: 15px;
            background: white;
            border-radius: 4px;
            margin-top: 15px;
            color: #555;
            line-height: 1.6;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
        }
        .status-active {
            background: #d4edda;
            color: #155724;
        }
        .status-pending {
            background: #fff3cd;
            color: #856404;
        }
        .status-closed {
            background: #f8d7da;
            color: #721c24;
        }
        .error {
            background: #f8d7da;
            color: #721c24;
            padding: 15px;
            border-radius: 4px;
            border: 1px solid #f5c6cb;
            margin-bottom: 20px;
        }
        .no-results {
            text-align: center;
            padding: 40px;
            color: #666;
            font-size: 18px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🏠 RESO Property Search</h1>

        <form class="search-form" method="GET" action="/search">
            <div class="form-grid">
                <div class="form-group">
                    <label for="city">City</label>
                    <input type="text" id="city" name="city" placeholder="e.g., Austin">
                </div>

                <div class="form-group">
                    <label for="state">State/Province</label>
                    <input type="text" id="state" name="state" placeholder="e.g., TX">
                </div>

                <div class="form-group">
                    <label for="status">Status</label>
                    <select id="status" name="status">
                        <option value="">Any</option>
                        <option value="Active">Active</option>
                        <option value="Pending">Pending</option>
                        <option value="Closed">Closed</option>
                        <option value="Expired">Expired</option>
                    </select>
                </div>

                <div class="form-group">
                    <label for="property_type">Property Type</label>
                    <select id="property_type" name="property_type">
                        <option value="">Any</option>
                        <option value="Residential">Residential</option>
                        <option value="Commercial">Commercial</option>
                        <option value="Land">Land</option>
                        <option value="Multi-Family">Multi-Family</option>
                    </select>
                </div>

                <div class="form-group">
                    <label for="min_price">Min Price ($)</label>
                    <input type="number" id="min_price" name="min_price" placeholder="e.g., 100000">
                </div>

                <div class="form-group">
                    <label for="max_price">Max Price ($)</label>
                    <input type="number" id="max_price" name="max_price" placeholder="e.g., 500000">
                </div>

                <div class="form-group">
                    <label for="min_beds">Min Bedrooms</label>
                    <input type="number" id="min_beds" name="min_beds" placeholder="e.g., 2">
                </div>

                <div class="form-group">
                    <label for="max_beds">Max Bedrooms</label>
                    <input type="number" id="max_beds" name="max_beds" placeholder="e.g., 5">
                </div>

                <div class="form-group">
                    <label for="min_baths">Min Bathrooms</label>
                    <input type="number" id="min_baths" name="min_baths" placeholder="e.g., 2">
                </div>

                <div class="form-group">
                    <label for="limit">Results Limit</label>
                    <input type="number" id="limit" name="limit" value="10" min="1" max="100">
                </div>
            </div>

            <button type="submit">🔍 Search Properties</button>
        </form>
"#,
    );

    // Add error message if present
    if let Some(err_msg) = error {
        html.push_str(&format!(
            r#"<div class="error">⚠️ <strong>Error:</strong> {}</div>"#,
            html_escape(err_msg)
        ));
    }

    // Add results if present
    if let Some(response) = results {
        if let Some(records) = response["value"].as_array() {
            html.push_str(&format!(
                r#"<div class="results">
                    <div class="result-count">Found {} propert{}</div>"#,
                records.len(),
                if records.len() == 1 { "y" } else { "ies" }
            ));

            if records.is_empty() {
                html.push_str(
                    r#"<div class="no-results">No properties found matching your criteria. Try adjusting your search filters.</div>"#,
                );
            } else {
                for record in records {
                    html.push_str(&render_property_card(record));
                }
            }

            html.push_str("</div>");
        }
    }

    html.push_str(
        r#"
    </div>
</body>
</html>"#,
    );

    html
}

fn render_property_card(property: &JsonValue) -> String {
    let mut card = String::from(r#"<div class="property-card">"#);

    // Header with address and price
    card.push_str(r#"<div class="property-header">"#);

    let address = property["UnparsedAddress"]
        .as_str()
        .or_else(|| {
            // Build address from components if UnparsedAddress is not available
            let street_num = property["StreetNumber"].as_str().unwrap_or("");
            let street_name = property["StreetName"].as_str().unwrap_or("");
            let _city = property["City"].as_str().unwrap_or("");
            let _state = property["StateOrProvince"].as_str().unwrap_or("");
            let _zip = property["PostalCode"].as_str().unwrap_or("");

            if !street_num.is_empty() || !street_name.is_empty() {
                Some("")
            } else {
                None
            }
        })
        .unwrap_or("Address not available");

    let full_address = if address.is_empty() {
        format!(
            "{} {}, {}, {} {}",
            property["StreetNumber"].as_str().unwrap_or(""),
            property["StreetName"].as_str().unwrap_or(""),
            property["City"].as_str().unwrap_or(""),
            property["StateOrProvince"].as_str().unwrap_or(""),
            property["PostalCode"].as_str().unwrap_or("")
        )
    } else {
        address.to_string()
    };

    card.push_str(&format!(
        r#"<div class="property-address">{}</div>"#,
        html_escape(&full_address)
    ));

    if let Some(price) = property["ListPrice"].as_f64() {
        card.push_str(&format!(
            r#"<div class="property-price">${:.0}</div>"#,
            price
        ));
    }

    card.push_str("</div>");

    // Status badge
    if let Some(status) = property["StandardStatus"].as_str() {
        let status_class = match status.to_lowercase().as_str() {
            "active" => "status-active",
            "pending" => "status-pending",
            _ => "status-closed",
        };
        card.push_str(&format!(
            r#"<div class="status-badge {}">{}</div>"#,
            status_class,
            html_escape(status)
        ));
    }

    // Property details grid
    card.push_str(r#"<div class="property-details">"#);

    let details: Vec<(&str, Option<String>)> = vec![
        ("Listing Key", property["ListingKey"].as_str().map(|s| s.to_string())),
        ("Listing ID", property["ListingId"].as_str().map(|s| s.to_string())),
        ("MLS Status", property["MlsStatus"].as_str().map(|s| s.to_string())),
        ("Property Type", property["PropertyType"].as_str().map(|s| s.to_string())),
        ("Property SubType", property["PropertySubType"].as_str().map(|s| s.to_string())),
        (
            "Bedrooms",
            property["BedroomsTotal"].as_i64().map(|v| v.to_string()),
        ),
        (
            "Bathrooms",
            property["BathroomsTotalInteger"]
                .as_i64()
                .map(|v| v.to_string()),
        ),
        (
            "Living Area",
            property["LivingArea"]
                .as_f64()
                .map(|v| format!("{:.0} sq ft", v)),
        ),
        (
            "Lot Size",
            property["LotSizeSquareFeet"]
                .as_f64()
                .map(|v| format!("{:.0} sq ft", v)),
        ),
        (
            "Lot Size (Acres)",
            property["LotSizeAcres"]
                .as_f64()
                .map(|v| format!("{:.2} acres", v)),
        ),
        ("Year Built", property["YearBuilt"].as_i64().map(|v| v.to_string())),
        ("Listing Date", property["ListingContractDate"].as_str().map(|s| s.to_string())),
        ("Last Modified", property["ModificationTimestamp"].as_str().map(|s| s.to_string())),
        (
            "Photos",
            property["PhotosCount"].as_i64().map(|v| v.to_string()),
        ),
    ];

    for (label, value) in details {
        if let Some(val) = value {
            if !val.is_empty() && val != "null" {
                card.push_str(&format!(
                    r#"<div class="detail-item">
                        <div class="detail-label">{}</div>
                        <div class="detail-value">{}</div>
                    </div>"#,
                    label,
                    html_escape(&val)
                ));
            }
        }
    }

    card.push_str("</div>");

    // Public remarks
    if let Some(remarks) = property["PublicRemarks"].as_str() {
        if !remarks.is_empty() {
            card.push_str(&format!(
                r#"<div class="property-remarks"><strong>Description:</strong><br>{}</div>"#,
                html_escape(remarks)
            ));
        }
    }

    card.push_str("</div>");
    card
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}