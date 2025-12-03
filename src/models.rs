use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Structure for error responses
#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

/// Structure representing Realm information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Realm {
    pub name: String, // Realm name
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    pub cacert: String,
    #[serde(rename = "signingKey")]
    pub signing_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub administrators: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
    pub disabled: bool,
    pub updated_at: DateTime<Utc>,
}

/// Structure used as a request body when creating a new Realm
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRealm {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub cacert: String,
    #[serde(rename = "signingKey")]
    pub signing_key: String,
    pub session_timeout: Option<i64>,
    pub administrators: Option<Vec<String>>,
    pub expired_at: Option<String>,
    pub disabled: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Realm
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRealm {
    pub title: String,
    pub description: Option<String>,
    pub cacert: String,
    #[serde(rename = "signingKey")]
    pub signing_key: String,
    pub session_timeout: Option<i64>,
    pub administrators: Option<Vec<String>>,
    pub expired_at: Option<String>,
    pub disabled: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Zone information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub name: String, // Zone name
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acme_certificate_provider: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Structure used as a request body when creating a new Zone
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewZone {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub dns_provider: Option<String>,
    pub acme_certificate_provider: Option<String>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Zone
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateZone {
    pub title: String,
    pub description: Option<String>,
    pub dns_provider: Option<String>,
    pub acme_certificate_provider: Option<String>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Subdomain information for storage and full response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subdomain {
    pub name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_realm: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub share_cookie: bool,
    // Read-only fields, populated on retrieval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Structure for creating a new Subdomain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSubdomain {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub realm: Option<String>,
    pub destination_realm: Option<String>,
    #[serde(default)]
    pub share_cookie: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure for updating a Subdomain. Note: `name` is immutable.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubdomain {
    pub title: String,
    pub description: Option<String>,
    pub realm: Option<String>,
    pub destination_realm: Option<String>,
    #[serde(default)]
    pub share_cookie: bool,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing VirtualHost information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHost {
    pub name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    pub subdomain: String,
    pub routing_chain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_recorder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_max_value_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Structure representing VirtualHost information for API responses, including derived fields.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHostResponse {
    pub name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    pub subdomain: String,
    pub routing_chain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_recorder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_max_value_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_log_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Structure used as a request body when creating a VirtualHost
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVirtualHost {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub subdomain: String,
    pub routing_chain: String,
    pub access_log_recorder: Option<String>,
    pub access_log_max_value_length: Option<i32>,
    pub access_log_format: Option<serde_json::Value>,
    pub certificate: Option<Vec<String>>,
    pub key: Option<String>,
    pub disabled: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a VirtualHost
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualHost {
    pub title: String,
    pub description: Option<String>,
    pub subdomain: String,
    pub routing_chain: String,
    pub access_log_recorder: Option<String>,
    pub access_log_max_value_length: Option<i32>,
    pub access_log_format: Option<serde_json::Value>,
    pub certificate: Option<Vec<String>>,
    pub key: Option<String>,
    pub disabled: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

// --- RoutingChain related structures ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetDeviceId {
    pub expiration: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutServices {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub target: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_body: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Redirect {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Jump {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetVariables {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetHeaders {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Authentication {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    SetDeviceId(SetDeviceId),
    CheckoutServices(CheckoutServices),
    Proxy(Proxy),
    Redirect(Redirect),
    Jump(Jump),
    SetVariables(SetVariables),
    SetHeaders(SetHeaders),
    Authentication(Authentication),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rule {
    #[serde(rename = "match")]
    pub match_condition: String,
    pub action: Action,
}

/// Structure representing RoutingChain information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingChain {
    pub name: String, // RoutingChain name
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Structure used for creating and updating a RoutingChain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoutingChain {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used for updating a RoutingChain
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoutingChain {
    pub title: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Hub information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hub {
    pub name: String, // Hub name
    pub title: String,
    pub fqdn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>, // readOnly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urn: Option<String>, // readOnly
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Structure used as a request body when creating a new Hub
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHub {
    pub name: String,
    pub title: String,
    pub fqdn: String,
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Hub
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHub {
    pub title: String,
    pub fqdn: String,
    pub server_port: Option<u16>,
    pub server_cert: String,
    pub server_cert_key: String,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure representing Service information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String, // Service name
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub realm: String,
    pub provider: Vec<String>,
    pub consumers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_management: Option<AvailabilityManagement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub singleton: Option<bool>,
    // Read-only fields
    pub hub: String,
    pub urn: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

/// Structure representing AvailabilityManagement information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvailabilityManagement {
    pub cluster_manager_urn: String,
    pub service_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ondemand_start: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<EnvVar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_points: Option<Vec<MountPoint>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MountPoint {
    pub volume_size: i32,
    pub target: String,
}

/// Structure used as a request body when creating a new Service
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewService {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub provider: Vec<String>,
    pub consumers: Vec<String>,
    pub availability_management: Option<AvailabilityManagement>,
    pub singleton: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structure used as a request body when updating a Service
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateService {
    pub title: String,
    pub description: Option<String>,
    pub provider: Vec<String>,
    pub consumers: Vec<String>,
    pub availability_management: Option<AvailabilityManagement>,
    pub singleton: Option<bool>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}
