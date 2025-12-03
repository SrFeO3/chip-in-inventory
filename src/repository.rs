use crate::ApiError;
use crate::models::{Hub, Realm, RoutingChain, Service, Subdomain, VirtualHost, Zone};
use etcd_client::{Client, GetOptions, SortOrder, SortTarget};

const REALM_KEY_PREFIX: &str = "realms/";

#[derive(Clone)]
pub struct EtcdRepository {
    client: Client,
}

impl EtcdRepository {
    pub async fn new(endpoints: &[&str]) -> Result<Self, etcd_client::Error> {
        let client = Client::connect(endpoints, None).await?;
        Ok(Self { client })
    }

    // --- Realm Methods ---

    fn realm_key(name: &str) -> String {
        format!("{}{}", REALM_KEY_PREFIX, name)
    }

    pub async fn save_realm(&self, realm: &Realm) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::realm_key(&realm.name);
        let value = serde_json::to_string(realm)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_realm(&self, id: &str) -> Result<Realm, ApiError> {
        let mut client = self.client.clone();
        let key = Self::realm_key(id); // Path parameter is still id
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let realm = serde_json::from_slice(kv.value())?;
            Ok(realm)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_realms(&self) -> Result<Vec<Realm>, ApiError> {
        let mut client = self.client.clone();
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(REALM_KEY_PREFIX, Some(options)).await?;
        let realms = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                // Ensure that the remaining key part after removing REALM_KEY_PREFIX does not contain '/'.
                // This targets only `realms/{id}` and excludes keys like `realms/{id}/zones/{id}`.
                if !key_str[REALM_KEY_PREFIX.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(realms)
    }

    pub async fn delete_realm(&self, id: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::realm_key(id); // Path parameter is still id
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- Zone Methods ---

    fn zone_key(realm_id: &str, zone_id: &str) -> String {
        format!("{}{}/zones/{}", REALM_KEY_PREFIX, realm_id, zone_id)
    }

    fn zones_prefix(realm_id: &str) -> String {
        format!("{}{}/zones/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_zone(&self, realm_id: &str, zone: &Zone) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::zone_key(realm_id, &zone.name);
        let value = serde_json::to_string(zone)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_zone(&self, realm_id: &str, zone_id: &str) -> Result<Zone, ApiError> {
        let mut client = self.client.clone();
        let key = Self::zone_key(realm_id, zone_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let zone = serde_json::from_slice(kv.value())?;
            Ok(zone)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_zones(&self, realm_id: &str) -> Result<Vec<Zone>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::zones_prefix(realm_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let zones = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(zones)
    }

    pub async fn delete_zone(&self, realm_id: &str, zone_id: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::zone_key(realm_id, zone_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- Subdomain Methods ---

    fn subdomain_key(realm_id: &str, zone_id: &str, subdomain_id: &str) -> String {
        format!(
            "{}{}/zones/{}/subdomains/{}",
            REALM_KEY_PREFIX, realm_id, zone_id, subdomain_id
        )
    }

    fn subdomains_prefix(realm_id: &str, zone_id: &str) -> String {
        format!(
            "{}{}/zones/{}/subdomains/",
            REALM_KEY_PREFIX, realm_id, zone_id
        )
    }

    pub async fn save_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain: &Subdomain,
    ) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::subdomain_key(realm_id, zone_id, &subdomain.name); // No change needed here
        let value = serde_json::to_string(subdomain)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<Subdomain, ApiError> {
        let mut client = self.client.clone();
        let key = Self::subdomain_key(realm_id, zone_id, subdomain_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let subdomain = serde_json::from_slice(kv.value())?;
            Ok(subdomain)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_subdomains(
        &self,
        realm_id: &str,
        zone_id: &str,
    ) -> Result<Vec<Subdomain>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::subdomains_prefix(realm_id, zone_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let subdomains = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(subdomains)
    }

    pub async fn delete_subdomain(
        &self,
        realm_id: &str,
        zone_id: &str,
        subdomain_id: &str,
    ) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::subdomain_key(realm_id, zone_id, subdomain_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- VirtualHost Methods ---

    fn virtual_host_key(realm_id: &str, virtual_host_id: &str) -> String {
        format!(
            "{}{}/virtual-hosts/{}",
            REALM_KEY_PREFIX, realm_id, virtual_host_id
        )
    }

    fn virtual_hosts_prefix(realm_id: &str) -> String {
        format!("{}{}/virtual-hosts/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_virtual_host(
        &self,
        realm_id: &str,
        virtual_host: &VirtualHost,
    ) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::virtual_host_key(realm_id, &virtual_host.name); // No change needed here
        let value = serde_json::to_string(virtual_host)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<VirtualHost, ApiError> {
        let mut client = self.client.clone();
        let key = Self::virtual_host_key(realm_id, virtual_host_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let vhost = serde_json::from_slice(kv.value())?;
            Ok(vhost)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_virtual_hosts(&self, realm_id: &str) -> Result<Vec<VirtualHost>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::virtual_hosts_prefix(realm_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let vhosts = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(vhosts)
    }

    pub async fn delete_virtual_host(
        &self,
        realm_id: &str,
        virtual_host_id: &str,
    ) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::virtual_host_key(realm_id, virtual_host_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- RoutingChain Methods ---

    fn routing_chain_key(realm_id: &str, routing_chain_id: &str) -> String {
        format!(
            "{}{}/routing-chains/{}",
            REALM_KEY_PREFIX, realm_id, routing_chain_id
        )
    }

    fn routing_chains_prefix(realm_id: &str) -> String {
        format!("{}{}/routing-chains/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_routing_chain(
        &self,
        realm_id: &str,
        routing_chain: &RoutingChain,
    ) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::routing_chain_key(realm_id, &routing_chain.name);
        let value = serde_json::to_string(routing_chain)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_routing_chain(
        &self,
        realm_id: &str,
        routing_chain_id: &str,
    ) -> Result<RoutingChain, ApiError> {
        let mut client = self.client.clone();
        let key = Self::routing_chain_key(realm_id, routing_chain_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let rchain = serde_json::from_slice(kv.value())?;
            Ok(rchain)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_routing_chains(&self, realm_id: &str) -> Result<Vec<RoutingChain>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::routing_chains_prefix(realm_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let rchains = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(rchains)
    }

    pub async fn delete_routing_chain(
        &self,
        realm_id: &str,
        routing_chain_id: &str,
    ) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::routing_chain_key(realm_id, routing_chain_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- Hub Methods ---

    fn hub_key(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    fn hubs_prefix(realm_id: &str) -> String {
        format!("{}{}/hubs/", REALM_KEY_PREFIX, realm_id)
    }

    pub async fn save_hub(&self, realm_id: &str, hub: &Hub) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::hub_key(realm_id, &hub.name);
        let value = serde_json::to_string(hub)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_hub(&self, realm_id: &str, hub_id: &str) -> Result<Hub, ApiError> {
        let mut client = self.client.clone();
        let key = Self::hub_key(realm_id, hub_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let hub = serde_json::from_slice(kv.value())?;
            Ok(hub)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_hubs(&self, realm_id: &str) -> Result<Vec<Hub>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::hubs_prefix(realm_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let hubs = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(hubs)
    }

    pub async fn delete_hub(&self, realm_id: &str, hub_id: &str) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::hub_key(realm_id, hub_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }

    // --- Service Methods ---

    fn service_key(realm_id: &str, hub_id: &str, service_id: &str) -> String {
        format!(
            "{}{}/hubs/{}/services/{}",
            REALM_KEY_PREFIX, realm_id, hub_id, service_id
        )
    }

    fn services_prefix(realm_id: &str, hub_id: &str) -> String {
        format!("{}{}/hubs/{}/services/", REALM_KEY_PREFIX, realm_id, hub_id)
    }

    pub async fn save_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service: &Service,
    ) -> Result<(), ApiError> {
        let mut client = self.client.clone();
        let key = Self::service_key(realm_id, hub_id, &service.name);
        let value = serde_json::to_string(service)?;
        client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<Service, ApiError> {
        let mut client = self.client.clone();
        let key = Self::service_key(realm_id, hub_id, service_id);
        let resp = client.get(key, None).await?;
        if let Some(kv) = resp.kvs().first() {
            let service = serde_json::from_slice(kv.value())?;
            Ok(service)
        } else {
            Err(ApiError::NotFound)
        }
    }

    pub async fn list_services(&self, realm_id: &str, hub_id: &str) -> Result<Vec<Service>, ApiError> {
        let mut client = self.client.clone();
        let prefix = Self::services_prefix(realm_id, hub_id);
        let options = GetOptions::new()
            .with_prefix()
            .with_sort(SortTarget::Key, SortOrder::Ascend);
        let resp = client.get(prefix.as_str(), Some(options)).await?;
        let services = resp
            .kvs()
            .iter()
            .filter_map(|kv| {
                let key_str = kv.key_str().ok()?;
                if !key_str[prefix.len()..].contains('/') {
                    serde_json::from_slice(kv.value()).ok()
                } else {
                    None
                }
            })
            .collect();
        Ok(services)
    }

    pub async fn delete_service(
        &self,
        realm_id: &str,
        hub_id: &str,
        service_id: &str,
    ) -> Result<bool, ApiError> {
        let mut client = self.client.clone();
        let key = Self::service_key(realm_id, hub_id, service_id);
        let resp = client.delete(key, None).await?;
        Ok(resp.deleted() > 0)
    }
}
