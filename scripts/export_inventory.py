import requests
import yaml
import sys

# --- Configuration ---
# Please change the base URL of the API server to match your environment.
API_BASE_URL = "http://localhost:3000/v1"
# --- End of configuration ---

def api_get(path):
    """Helper function to send a GET request to the API and handle errors"""
    url = f"{API_BASE_URL}{path}"
    try:
        response = requests.get(url)
        response.raise_for_status()  # Raise an exception for non-2xx status codes
        if response.status_code == 204 or not response.content:
            return [] # Return empty list for No Content or empty body
        return response.json()
    except requests.exceptions.HTTPError as e:
        print(f"ERROR: GET {path} failed with status {e.response.status_code}", file=sys.stderr)
        print(f"       Response: {e.response.text}", file=sys.stderr)
        raise
    except requests.exceptions.RequestException as e:
        print(f"FATAL: Could not connect to API at {url}. Error: {e}", file=sys.stderr)
        sys.exit(1)

def clean_item_for_export(item, item_type):
    """Remove read-only and server-generated fields from an item."""
    cleaned = item.copy()
    
    # Fields to remove from all items
    for key in ['urn', 'zone', 'hub']:
        if key in cleaned:
            del cleaned[key]

    # The 'realm' field is editable for Service, but read-only for others.
    if item_type != 'Service' and 'realm' in cleaned:
        del cleaned['realm']

    # FQDN is read-only for VirtualHost and Subdomain, but editable for Hub.
    if item_type in ['VirtualHost', 'Subdomain'] and 'fqdn' in cleaned:
        del cleaned['fqdn']

    # For Realm, 'disabled' is a boolean, not Option<bool>. Ensure it exists.
    if item_type == 'Realm':
        if cleaned.get('disabled') is None:
            cleaned['disabled'] = False

    return cleaned

async def fetch_all_data_for_export():
    """Fetch all inventory data from the API."""
    print("Fetching data from API...", file=sys.stderr)
    realms = api_get('/realms')
    export_data = {"realms": []}

    for realm in realms:
        realm_name = realm['name']
        realm_path = f"/realms/{realm_name}"
        print(f"  - Processing Realm: {realm_name}", file=sys.stderr)
        cleaned_realm = clean_item_for_export(realm, 'Realm')

        # Fetch Zones and their Subdomains
        zones = api_get(f"{realm_path}/zones")
        cleaned_realm['zones'] = []
        for zone in zones:
            zone_name = zone['name']
            zone_path = f"{realm_path}/zones/{zone_name}"
            cleaned_zone = clean_item_for_export(zone, 'Zone')
            
            subdomains = api_get(f"{zone_path}/subdomains")
            cleaned_zone['subdomains'] = [clean_item_for_export(sub, 'Subdomain') for sub in subdomains]
            cleaned_realm['zones'].append(cleaned_zone)

        # Fetch Hubs and their Services
        hubs = api_get(f"{realm_path}/hubs")
        cleaned_realm['hubs'] = []
        for hub in hubs:
            hub_name = hub['name']
            hub_path = f"{realm_path}/hubs/{hub_name}"
            cleaned_hub = clean_item_for_export(hub, 'Hub')

            services = api_get(f"{hub_path}/services")
            cleaned_hub['services'] = [clean_item_for_export(svc, 'Service') for svc in services]
            cleaned_realm['hubs'].append(cleaned_hub)

        # Fetch Virtual Hosts
        virtual_hosts = api_get(f"{realm_path}/virtual-hosts")
        cleaned_realm['virtualHosts'] = [clean_item_for_export(vh, 'VirtualHost') for vh in virtual_hosts]

        # Fetch Routing Chains
        routing_chains = api_get(f"{realm_path}/routing-chains")
        cleaned_realm['routingChains'] = [clean_item_for_export(rc, 'RoutingChain') for rc in routing_chains]

        export_data["realms"].append(cleaned_realm)

    print("Data fetching complete.", file=sys.stderr)
    return export_data

def main():
    """Main processing"""
    try:
        # The original script.js uses an async function, so we define one here too for consistency.
        # In this simple script, 'await' is not strictly necessary, but we'll use a helper.
        import asyncio
        data = asyncio.run(fetch_all_data_for_export())

        # Convert the data to YAML format
        # sort_keys=False preserves the order from the API as much as possible
        yaml_output = yaml.dump(data, indent=2, sort_keys=False, allow_unicode=True)

        # Print the YAML to standard output
        print(yaml_output)

    except Exception as e:
        print(f"\nAn error occurred during export: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    # This script does not take any command line arguments.
    if len(sys.argv) > 1:
        print(f"Usage: python {sys.argv[0]}", file=sys.stderr)
        print("This script exports the entire inventory to YAML format on standard output.", file=sys.stderr)
        print("Redirect the output to a file, e.g., `python export_inventory.py > inventory.yaml`", file=sys.stderr)
        sys.exit(1)
    
    main()
