// This is a sample Rust client for the chip-in-inventory API.
// It demonstrates how to:
// 1. Fetch all resources from the API and build a hierarchical in-memory representation.
// 2. Use the in-memory data to perform a specific task, such as finding a matching routing action for a given request.

use serde::Deserialize;
use reqwest::Error;
use std::sync::Arc;
use futures::future::join_all;

// --- Data Structures based on OpenAPI schemas ---
// This struct is defined based on the Realm.yaml schema specification.
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Realm {
    // Required fields
    name: String,
    title: String,
    cacert: String,
    #[serde(rename = "signingKey")]
    signing_key: String,
    disabled: bool,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "sessionTimeout")]
    session_timeout: Option<u64>,
    administrators: Option<Vec<String>>,
    #[serde(rename = "expiredAt")]
    expired_at: Option<String>,

    // readOnly field
    urn: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Zone {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "dnsProvider")]
    dns_provider: Option<String>,
    #[serde(rename = "acmeCertificateProvider")]
    acme_certificate_provider: Option<String>,

    // readOnly fields
    urn: Option<String>,
    realm: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Subdomain {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "destinationRealm")]
    destination_realm: Option<String>,
    #[serde(default)] // The schema has a default value of false
    share_cookie: bool,

    // readOnly fields
    urn: Option<String>,
    zone: Option<String>,
    fqdn: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Hub {
    // Required fields
    name: String,
    title: String,
    fqdn: String,
    #[serde(rename = "serverCert")]
    server_cert: String,
    #[serde(rename = "serverCertKey")]
    server_cert_key: String,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "serverPort")]
    server_port: Option<u16>,

    // readOnly fields
    urn: Option<String>,
    realm: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Service {
    // Required fields
    name: String,
    title: String,
    realm: String,
    provider: Vec<String>,
    consumers: Vec<String>,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "availabilityManagement")]
    availability_management: Option<AvailabilityManagement>,
    singleton: Option<bool>,

    // readOnly fields
    urn: Option<String>,
    hub: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AvailabilityManagement {
    cluster_manager_urn: String,
    service_id: String,
    start_at: Option<String>,
    stop_at: Option<String>,
    ondemand_start: Option<bool>,
    idle_timeout: Option<i32>,
    image: Option<String>,
    command: Option<Vec<String>>,
    env: Option<Vec<EnvVar>>,
    mount_points: Option<Vec<MountPoint>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct EnvVar {
    name: String,
    value: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct MountPoint {
    #[serde(rename = "volumeSize")]
    volume_size: i32,
    target: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct VirtualHost {
    // Required fields
    name: String,
    title: String,
    subdomain: String,
    #[serde(rename = "routingChain")]
    routing_chain: String,

    // Optional fields
    description: Option<String>,
    #[serde(rename = "accessLogRecorder")]
    access_log_recorder: Option<String>,
    #[serde(rename = "accessLogMaxValueLength")]
    access_log_max_value_length: Option<i32>,
    #[serde(rename = "accessLogFormat")]
    access_log_format: Option<serde_json::Value>,
    certificate: Option<Vec<String>>,
    key: Option<String>,
    disabled: Option<bool>,

    // readOnly fields
    urn: Option<String>,
    realm: Option<String>,
    fqdn: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct RoutingChain {
    // Required fields
    name: String,
    title: String,

    // Optional fields
    description: Option<String>,
    #[serde(default)]
    rules: Vec<Rule>,

    // readOnly fields
    urn: Option<String>,
    realm: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct Rule {
    #[serde(rename = "match")]
    match_condition: String,
    action: Action,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Action {
    SetDeviceId(serde_json::Value),
    CheckoutServices(serde_json::Value),
    Proxy(ProxyAction),
    Redirect(serde_json::Value),
    Jump(serde_json::Value),
    SetVariables(serde_json::Value),
    SetHeaders(serde_json::Value),
    Authentication(serde_json::Value),
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ProxyAction {
    target: String,
    #[serde(default)]
    no_body: bool,
}

// --- In-memory representation for hierarchical data (Nodes) ---

#[derive(Debug)]
struct ZoneNode {
    zone: Zone,
    subdomains: Vec<Subdomain>,
}

#[derive(Debug)]
struct HubNode {
    hub: Hub,
    services: Vec<Service>,
}

#[derive(Debug)]
struct RealmNode {
    realm: Realm,
    zones: Vec<ZoneNode>,
    hubs: Vec<HubNode>,
    virtual_hosts: Vec<VirtualHost>,
    routing_chains: Vec<RoutingChain>,
}


// --- Generic Fetch Function ---
async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str, client: &reqwest::Client) -> Result<T, Error> {
    client.get(url).send().await?.json::<T>().await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_base_url = "http://localhost:3000/v1";
    let client = Arc::new(reqwest::Client::new());

    // --- Phase 1: Fetching all data from the API ---
    println!("--- Phase 1: Fetching all data from the API ---\n");

    let realms: Vec<Realm> = match fetch_json(&format!("{}/realms", api_base_url), &client).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch realms: {}", e);
            return Ok(());
        }
    };
    println!("Found {} realm(s). Fetching sub-resources...", realms.len());

    if realms.is_empty() {
        return Ok(());
    }

    // For each realm, fetch its sub-resources and build an in-memory representation.
    let mut tasks = Vec::new();
    for realm in realms {
        let client_clone = Arc::clone(&client);
        let api_base_url_clone = api_base_url.to_string();
        let task = tokio::spawn(async move {
            let realm_name = realm.name.clone();
            println!("[{}] Fetching resources...", realm_name);

            // --- Fetch Zones and their Subdomains ---
            let mut zone_nodes = Vec::new();
            let zones_url = format!("{}/realms/{}/zones", api_base_url_clone, realm.name);
            if let Ok(zones) = fetch_json::<Vec<Zone>>(&zones_url, &client_clone).await {
                let mut sub_tasks = Vec::new();
                for zone in zones {
                    let client_clone2 = Arc::clone(&client_clone);
                    let api_base_url_clone2 = api_base_url_clone.to_string();
                    let realm_name = realm.name.clone();
                    sub_tasks.push(tokio::spawn(async move {
                        let mut subdomains = Vec::new();
                        let subdomains_url = format!("{}/realms/{}/zones/{}/subdomains", api_base_url_clone2, realm_name, zone.name);
                        if let Ok(fetched_subdomains) = fetch_json::<Vec<Subdomain>>(&subdomains_url, &client_clone2).await {
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
            if let Ok(hubs) = fetch_json::<Vec<Hub>>(&hubs_url, &client_clone).await {
                let mut sub_tasks = Vec::new();
                for hub in hubs {
                    let client_clone2 = Arc::clone(&client_clone);
                    let api_base_url_clone2 = api_base_url_clone.to_string();
                    let realm_name = realm.name.clone();
                    sub_tasks.push(tokio::spawn(async move {
                        let mut services = Vec::new();
                        let services_url = format!("{}/realms/{}/hubs/{}/services", api_base_url_clone2, realm_name, hub.name);
                        if let Ok(fetched_services) = fetch_json::<Vec<Service>>(&services_url, &client_clone2).await {
                            services = fetched_services;
                        }
                        HubNode { hub, services }
                    }));
                }
                hub_nodes = join_all(sub_tasks).await.into_iter().filter_map(Result::ok).collect();
            }

            // --- Fetch Virtual Hosts ---
            let vhosts_url = format!("{}/realms/{}/virtual-hosts", api_base_url_clone, realm.name);
            let virtual_hosts = fetch_json::<Vec<VirtualHost>>(&vhosts_url, &client_clone).await.unwrap_or_default();

            // --- Fetch Routing Chains ---
            let rchains_url = format!("{}/realms/{}/routing-chains", api_base_url_clone, realm.name);
            let routing_chains = fetch_json::<Vec<RoutingChain>>(&rchains_url, &client_clone).await.unwrap_or_default();

            println!("[{}] ...Done.", realm_name);

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

    println!("\n--- Phase 2: Usage Phase (Displaying hierarchical data) ---\n");

    for r_node in &realm_nodes {
        println!("Realm: {:#?}", r_node.realm);

        if !r_node.zones.is_empty() {
            println!("  - Zones:");
            for z_node in &r_node.zones {
                println!("    - Zone: {:#?}", z_node.zone);
                if !z_node.subdomains.is_empty() {
                    println!("      - Subdomains:");
                    for subdomain in &z_node.subdomains {
                        println!("        - Subdomain: {:#?}", subdomain);
                    }
                }
            }
        }

        if !r_node.hubs.is_empty() {
            println!("  - Hubs:");
            for h_node in &r_node.hubs {
                println!("    - Hub: {:#?}", h_node.hub);
                if !h_node.services.is_empty() {
                    println!("      - Services:");
                    for service in &h_node.services {
                        println!("        - Service: {:#?}", service);
                    }
                }
            }
        }
        if !r_node.virtual_hosts.is_empty() {
            println!("  - VirtualHosts:");
            for vhost in &r_node.virtual_hosts {
                println!("    - VirtualHost: {:#?}", vhost);
            }
        }

        if !r_node.routing_chains.is_empty() {
            println!("  - RoutingChains:");
            for rchain in &r_node.routing_chains {
                println!("    - RoutingChain: {:#?}", rchain);
            }
        }
        println!(); // Add a blank line for readability
    }

    // --- Phase 3: Usage Phase (Find a matching action from a routing-chain) ---
    println!("\n--- Phase 3: Usage Phase (Find a matching action) ---\n");

    // Sample input: FQDN and path
    let sample_fqdn = "sirius.sr.example.com";
    let sample_path = "/api/v1/products";

    println!("Searching for an action for FQDN: '{}' and Path: '{}'...", sample_fqdn, sample_path);

    match find_matching_action(&realm_nodes, sample_fqdn, sample_path) {
        Some(action) => {
            println!("\nFound a matching action:");
            println!("{:#?}", action);
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
            if let Some(rchain) = r_node.routing_chains.iter().find(|rc| rc.name == vhost.routing_chain) {
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
    // If no match is found, return None.
    None
}

/// A simple helper to evaluate if a rule's match condition is met.
/// This is a basic implementation and does not support all complex syntaxes.
fn rule_matches(match_condition: &str, fqdn: &str, path: &str) -> bool {
    // Split conditions by '&&'
    for condition in match_condition.split("&&") {
        let trimmed_condition = condition.trim();
        if !evaluate_single_condition(trimmed_condition, fqdn, path) {
            // If any condition is false, the whole rule does not match.
            return false;
        }
    }
    // If all conditions are true, the rule matches.
    true
}

/// Evaluates a single condition like `path_prefix(...)` or `host(...)`.
fn evaluate_single_condition(condition: &str, fqdn: &str, path: &str) -> bool {
    if let Some(arg) = extract_arg(condition, "path_prefix") {
        return path.starts_with(&arg);
    }
    if let Some(arg) = extract_arg(condition, "path") {
        return path == arg;
    }
    if let Some(arg) = extract_arg(condition, "host") {
        return fqdn == arg;
    }
    if let Some(arg) = extract_arg(condition, "method") {
        // For this example, we are not checking the HTTP method, so we'll just log it.
        // In a real scenario, you would pass the method to this function.
        println!("[NOTE] 'method' condition with arg '{}' is not evaluated in this example.", arg);
        return true; // Assume it matches for the sake of the example.
    }

    // If the condition is unknown, assume it doesn't match.
    false
}

/// Extracts the argument from a function-like string, e.g., `path_prefix(`/api`)` -> `/api`.
fn extract_arg<'a>(condition: &'a str, func_name: &str) -> Option<String> {
    if condition.starts_with(func_name) && condition.contains('(') && condition.ends_with(')') {
        let start = condition.find('(')? + 1;
        let end = condition.rfind(')')?;
        if start >= end {
            return None;
        }
        // Extract the argument and trim quotes (`, ', ")
        let arg = condition[start..end].trim();
        let trimmed_arg = arg.trim_matches(|c| c == '`' || c == '\'' || c == '"');
        return Some(trimmed_arg.to_string());
    }
    None
}