import yaml
import requests
import sys
import json
import os

# --- Configuration ---
# Please change the base URL of the API server to match your environment.
# The API_BASE_URL can be overridden by an environment variable.
API_BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:3000/v1")
# --- End of configuration ---

def send_request(method, path, data=None):
    """Helper function to send a request to the API and handle errors"""
    url = f"{API_BASE_URL}{path}"
    headers = {"Content-Type": "application/json"}
    
    try:
        response = requests.request(method, url, data=json.dumps(data) if data else None, headers=headers)
        response.raise_for_status()  # Raise an exception for non-2xx status codes
        print(f"SUCCESS: {method} {path}")
        return response
    except requests.exceptions.HTTPError as e:
        # Treat 404 Not Found as a normal case for later POST
        if e.response.status_code == 404:
            return e.response
        
        # Other HTTP errors
        print(f"ERROR: {method} {path} failed with status {e.response.status_code}")
        print(f"       Response: {e.response.text}")
        raise
    except requests.exceptions.RequestException as e:
        # Connection errors, etc.
        print(f"FATAL: Could not connect to API at {url}. Error: {e}")
        sys.exit(1)

def import_resource(resource_type, path, resource_data):
    """
    Import a resource (try to update, if not found, create).
    """
    # Keys for nested resources that are handled by separate API calls.
    # These should be removed from both POST and PUT payloads.
    nested_keys = ['zones', 'subdomains', 'hubs', 'services', 'virtualHosts', 'routingChains']
    
    # Keys that are read-only or path parameters and should be excluded from the PUT body.
    # 'name' is also excluded as it's in the path.

    # --- Prepare payload for POST (create) ---
    # The POST payload should not contain nested resource lists.
    post_payload = resource_data.copy()
    for key in nested_keys:
        if key in post_payload:
            del post_payload[key]

    # --- Prepare payload for PUT (update) ---
    update_payload = post_payload.copy()
    
    # Base keys to always remove from PUT body (they are in the URL path or server-managed)
    keys_to_remove = ['name', 'urn', 'createdAt', 'updatedAt']
    
    # FQDN is read-only for VirtualHost and Subdomain, but editable for Hub.
    if resource_type in ['VirtualHost', 'Subdomain']:
        keys_to_remove.append('fqdn')

    for key in keys_to_remove:
        if key in update_payload:
            del update_payload[key]

    # 1. First, try to update with PUT
    response = send_request('PUT', path, data=update_payload)

    # 2. If PUT fails with 404, create a new one with POST
    if response.status_code == 404:
        resource_name = resource_data.get('name', path.split('/')[-1])
        print(f"INFO: '{resource_name}' not found, creating new {resource_type}...")
        # Use the prepared POST payload for creation
        send_request('POST', path.rsplit('/', 1)[0], data=post_payload)

def main(filepath):
    """Main processing"""
    try:
        with open(filepath, 'r') as f:
            data = yaml.safe_load(f)
    except FileNotFoundError:
        print(f"FATAL: YAML file not found at '{filepath}'")
        sys.exit(1)
    except yaml.YAMLError as e:
        print(f"FATAL: Error parsing YAML file: {e}")
        sys.exit(1)

    if 'realms' not in data or not data['realms']:
        print("No realms found in the YAML file.")
        return

    # Import resources hierarchically
    for realm in data['realms']:
        realm_name = realm['name']
        realm_path = f"/realms/{realm_name}"
        # For the top-level realm, we pass the whole realm object.
        # import_resource will clean it.
        import_resource('Realm', realm_path, realm)

        # Zones and Subdomains
        if 'zones' in realm and realm['zones']:
            for zone in realm['zones']:
                zone_name = zone['name']
                zone_path = f"{realm_path}/zones/{zone_name}"
                import_resource('Zone', zone_path, zone)
                
                if 'subdomains' in zone and zone['subdomains']:
                    for subdomain in zone['subdomains']:
                        subdomain_name = subdomain['name']
                        subdomain_path = f"{zone_path}/subdomains/{subdomain_name}"
                        import_resource('Subdomain', subdomain_path, subdomain)

        # Hubs and Services
        if 'hubs' in realm and realm['hubs']:
            for hub in realm['hubs']:
                hub_name = hub['name']
                hub_path = f"{realm_path}/hubs/{hub_name}"
                import_resource('Hub', hub_path, hub)

                if 'services' in hub and hub['services']:
                    for service in hub['services']:
                        service_name = service['name']
                        service_path = f"{hub_path}/services/{service_name}"
                        import_resource('Service', service_path, service)

        # Virtual Hosts
        if 'virtualHosts' in realm and realm['virtualHosts']:
            for vhost in realm['virtualHosts']:
                vhost_name = vhost['name']
                vhost_path = f"{realm_path}/virtual-hosts/{vhost_name}"
                import_resource('VirtualHost', vhost_path, vhost)
        
        # Routing Chains
        if 'routingChains' in realm and realm['routingChains']:
            for rchain in realm['routingChains']:
                rchain_name = rchain['name']
                rchain_path = f"{realm_path}/routing-chains/{rchain_name}"
                import_resource('RoutingChain', rchain_path, rchain)

    print("\nImport process finished.")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: python {sys.argv[0]} <path_to_yaml_file>")
        sys.exit(1)
    
    yaml_filepath = sys.argv[1]
    main(yaml_filepath)
