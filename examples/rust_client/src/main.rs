// This is a sample Rust client for the chip-in-inventory API.
// It demonstrates how to:
// 1. Fetch all resources from the API and build a hierarchical in-memory representation.
// 2. Use the in-memory data to perform a specific task, such as finding a matching routing action for a given request.

use serde::{Deserialize, Serialize};
use reqwest::Error;
use std::sync::Arc;
use futures::future::join_all;
use std::collections::HashMap;

// --- Data Structures based on OpenAPI schemas ---
// This struct is defined based on the Realm.yaml schema specification.
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct MountPoint {
    volume_size: i32,
    target: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct VirtualHost {
    // Required fields
    name: String,
    title: String,
    subdomain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    routing_chain: Option<String>,

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
    #[serde(skip_serializing)]
    fqdn: Option<String>,
}

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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Rule {
    #[serde(rename = "match")]
    match_condition: String,
    action: Action,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Proxy {
    upstream: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_scope_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Redirect {
    url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ReturnStaticText {
    content: String,
    status: u16,
}

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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SetUpstreamRequestHeader {
    name: String,
    value: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SetDownstreamResponseHeader {
    name: String,
    value: String,
}

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZoneNode {
    #[serde(flatten)]
    zone: Zone,
    subdomains: Vec<Subdomain>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HubNode {
    #[serde(flatten)]
    hub: Hub,
    services: Vec<Service>,
}

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Inventory {
    realms: Vec<RealmNode>,
}

/// Fetches the entire inventory from the API and builds a hierarchical representation.
/// This function can replace the logic that loads configuration from a YAML file.
async fn load_inventory(api_base_url: &str) -> Result<Vec<RealmNode>, Error> {
    let client = Arc::new(reqwest::Client::new());

    let realms: Vec<Realm> = client
        .get(&format!("{}/realms", api_base_url))
        .send()
        .await?
        .json()
        .await?;

    if realms.is_empty() {
        return Ok(vec![]);
    }

    // For each realm, fetch its sub-resources and build an in-memory representation.
    let mut tasks = Vec::new();
    for realm in realms {
        let client_clone = Arc::clone(&client);
        let api_base_url_clone = api_base_url.to_string();
        let task = tokio::spawn(async move {
            // --- Fetch Zones and their Subdomains ---
            let mut zone_nodes = Vec::new();
            let zones_url = format!("{}/realms/{}/zones", api_base_url_clone, realm.name);
            if let Ok(zones) = async { client_clone.get(&zones_url).send().await?.json::<Vec<Zone>>().await }.await {
                let mut sub_tasks = Vec::new();
                for zone in zones {
                    let client_clone2 = Arc::clone(&client_clone);
                    let api_base_url_clone2 = api_base_url_clone.to_string();
                    let realm_name = realm.name.clone();
                    sub_tasks.push(tokio::spawn(async move {
                        let mut subdomains = Vec::new();
                        let subdomains_url = format!("{}/realms/{}/zones/{}/subdomains", api_base_url_clone2, realm_name, zone.name);
                        if let Ok(fetched_subdomains) = async { client_clone2.get(&subdomains_url).send().await?.json::<Vec<Subdomain>>().await }.await {
                           subdomains = fetched_subdomains;
                        }
                        ZoneNode { zone, subdomains }
                    }));
                }
                zone_nodes = join_all(sub_tasks).await.into_iter().filter_map(Result::ok).collect();
            }

            // --- Fetch Hubs and their Services ---
            let mut hub_nodes = Vec::new();
            let hubs_url = format!("{}/realms/{}/hubs", api_base_url_clone, realm.name);
            if let Ok(hubs) = async { client_clone.get(&hubs_url).send().await?.json::<Vec<Hub>>().await }.await {
                let mut sub_tasks = Vec::new();
                for hub in hubs {
                    let client_clone2 = Arc::clone(&client_clone);
                    let api_base_url_clone2 = api_base_url_clone.to_string();
                    let realm_name = realm.name.clone();
                    sub_tasks.push(tokio::spawn(async move {
                        let mut services = Vec::new();
                        let services_url = format!("{}/realms/{}/hubs/{}/services", api_base_url_clone2, realm_name, hub.name);
                        if let Ok(fetched_services) = async { client_clone2.get(&services_url).send().await?.json::<Vec<Service>>().await }.await {
                            services = fetched_services;
                        }
                        HubNode { hub, services }
                    }));
                }
                hub_nodes = join_all(sub_tasks).await.into_iter().filter_map(Result::ok).collect();
            }

            // --- Fetch Virtual Hosts ---
            let vhosts_url = format!("{}/realms/{}/virtual-hosts", api_base_url_clone, realm.name);
            let virtual_hosts = async { client_clone.get(&vhosts_url).send().await?.json::<Vec<VirtualHost>>().await }
                .await
                .unwrap_or_default();

            // --- Fetch Routing Chains ---
            let rchains_url = format!("{}/realms/{}/routing-chains", api_base_url_clone, realm.name);
            let routing_chains = async { client_clone.get(&rchains_url).send().await?.json::<Vec<RoutingChain>>().await }
                .await
                .unwrap_or_default();

            // Return the fully populated in-memory realm object
            RealmNode {
                realm,
                zones: zone_nodes,
                hubs: hub_nodes,
                virtual_hosts,
                routing_chains,
            }
        });
        tasks.push(task);
    }

    // Wait for all realm-specific tasks to complete and collect the results
    let realm_nodes: Vec<RealmNode> = join_all(tasks).await.into_iter().filter_map(Result::ok).collect();
    Ok(realm_nodes)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_base_url = "http://localhost:3000/v1";

    // --- Phase 1: Fetching all data from the API ---
    println!("--- Phase 1: Fetching all data from the API ---\n");
    println!("Connecting to {} ...", api_base_url);

    let realm_nodes = load_inventory(api_base_url).await?;
    println!("Successfully loaded inventory with {} realm(s).", realm_nodes.len());

    println!("\n--- Phase 2: Usage Phase (Displaying hierarchical data) ---\n");

    let inventory = Inventory { realms: realm_nodes.into_iter().collect() };

    match serde_yaml::to_string(&inventory) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => eprintln!("Failed to serialize to YAML: {}", e),
    }

    // --- Phase 3: Usage Phase (Find a matching action from a routing-chain) ---
    println!("\n--- Phase 3: Usage Phase (Find a matching action) ---\n");

    // Sample input: FQDN and path
    let sample_fqdn = "sirius.sr.example.com";
    let sample_path = "/api/v1/products";

    println!("Searching for an action for FQDN: '{}' and Path: '{}'...", sample_fqdn, sample_path);

    match find_matching_action(&inventory.realms, sample_fqdn, sample_path) {
        Some(action) => {
            println!("\nFound a matching action:");
            match serde_yaml::to_string(&action) {
                Ok(yaml) => println!("{}", yaml),
                Err(e) => eprintln!("Failed to serialize action to YAML: {}", e),
            }
        }
        None => {
            println!("\nNo matching action found for the given FQDN and path.");
        }
    }

    Ok(())
}

/// Finds a matching action for a given FQDN and path from the in-memory data.
fn find_matching_action(realm_nodes: &[RealmNode], fqdn: &str, path: &str) -> Option<Action> {
    for r_node in realm_nodes {
        // 1. Find the VirtualHost that matches the FQDN.
        if let Some(vhost) = r_node.virtual_hosts.iter().find(|v| v.fqdn.as_deref() == Some(fqdn)) {
            // 2. Find the RoutingChain associated with the VirtualHost.
            if let Some(rc_name) = &vhost.routing_chain {
                if let Some(rchain) = r_node.routing_chains.iter().find(|rc| &rc.name == rc_name) {
                    // 3. Iterate through the rules of the RoutingChain to find a match.
                    for rule in &rchain.rules {
                        if rule_matches(&rule.match_condition, fqdn, path) {
                            // 4. If a rule matches, return its action.
                            return Some(rule.action.clone());
                        }
                    }
                }
            }
        }
    }
    // If no match is found, return None.
    None
}

/// A simple helper to evaluate if a rule's match condition is met.
/// This is a placeholder implementation.
fn rule_matches(match_condition: &str, _fqdn: &str, _path: &str) -> bool {
    // In a real implementation, you would parse the 'match_condition' string (e.g., using a parser combinator or a scripting engine)
    // and evaluate it against the request context (FQDN, path, headers, etc.).
    // For this example, we'll just print the condition and return true to demonstrate the flow.
    println!("    [Mock Evaluation] Checking condition: '{}'", match_condition);
    true
}