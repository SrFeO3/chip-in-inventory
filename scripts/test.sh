#!/bin/bash
set -u

BASE_URL="http://127.0.0.1:3000/v1"

# Counters and colors
SUCCESS_COUNT=0
FAIL_COUNT=0
TOTAL_COUNT=0
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Helper function to run a test and check the result
# $1: Test description
# $2: Expected HTTP status code
# $3...: curl command arguments
run_test() {
    local description="$1"
    local expected_status="$2"

    echo -n "--- $description ---"
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    shift 2

    # Capture response body and HTTP status code
    response_file=$(mktemp)
    http_status=$(curl -s -o "$response_file" -w "%{http_code}" "$@")

    if [ "$http_status" -eq "$expected_status" ]; then
        printf " %b[SUCCESS]%b (Status: %s)\n" "${GREEN}" "${NC}" "$http_status"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        # If the body is not empty, format and display it with jq
        if [ -s "$response_file" ]; then
            if command -v jq &> /dev/null; then
                jq . "$response_file"
            else
                cat "$response_file"
            fi
        fi
    else
        printf " %b[FAIL]%b (Expected: %s, Got: %s)\n" "${RED}" "${NC}" "$expected_status" "$http_status"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        # Also display the body on failure
        if [ -s "$response_file" ]; then
            cat "$response_file"
        fi
    fi
    rm -f "$response_file"
    echo ""
}

echo "##############################"
echo "### Testing Realms API ###"
echo "##############################"

REALM_ID="my-test-realm"

run_test "Creating a new realm" 201 \
  -X POST "${BASE_URL}/realms" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${REALM_ID}"'",
    "title": "Test Realm",
    "description": "A realm for testing",
    "cacert": "-----BEGIN CERTIFICATE-----\\n...\\n-----END CERTIFICATE-----",
    "deviceIdSigningKey": "-----BEGIN PRIVATE KEY-----\\n...\\n-----END PRIVATE KEY-----",
    "deviceIdVerificationKey": "-----BEGIN PUBLIC KEY-----\\n...\\n-----END PUBLIC KEY-----",
    "disabled": false
  }'

run_test "Listing all realms" 200 \
  "${BASE_URL}/realms"

echo "##############################"
echo "### Testing Zones API ###"
echo "##############################"

ZONE_ID="my-test-zone"

run_test "Creating a new zone in realm '${REALM_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/zones" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${ZONE_ID}"'",
    "title": "Test Zone",
    "description": "A zone for testing"
  }'

run_test "Listing all zones in realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/zones"

run_test "Getting the zone '${ZONE_ID}' from realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}"

echo "##############################"
echo "### Testing Subdomains API ###"
echo "##############################"

SUBDOMAIN_ID="my-test-subdomain"

run_test "Creating a new subdomain in zone '${ZONE_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}/subdomains" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${SUBDOMAIN_ID}"'",
    "title": "Test Subdomain",
    "description": "A subdomain for testing"
  }'

run_test "Listing all subdomains in zone '${ZONE_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}/subdomains"

run_test "Getting the subdomain '${SUBDOMAIN_ID}' from zone '${ZONE_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}/subdomains/${SUBDOMAIN_ID}"

run_test "Updating the subdomain '${SUBDOMAIN_ID}'" 200 \
  -X PUT "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}/subdomains/${SUBDOMAIN_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test Subdomain",
    "description": "An updated subdomain description",
    "shareCookie": false
  }'

run_test "Deleting the subdomain '${SUBDOMAIN_ID}'" 204 \
  -X DELETE "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}/subdomains/${SUBDOMAIN_ID}"

echo "##############################"
echo "### Testing VirtualHosts API ###"
echo "##############################"

VHOST_ID="www"
SUBDOMAIN_URN="urn:chip-in:subdomain:${REALM_ID}:${ZONE_ID}:@"
RCHAIN_URN="urn:chip-in:routing-chain:${REALM_ID}:my-test-rchain"

run_test "Creating a new virtual host in realm '${REALM_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/virtual-hosts" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${VHOST_ID}"'", "title": "Test VHost", "description": "A vhost for testing",
    "subdomain": "'"${SUBDOMAIN_URN}"'", "routingChain": "'"${RCHAIN_URN}"'"
  }'

run_test "Listing all virtual hosts in realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/virtual-hosts"

run_test "Getting the virtual host '${VHOST_ID}' from realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/virtual-hosts/${VHOST_ID}"

run_test "Updating the virtual host '${VHOST_ID}'" 200 \
  -X PUT "${BASE_URL}/realms/${REALM_ID}/virtual-hosts/${VHOST_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test VHost", "description": "An updated vhost description",
    "subdomain": "'"${SUBDOMAIN_URN}"'", "routingChain": "'"${RCHAIN_URN}"'"
  }'

