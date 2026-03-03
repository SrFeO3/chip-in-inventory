// This example Rust client demonstrates how to interact with the chip-in-inventory API.
// It performs two main tasks:
// 1. Fetches all resources from the API to build a complete, hierarchical in-memory representation of the inventory.
// 2. Uses this in-memory data to find a matching routing action for a given request FQDN and path.

use serde::{Deserialize, Serialize};
use reqwest::Error;
use std::sync::Arc;
use futures::future::join_all;
use std::collections::HashMap;

// --- Data Structures based on OpenAPI schemas ---
// Data structure for a Realm, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Realm {
    // Required fields
    name: String,
    title: String,
    cacert: String,
    device_id_signing_key: String,
    device_id_verification_key: String,
    disabled: bool,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    administrators: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expired_at: Option<String>,

    // readOnly field
    #[serde(skip_serializing_if = "Option::is_none")]
    urn: Option<String>,
}

// Data structure for a Zone, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Zone {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dns_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    acme_certificate_provider: Option<String>,

    // readOnly fields
    #[serde(skip_serializing_if = "Option::is_none")]
    urn: Option<String>,
}

// Data structure for a Subdomain, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Subdomain {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination_realm: Option<String>,
    #[serde(default)] // The schema has a default value of false
    share_cookie: bool,

    // readOnly fields
    #[serde(skip_serializing_if = "Option::is_none")]
    urn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fqdn: Option<String>,
}

// Data structure for a Hub, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Hub {
    // Required fields
    name: String,
    title: String,
    fqdn: String,
    server_cert: String,
    server_cert_key: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    attributes: serde_json::Value,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_port: Option<u16>,

    // readOnly fields
    #[serde(skip_serializing_if = "Option::is_none")]
    urn: Option<String>,
}

// Data structure for a Service, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Service {
    // Required fields
    name: String,
    title: String,
    provider: String,
    consumers: Vec<String>,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    availability_management: Option<AvailabilityManagement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    singleton: Option<bool>,

    // readOnly fields
    #[serde(skip_serializing_if = "Option::is_none")]
    urn: Option<String>,
}

// Data structure for AvailabilityManagement, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AvailabilityManagement {
    cluster_manager_urn: String,
    service_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ondemand_start_on_consumer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ondemand_start_on_payload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idle_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mount_points: Option<Vec<MountPoint>>,
}

// Data structure for a MountPoint, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct MountPoint {
    volume_size: i32,
    target: String,
}

// Data structure for a VirtualHost, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Allow unused fields that are needed for deserialization
struct VirtualHost {
    // Required fields
    name: String,
    title: String,
    subdomain: String,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_log_recorder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_log_max_value_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_log_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    certificate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    // readOnly fields
    // These fields are included in the server's response.
    // `fqdn` is used in the client's logic, while the others are present to ensure successful deserialization.
    // They are marked with `#[serde(skip_serializing)]` to exclude them from the final YAML output.
    #[serde(skip_serializing)]
    fqdn: Option<String>,
    #[serde(skip_serializing)]
    realm: Option<String>,
    #[serde(skip_serializing)]
    urn: Option<String>,
    #[serde(skip_serializing)]
    created_at: Option<String>,
    #[serde(skip_serializing)]
    updated_at: Option<String>,
}

// Data structure for a RoutingChain, based on the OpenAPI schema.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RoutingChain {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default)]
    rules: Vec<Rule>,
}

// Data structure for a Rule within a RoutingChain.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Rule {
    #[serde(rename = "match")]
    match_condition: String,
    action: Action,
}

// Data structure for a Proxy action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Proxy {
    upstream: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_scope_name: Option<String>,
}

// Data structure for a Redirect action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Redirect {
    url: String,
}

// Data structure for a ReturnStaticText action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ReturnStaticText {
    content: String,
    status: u16,
}

// Data structure for a RequireAuthentication action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RequireAuthentication {
    auth_scope_name: String,
    protected_upstream: String,
    oidc_client_id: String,
    oidc_client_secret: String,
    oidc_authorization_endpoint: String,
    oidc_redirect_url: String,
    oidc_token_endpoint: String,
}

// Data structure for a SetUpstreamRequestHeader action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SetUpstreamRequestHeader {
    name: String,
    value: String,
}

// Data structure for a SetDownstreamResponseHeader action.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SetDownstreamResponseHeader {
    name: String,
    value: String,
}

// Enum representing all possible routing Actions.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Action {
    Proxy(Proxy),
    Redirect(Redirect),
    ReturnStaticText(ReturnStaticText),
    RequireAuthentication(RequireAuthentication),
    SetUpstreamRequestHeader(SetUpstreamRequestHeader),
    SetDownstreamResponseHeader(SetDownstreamResponseHeader),
}

// --- In-memory representation for hierarchical data (Nodes) ---

/// Represents a Zone and its associated Subdomains.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZoneNode {
    #[serde(flatten)]
    zone: Zone,
    subdomains: Vec<Subdomain>,
}

/// Represents a Hub and its associated Services.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HubNode {
    #[serde(flatten)]
    hub: Hub,
    services: Vec<Service>,
}

/// Represents a Realm and all its nested resources.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RealmNode {
    #[serde(flatten)]
    realm: Realm,
    zones: Vec<ZoneNode>,
    hubs: Vec<HubNode>,
    virtual_hosts: Vec<VirtualHost>,
    routing_chains: Vec<RoutingChain>,
}

/// The root of the in-memory inventory structure.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Inventory {
    realms: Vec<RealmNode>,
}

/// Fetches all subdomains for a given zone.
async fn fetch_subdomains_for_zone(client: &reqwest::Client, api_base_url: &str, realm_name: &str, zone: Zone) -> Result<ZoneNode, Error> {
    let subdomains_url = format!("{}/realms/{}/zones/{}/subdomains", api_base_url, realm_name, zone.name);
    let subdomains = client.get(&subdomains_url).send().await?.json::<Vec<Subdomain>>().await?;
    Ok(ZoneNode { zone, subdomains })
}

/// Fetches all zones and their subdomains for a given realm.
async fn fetch_zones_for_realm(client: &reqwest::Client, api_base_url: &str, realm_name: &str) -> Result<Vec<ZoneNode>, Error> {
    let zones_url = format!("{}/realms/{}/zones", api_base_url, realm_name);
    let zones = client.get(&zones_url).send().await?.json::<Vec<Zone>>().await?;

    let tasks = zones.into_iter().map(|zone| {
        fetch_subdomains_for_zone(client, api_base_url, realm_name, zone)
    });

    let results = join_all(tasks).await;
    results.into_iter().collect() // Convert Vec<Result<T, E>> to Result<Vec<T>, E>
}

/// Fetches all services for a given hub.
async fn fetch_services_for_hub(client: &reqwest::Client, api_base_url: &str, realm_name: &str, hub: Hub) -> Result<HubNode, Error> {
    let services_url = format!("{}/realms/{}/hubs/{}/services", api_base_url, realm_name, hub.name);
    let services = client.get(&services_url).send().await?.json::<Vec<Service>>().await?;
    Ok(HubNode { hub, services })
}

/// Fetches all hubs and their services for a given realm.
async fn fetch_hubs_for_realm(client: &reqwest::Client, api_base_url: &str, realm_name: &str) -> Result<Vec<HubNode>, Error> {
    let hubs_url = format!("{}/realms/{}/hubs", api_base_url, realm_name);
    let hubs = client.get(&hubs_url).send().await?.json::<Vec<Hub>>().await?;

    let tasks = hubs.into_iter().map(|hub| {
        fetch_services_for_hub(client, api_base_url, realm_name, hub)
    });

    let results = join_all(tasks).await;
    results.into_iter().collect()
}