run_test "Deleting the virtual host '${VHOST_ID}'" 204 \
  -X DELETE "${BASE_URL}/realms/${REALM_ID}/virtual-hosts/${VHOST_ID}"

echo "##############################"
echo "### Testing RoutingChains API ###"
echo "##############################"

RCHAIN_ID="my-test-rchain"

run_test "Creating a new routing chain in realm '${REALM_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/routing-chains" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${RCHAIN_ID}"'",
    "title": "Test Routing Chain",
    "description": "A routing chain for testing"
  }'

run_test "Listing all routing chains in realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/routing-chains"

run_test "Getting the routing chain '${RCHAIN_ID}' from realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/routing-chains/${RCHAIN_ID}"

run_test "Updating the routing chain '${RCHAIN_ID}'" 200 \
  -X PUT "${BASE_URL}/realms/${REALM_ID}/routing-chains/${RCHAIN_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test Routing Chain",
    "description": "An updated routing chain description",
    "rules": []
  }'

run_test "Deleting the routing chain '${RCHAIN_ID}'" 204 \
  -X DELETE "${BASE_URL}/realms/${REALM_ID}/routing-chains/${RCHAIN_ID}"

echo "##############################"
echo "### Testing Hubs API ###"
echo "##############################"

HUB_ID="my-test-hub"

run_test "Creating a new hub in realm '${REALM_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/hubs" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${HUB_ID}"'",
    "title": "Test Hub",
    "description": "A hub for testing",
    "fqdn": "hub.example.com",
    "serverCert": "-----BEGIN CERTIFICATE-----\\n...\\n-----END CERTIFICATE-----",
    "serverCertKey": "-----BEGIN RSA PRIVATE KEY-----\\n...\\n-----END RSA PRIVATE KEY-----"
  }'

run_test "Listing all hubs in realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/hubs"

run_test "Getting the hub '${HUB_ID}' from realm '${REALM_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}"

run_test "Updating the hub '${HUB_ID}'" 200 \
  -X PUT "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test Hub",
    "description": "An updated hub description",
    "fqdn": "hub.example.com",
    "serverCert": "-----BEGIN CERTIFICATE-----\\n...\\n-----END CERTIFICATE-----",
    "serverCertKey": "-----BEGIN RSA PRIVATE KEY-----\\n...\\n-----END RSA PRIVATE KEY-----"
  }'

echo "##############################"
echo "### Testing Services API ###"
echo "##############################"

SERVICE_ID="my-test-service"

run_test "Creating a new service in hub '${HUB_ID}'" 201 \
  -X POST "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}/services" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "'"${SERVICE_ID}"'",
    "title": "Test Service",
    "description": "A service for testing",
    "realm": "'"${REALM_ID}"'",
    "provider": [], "consumers": []
  }'

run_test "Listing all services in hub '${HUB_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}/services"

run_test "Getting the service '${SERVICE_ID}' from hub '${HUB_ID}'" 200 \
  "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}/services/${SERVICE_ID}"

run_test "Updating the service '${SERVICE_ID}'" 200 \
  -X PUT "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}/services/${SERVICE_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test Service",
    "description": "An updated service description",
    "realm": "'"${REALM_ID}"'",
    "provider": [],
    "consumers": []
  }'

run_test "Deleting the service '${SERVICE_ID}'" 204 \
  -X DELETE "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}/services/${SERVICE_ID}"

echo "##############################"
echo "### Cleaning up resources ###"
echo "##############################"

run_test "Deleting the hub '${HUB_ID}'" 204 -X DELETE "${BASE_URL}/realms/${REALM_ID}/hubs/${HUB_ID}"
run_test "Deleting the zone '${ZONE_ID}'" 204 -X DELETE "${BASE_URL}/realms/${REALM_ID}/zones/${ZONE_ID}"
run_test "Deleting the realm '${REALM_ID}'" 204 -X DELETE "${BASE_URL}/realms/${REALM_ID}"

echo "##############################"
echo "### Test Summary ###"
echo "##############################"
printf "Total: %s, %bSuccess: %s%b, %bFail: %s%b\n" "$TOTAL_COUNT" "${GREEN}" "$SUCCESS_COUNT" "${NC}" "${RED}" "$FAIL_COUNT" "${NC}"

if [ "$FAIL_COUNT" -gt 0 ]; then
    printf "\n%bSome tests failed.%b\n" "${RED}" "${NC}"
    exit 1
else
    printf "\n%bAll tests passed successfully!%b\n" "${GREEN}" "${NC}"
    exit 0
fi