/// Fetches all sub-resources for a given realm and builds a RealmNode.
async fn build_realm_node(client: Arc<reqwest::Client>, api_base_url: String, realm: Realm) -> Result<RealmNode, Error> {
    let realm_name = realm.name.clone();

    // Use try_join! to run futures in parallel and propagate errors.
    let (zones, hubs, virtual_hosts, routing_chains) = futures::try_join!(
        fetch_zones_for_realm(&client, &api_base_url, &realm_name),
        fetch_hubs_for_realm(&client, &api_base_url, &realm_name),
        async {
            let url = format!("{}/realms/{}/virtual-hosts", api_base_url, realm_name);
            client.get(&url).send().await?.json::<Vec<VirtualHost>>().await
        },
        async {
            let url = format!("{}/realms/{}/routing-chains", api_base_url, realm_name);
            client.get(&url).send().await?.json::<Vec<RoutingChain>>().await
        }
    )?;

    Ok(RealmNode {
        realm,
        zones,
        hubs,
        virtual_hosts,
        routing_chains,
    })
}

/// Fetches the entire inventory from the API and builds a hierarchical in-memory representation.
async fn load_inventory(api_base_url: &str) -> Result<Vec<RealmNode>, Error> {
    let client = Arc::new(reqwest::Client::new());

    let realms_url = format!("{}/realms", api_base_url);
    let realms: Vec<Realm> = client.get(&realms_url).send().await?.json().await?;

    if realms.is_empty() {
        return Ok(vec![]);
    }

    // For each realm, spawn a task to fetch its sub-resources in parallel.
    let tasks = realms.into_iter().map(|realm| {
        let client_clone = Arc::clone(&client);
        let api_base_url_clone = api_base_url.to_string();
        tokio::spawn(build_realm_node(client_clone, api_base_url_clone, realm))
    });

    // Wait for all realm-specific tasks to complete and collect the results
    let results: Vec<Result<RealmNode, Error>> = join_all(tasks).await.into_iter()
        .map(|res| res.unwrap()) // Unwrap the JoinHandle result
        .collect();

    // Convert Vec<Result<T, E>> to a single Result<Vec<T>, E>
    results.into_iter().collect()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_base_url = "http://localhost:3000/v1";

    // --- Phase 1: Fetch all data from the API ---
    println!("--- Phase 1: Fetching all data from the API ---\n");
    println!("Connecting to {} ...", api_base_url);

    let realm_nodes = load_inventory(api_base_url).await?;
    println!("Successfully loaded inventory with {} realm(s).", realm_nodes.len());

    println!("\n--- Phase 2: Displaying the fetched hierarchical data as YAML ---\n");

    let inventory = Inventory { realms: realm_nodes.into_iter().collect() };

    match serde_yaml::to_string(&inventory) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => eprintln!("Error: Failed to serialize inventory to YAML: {}", e),
    }

    // --- Phase 3: Find a matching action from a routing chain ---
    println!("\n--- Phase 3: Finding a matching routing action ---\n");

    // Sample input: FQDN and path
    let sample_fqdn = "sirius.sr.example.com";
    let sample_path = "/api/v1/products";

    println!("Searching for an action for FQDN: '{}' and Path: '{}'...", sample_fqdn, sample_path);

    // Find the realm that contains the virtual host matching the FQDN.
    // Then, get its (single) routing chain and find the first rule that matches the request.
    let action = inventory.realms.iter()
        .find(|r_node| r_node.virtual_hosts.iter().any(|v| v.fqdn.as_deref() == Some(sample_fqdn)))
        .and_then(|r_node| r_node.routing_chains.first())
        .and_then(|rchain| {
            rchain.rules.iter()
                .find(|rule| {
                    // This is a placeholder for rule evaluation logic.
                    // A real implementation would parse and evaluate the `match_condition`.
                    // For this example, we just print the condition and return true to find the first action.
                    println!("    [Mock Evaluation] Checking condition: '{}'", &rule.match_condition);
                    true
                })
                .map(|rule| rule.action.clone())
        });

    match action {
        Some(action) => {
            println!("\nFound a matching action:");
            match serde_yaml::to_string(&action) {
                Ok(yaml) => println!("{}", yaml),
                Err(e) => eprintln!("Error: Failed to serialize action to YAML: {}", e),
            }
        }
        None => {
            println!("\nNo matching action found for the given FQDN and path.");
        }
    }

    Ok(())
}
