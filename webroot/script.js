document.addEventListener('DOMContentLoaded', () => {
    const API_BASE_URL = '/v1';
    const realmsList = document.getElementById('realms-list');
    const addRealmBtn = document.getElementById('add-realm-btn');
    const detailsContainer = document.getElementById('details-container');
    let selectedItemHeader = null; // To keep track of the selected item

    // The button is initially hidden via inline style, make it visible.
    addRealmBtn.style.display = 'inline-block';

    // --- Create a proper header for the application ---
    const appHeader = document.createElement('header');
    appHeader.className = 'app-header';

    const mainTitle = document.createElement('h1');
    mainTitle.textContent = 'Inventory Management';
    appHeader.appendChild(mainTitle);

    // Insert the new header at the top of the body
    document.body.insertBefore(appHeader, document.body.firstChild);

    // --- Create a main container for side-by-side layout ---
    const mainContainer = document.createElement('div');
    mainContainer.id = 'main-layout-container';

    // --- Create a dedicated container for the list pane ---
    const listPaneContainer = document.createElement('div');
    listPaneContainer.id = 'list-pane-container';

    const listHeader = document.createElement('div');
    listHeader.className = 'pane-header';
    const listTitle = document.createElement('h2');
    listTitle.textContent = 'Objects';
    listHeader.appendChild(listTitle);
    listHeader.appendChild(addRealmBtn); // Move button next to the list title

    listPaneContainer.appendChild(listHeader);
    listPaneContainer.appendChild(realmsList);

    // Set initial width for the list pane to be 50% of the available space.
    listPaneContainer.style.flex = '1 1 50%';

    // --- Add the resizer (splitter) between the panes ---
    const resizer = document.createElement('div');
    resizer.id = 'resizer';

    mainContainer.appendChild(listPaneContainer);
    mainContainer.appendChild(resizer);
    mainContainer.appendChild(detailsContainer);

    // Append the fully constructed main container to the body
    document.body.appendChild(mainContainer);

    // --- Placeholder for Details View ---
    const showDetailsPlaceholder = () => {
        detailsContainer.innerHTML = `
            <h2>Details</h2>
            <p class="placeholder-text">Select an item from the list to see its details.</p>
        `;
    };

    // --- Resizer functionality ---
    const resizePanes = (e) => {
        // Calculate the new flex-basis for the list pane
        const containerRect = mainContainer.getBoundingClientRect();
        const newLeftWidth = e.clientX - containerRect.left;

        // Set constraints for min/max width
        const minWidth = 200; // Minimum width in pixels
        const maxWidth = containerRect.width - minWidth;

        if (newLeftWidth > minWidth && newLeftWidth < maxWidth) {
            listPaneContainer.style.flex = `0 0 ${newLeftWidth}px`;
        }
    };

    resizer.addEventListener('mousedown', (e) => {
        e.preventDefault(); // Prevent text selection

        // Add a class to the body to prevent text selection during resize
        document.body.classList.add('resizing');

        // Attach listeners to the document to handle mouse move and up events
        document.addEventListener('mousemove', resizePanes);
        document.addEventListener('mouseup', () => {
            // Cleanup listeners
            document.removeEventListener('mousemove', resizePanes);
            document.body.classList.remove('resizing');
        }, { once: true }); // The mouseup listener should only fire once
    });

    // --- Utility Functions ---
    const apiFetch = async (path, options = {}) => {
        const response = await fetch(`${API_BASE_URL}${path}`, options);

        // If the resource is not found, treat it as an empty list, not an error.
        if (response.status === 404) {
            return [];
        }

        if (!response.ok) {
            const errorText = await response.text(); // Get the raw error text first
            try {
                // Try to parse it as JSON to get a structured error message
                const errorJson = JSON.parse(errorText);
                throw new Error(errorJson.message || errorText || `HTTP error! status: ${response.status}`);
            } catch (e) {
                // If it's not JSON, or if the JSON doesn't have a 'message' field,
                // throw the raw text.
                throw new Error(errorText || `HTTP error! status: ${response.status}`);
            }
        }
        if (response.status === 204) return null; // No Content
        
        // Handle empty response body for successful requests
        const text = await response.text();
        return text ? JSON.parse(text) : []; // Return empty array for empty body
    };

    const createButton = (text, className, onClick) => {
        const button = document.createElement('button');
        button.textContent = text;
        button.className = className;
        button.addEventListener('click', (e) => {
            e.stopPropagation();
            onClick();
        });
        return button;
    };

    const createInputElement = (type, id, placeholder) => {
        const input = document.createElement(type);
        input.id = id;
        if (placeholder) input.placeholder = placeholder;
        return input;
    };

    // --- Global Form Group Helper ---
    const createFormGroup = (form, item, labelText, inputElement, { description, example, readOnly = false, required = false }) => {
        const group = document.createElement('div');
        group.className = 'form-group';

        const labelWrapper = document.createElement('div');
        labelWrapper.className = 'label-wrapper';

        const label = document.createElement('label');
        label.textContent = labelText + (required ? ' *' : '');
        labelWrapper.appendChild(label);

        if (description) {
            const helpText = document.createElement('small');
            helpText.className = 'help-text';
            helpText.textContent = description;
            labelWrapper.appendChild(helpText);
        }

        group.appendChild(labelWrapper);

        if (readOnly) {
            const readOnlyDiv = document.createElement('div');
            // Convert label to camelCase to match item property, e.g., "Routing Chain" -> "routingChain"
            const key = labelText.toLowerCase().replace(/ \w/g, c => c.charAt(1).toUpperCase());
            // Use the derived key to get the value, with fallbacks for name.
            readOnlyDiv.textContent = item[key] || 'N/A';
            readOnlyDiv.className = 'read-only-field';
            group.appendChild(readOnlyDiv);
            form.appendChild(group);
            return null; // Return null for read-only fields
        }

        group.appendChild(inputElement);
        if (example && (inputElement.type === 'text' || inputElement.type === 'number' || inputElement.tagName.toLowerCase() === 'textarea')) {
            inputElement.placeholder = example;
        }

        form.appendChild(group);
       return inputElement;
    };

    // --- Details UI Builder for Realm ---
    const buildRealmDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const title = document.createElement('h2');
        title.textContent = `Details for Realm: ${item.name}`;
        detailsContainer.appendChild(title);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'The unique URN for the realm.', readOnly: true });

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the realm.',
            example: 'CMS System',
            required: true
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea', 'realm-description-input'), {
            description: 'A description for the realm.',
            example: 'Realm for the CMS system'
        });
        descriptionInput.value = item.description || '';

        const cacertInput = createFormGroup(form, item, 'CA Certificate', createInputElement('textarea'), {
            description: 'The CA certificate for the realm in PEM format.',
            example: '-----BEGIN CERTIFICATE-----\n...',
            required: true
        });
        cacertInput.rows = 5;
        cacertInput.value = item.cacert || '';

        const signingKeyInput = createFormGroup(form, item, 'Signing Key', createInputElement('textarea'), {
            description: 'The private key used for signing tokens in PEM format.',
            example: 'a-very-secret-signing-key-that-is-long',
            required: true
        });
        signingKeyInput.rows = 2;
        signingKeyInput.value = item.signingKey || '';

        const sessionTimeoutInput = createFormGroup(form, item, 'Session Timeout (sec)', createInputElement('input'), {
            description: 'Session token expiration in seconds. Defaults to 2592000 (30 days).',
            example: '2592000'
        });
        sessionTimeoutInput.type = 'number';
        sessionTimeoutInput.value = item.sessionTimeout || '';

        const administratorsInput = createFormGroup(form, item, 'Administrators (comma-separated)', createInputElement('textarea'), {
            description: 'User IDs of realm administrators.',
            example: 'admin1@example.com,admin2@example.com'
        });
        administratorsInput.value = (item.administrators || []).join(', ');

        const expiredAtInput = createFormGroup(form, item, 'Expired At', createInputElement('input'), {
            description: 'Realm expiration date in ISO8601 format.',
            example: '2024-12-31T23:59:59Z'
        });
        expiredAtInput.size = 50;
        expiredAtInput.value = item.expiredAt || '';

        const disabledGroup = document.createElement('div');
        disabledGroup.className = 'form-group form-group-checkbox';
        const disabledInput = createInputElement('input');
        disabledInput.type = 'checkbox';
        disabledInput.checked = !!item.disabled;
        const disabledLabel = document.createElement('label');
        disabledLabel.appendChild(disabledInput);
        disabledLabel.appendChild(document.createTextNode(' Disabled'));
        disabledLabel.title = 'Check to disable the realm.';
        disabledGroup.appendChild(disabledLabel);
        form.appendChild(disabledGroup);

        detailsContainer.appendChild(form);

        return { form, titleInput, descriptionInput, cacertInput, signingKeyInput, sessionTimeoutInput, administratorsInput, expiredAtInput, disabledInput };
    };

    // --- Details UI Builder for Zone ---
    const buildZoneDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const title = document.createElement('h2');
        title.textContent = `Details for Zone: ${item.name}`;
        detailsContainer.appendChild(title);
    
        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());
    
        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'URN of the zone.', readOnly: true });
        createFormGroup(form, item, 'Realm', null, { description: 'Name of the parent realm.', readOnly: true });

        // --- Editable properties ---
        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the zone, used as a FQDN suffix. Cannot be changed after creation.',
            example: 'example.com',
            required: true,
            readOnly: !!item.urn // The zone name is the ID and should not be changed after creation.
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name || '';
        }

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the zone.',
            example: 'CMS System Staging Environment Zone',
            required: true
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), {
            description: 'A description for the zone.',
            example: 'Zone for the staging environment of the CMS system.'
        });
        descriptionInput.value = item.description || '';

        const dnsProviderInput = createFormGroup(form, item, 'DNS Provider', createInputElement('input'), {
            description: 'URN of the DNS provider for managing DNS records in this zone.',
            example: 'urn:chip-in:service:example-realm:example-zone-route53'
        });
        dnsProviderInput.size = 50;
        dnsProviderInput.value = item.dnsProvider || '';

        const acmeProviderInput = createFormGroup(form, item, 'ACME Provider', createInputElement('input'), {
            description: 'URL of the ACME certificate provider for obtaining SSL/TLS certificates.',
            example: 'https://acme-v02.api.letsencrypt.org/directory'
        });
        acmeProviderInput.size = 50;
        acmeProviderInput.value = item.acmeCertificateProvider || '';

        const attributesContainer = document.createElement('div'); // Dummy container for compatibility

        return { form, titleInput, descriptionInput, dnsProviderInput, acmeProviderInput, attributesContainer };
    };

    // --- Details UI Builder for Hub (Corrected based on new schema) ---
    const buildHubDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const mainTitle = document.createElement('h2');
        mainTitle.textContent = `Details for Hub: ${item.name}`;
        detailsContainer.appendChild(mainTitle);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'Unique URN for the Hub.', readOnly: true, isHeader: true });
        createFormGroup(form, item, 'Realm', null, { description: 'URN of the parent realm.', readOnly: true, isHeader: true });

        // --- Editable properties ---
        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the SPN Hub. Used as part of the URN. Cannot be changed after creation.',
            example: 'hub1',
            required: true,
            readOnly: !!item.urn
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name;
        }

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the SPN Hub.',
            example: 'CMS System Private Network Hub',
            required: true
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), {
            description: 'A description for the SPN Hub.',
            example: 'Hub for the private network of the CMS system.'
        });
        descriptionInput.value = item.description || '';

        const fqdnInput = createFormGroup(form, item, 'FQDN', createInputElement('input'), {
            description: 'The FQDN of the SPN Hub server.',
            example: 'core.stg.chip-in.net',
            required: true
        });
        fqdnInput.size = 50;
        fqdnInput.value = item.fqdn || '';

        const serverPortInput = createFormGroup(form, item, 'Server Port', createInputElement('input'), {
            description: 'The port number of the SPN Hub server. Defaults to 443.',
            example: '443'
        });
        serverPortInput.type = 'number';
        serverPortInput.value = item.serverPort || '';

        const serverCertInput = createFormGroup(form, item, 'Server Certificate', createInputElement('textarea'), {
            description: 'The server certificate in PEM format for mTLS.',
            example: '-----BEGIN CERTIFICATE-----\n...',
            required: true
        });
        serverCertInput.rows = 5;
        serverCertInput.value = item.serverCert || '';

        const serverCertKeyInput = createFormGroup(form, item, 'Server Certificate Key', createInputElement('textarea'), {
            description: 'The private key for the server certificate in PEM format for mTLS.',
            example: '-----BEGIN RSA PRIVATE KEY-----\n...',
            required: true
        });
        serverCertKeyInput.rows = 5;
        serverCertKeyInput.value = item.serverCertKey || '';

        // No custom attributes for Hub
        const attributesContainer = document.createElement('div'); // Dummy container

        return { form, nameInput, titleInput, descriptionInput, fqdnInput, serverPortInput, serverCertInput, serverCertKeyInput, attributesContainer };
    };

    // --- Details UI Builder for VirtualHost ---
    const buildVirtualHostDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const mainTitle = document.createElement('h2');
        mainTitle.textContent = `Details for Virtual Host: ${item.name}`;
        detailsContainer.appendChild(mainTitle);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'Unique URN for the Virtual Host.', readOnly: true });
        createFormGroup(form, item, 'Realm', null, { description: 'URN of the parent realm.', readOnly: true, isHeader: true });

        // --- Editable properties ---
        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the virtual host. Used as part of the URN. Cannot be changed after creation.',
            example: 'www', required: true,
            readOnly: !!item.urn
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name;
        }

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the virtual host.',
            example: 'Corporate Website', required: true
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), {
            description: 'A description for the virtual host.',
            example: 'The main corporate website.'
        });
        descriptionInput.value = item.description || '';

        const subdomainInput = createFormGroup(form, item, 'Subdomain', createInputElement('input'), {
            description: 'URN of the subdomain this virtual host belongs to.',
            example: 'urn:chip-in:subdomain:example-realm:stg',
            required: true
        });
        subdomainInput.size = 50;
        subdomainInput.value = item.subdomain || '';

        const routingChainInput = createFormGroup(form, item, 'Routing Chain', createInputElement('input'), { description: 'URN of the routing chain that handles requests.', example: 'urn:chip-in:service:example-realm:example-routing-chain', required: true });
        routingChainInput.size = 50;
        routingChainInput.value = item.routingChain || '';

        const accessLogRecorderInput = createFormGroup(form, item, 'Access Log Recorder', createInputElement('input'), {
            description: 'URN of the service to record access logs. Defaults to standard output.',
            example: 'urn:chip-in:service:example-realm:access-log-service'
        });
        accessLogRecorderInput.size = 50;
        accessLogRecorderInput.value = item.accessLogRecorder || '';

        const accessLogMaxValueLengthInput = createFormGroup(form, item, 'Access Log Max Value Length', createInputElement('input'), {
            description: 'Max length for access log values. Defaults to 512.',
            example: '512'
        });
        accessLogMaxValueLengthInput.type = 'number';
        accessLogMaxValueLengthInput.value = item.accessLogMaxValueLength || '';

        const accessLogFormatInput = createFormGroup(form, item, 'Access Log Format (JSON)', createInputElement('textarea'), {
            description: 'Format for access log output in JSON.'
        });
        accessLogFormatInput.rows = 8;
        accessLogFormatInput.style.fontFamily = 'monospace';
        accessLogFormatInput.value = item.accessLogFormat ? JSON.stringify(item.accessLogFormat, null, 2) : '';

        const certificateInput = createFormGroup(form, item, 'Certificate (PEM)', createInputElement('textarea'), {
            description: 'Server certificate in PEM format. Can be a chain.'
        });
        certificateInput.rows = 8;
        certificateInput.value = item.certificate ? item.certificate.join('\n') : '';

        const keyInput = createFormGroup(form, item, 'Key (PEM)', createInputElement('textarea'), {
            description: 'Private key for the server certificate in PEM format.'
        });
        keyInput.rows = 5;
        keyInput.value = item.key || '';

        const disabledInput = createFormGroup(form, item, 'Disabled', createInputElement('input'), {
            description: 'If checked, the virtual host will be disabled.'
        });
        disabledInput.type = 'checkbox';
        disabledInput.checked = item.disabled === true;

        // No custom attributes for VirtualHost
        const attributesContainer = document.createElement('div'); // Dummy container

        return { form, nameInput, titleInput, descriptionInput, subdomainInput, routingChainInput, accessLogRecorderInput, accessLogMaxValueLengthInput, accessLogFormatInput, certificateInput, keyInput, disabledInput, attributesContainer };
    };

    // --- Details UI Builder for RoutingChain ---
    const buildRoutingChainDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const mainTitle = document.createElement('h2');
        mainTitle.textContent = `Details for Routing Chain: ${item.name}`;
        detailsContainer.appendChild(mainTitle);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'Unique URN for the Routing Chain.', readOnly: true });
        createFormGroup(form, item, 'Realm', null, { description: 'URN of the parent realm.', readOnly: true, isHeader: true });

        // --- Editable properties ---
        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the routing chain. Used as part of the URN. Cannot be changed after creation.',
            example: 'main-chain', required: true,
            readOnly: !!item.urn
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name;
        }

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), { description: 'A human-readable title for the routing chain.', example: 'Main Routing Chain', required: true });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), { description: 'A description for the routing chain.', example: 'Handles all incoming requests for the main service.' });
        descriptionInput.value = item.description || '';

        const rulesInput = createFormGroup(form, item, 'Rules (array of JSON)', createInputElement('textarea'), {
            description: `An array of rules. Each rule has a 'match' condition (evalexpr format) and an 'action' object. Valid action types are: setDeviceId, checkoutServices, proxy, redirect, jump, setVariables, setHeaders, authentication.`,
            example: JSON.stringify([
                {
                    "match": "request.path.starts_with(\"/api/\")",
                    "action": {
                        "type": "proxy",
                        "target": "http://backend-service:8080"
                    }
                }
            ], null, 2)
        });
        rulesInput.rows = 15;
        rulesInput.style.fontFamily = 'monospace';
        rulesInput.value = item.rules ? JSON.stringify(item.rules, null, 2) : '[]';

        // --- UI Helper for Rules ---
        const rulesGroup = rulesInput.closest('.form-group');
        if (rulesGroup) {
            const helperContainer = document.createElement('div');
            helperContainer.className = 'rules-helper';

            const helperTitle = document.createElement('small');
            helperTitle.textContent = 'Insert rule template:';
            helperContainer.appendChild(helperTitle);

            const ruleTemplates = {
                'Proxy': { match: 'request.path.starts_with("/api/")', action: { type: 'proxy', target: 'http://backend-service.internal' } },
                'Redirect': { match: 'request.path == "/old-path"', action: { type: 'redirect', location: 'https://example.com/new-path', status: 302 } },
                'Set Headers': { match: 'true', action: { type: 'setHeaders', headers: { 'X-Custom-Header': 'MyValue' } } },
                'Authentication': { match: 'request.path.starts_with("/admin/")', action: { type: 'authentication', authenticator: 'urn:chip-in:service:your-realm:auth-service', allow: 'claims.groups.contains("admin")' } },
                'Jump': { match: 'request.path == "/jump"', action: { type: 'jump', chain: 'urn:chip-in:routing-chain:your-realm:another-chain' } },
            };

            Object.entries(ruleTemplates).forEach(([name, template]) => {
                const btn = createButton(name, 'helper-btn', () => {
                    try {
                        const currentRules = JSON.parse(rulesInput.value || '[]');
                        if (!Array.isArray(currentRules)) {
                            throw new Error('Current content is not a JSON array.');
                        }
                        currentRules.push(template);
                        rulesInput.value = JSON.stringify(currentRules, null, 2);
                    } catch (e) {
                        alert(`Could not add template. Please ensure the content is a valid JSON array.\nError: ${e.message}`);
                    }
                });
                helperContainer.appendChild(btn);
            });

            rulesGroup.appendChild(helperContainer);
        }

        const attributesContainer = document.createElement('div'); // Dummy container

        return { form, nameInput, titleInput, descriptionInput, rulesInput, attributesContainer };
    };

    // --- Details UI Builder for Subdomain ---
    const buildSubdomainDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const mainTitle = document.createElement('h2');
        mainTitle.textContent = `Details for Subdomain: ${item.name}`;
        detailsContainer.appendChild(mainTitle);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'The unique URN for the subdomain.', readOnly: true, isHeader: true });
        createFormGroup(form, item, 'Realm', null, {
            description: 'URN of the realm this subdomain belongs to.',
            example: 'urn:chip-in:realm:your-realm',
            readOnly: true, isHeader: true
        });
        createFormGroup(form, item, 'FQDN', null, { description: 'The Fully Qualified Domain Name (FQDN) of the subdomain. Automatically generated by the server.', readOnly: true });
        createFormGroup(form, item, 'Zone', null, { description: 'The URN of the zone this subdomain belongs to.', readOnly: true });


        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the subdomain. Used as part of the URN.',
            example: 'stg', required: true,
            // Name is part of the URN and cannot be changed after creation.
            readOnly: !!item.urn 
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name;
        }

        // --- Editable properties ---
        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the subdomain.',
            example: 'Staging Environment',
            required: true // title is required
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), {
            description: 'A description for the subdomain.',
            example: 'Staging environment subdomain.' // description is optional
        });
        descriptionInput.value = item.description || '';

        const destinationRealmInput = createFormGroup(form, item, 'Destination Realm', createInputElement('input'), {
            description: 'URN of the realm to which this subdomain is lent.',
            example: 'urn:chip-in:realm:another-realm'
        });
        destinationRealmInput.size = 50;
        destinationRealmInput.value = item.destinationRealm || '';

        const shareCookieGroup = document.createElement('div');
        shareCookieGroup.className = 'form-group form-group-checkbox';
        const shareCookieInput = createInputElement('input', 'subdomain-sharecookie-input');
        shareCookieInput.type = 'checkbox';
        shareCookieInput.checked = !!item.shareCookie;
        const shareCookieLabel = document.createElement('label');
        shareCookieLabel.appendChild(shareCookieInput);
        shareCookieLabel.appendChild(document.createTextNode(' Share Session Cookie'));
        shareCookieLabel.title = 'If checked, session cookies will be shared across virtual hosts in this subdomain.';
        shareCookieGroup.appendChild(shareCookieLabel);
        form.appendChild(shareCookieGroup);

        // No custom attributes for Subdomain
        const attributesContainer = document.createElement('div'); // Dummy container

        return { form, nameInput, titleInput, descriptionInput, destinationRealmInput, shareCookieInput, attributesContainer };
    };

    // --- Details UI Builder for Service ---
    const buildServiceDetailsUI = (item, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content

        const mainTitle = document.createElement('h2');
        mainTitle.textContent = `Details for Service: ${item.name}`;
        detailsContainer.appendChild(mainTitle);

        const pathElement = document.createElement('p');
        pathElement.textContent = path;
        pathElement.className = 'details-path';
        detailsContainer.appendChild(pathElement);

        const form = document.createElement('form');
        form.addEventListener('submit', e => e.preventDefault());

        // --- Read-only properties ---
        createFormGroup(form, item, 'URN', null, { description: 'The unique URN for the service.', readOnly: true });
        createFormGroup(form, item, 'Realm', null, { description: 'URN of the parent realm.', readOnly: true });
        createFormGroup(form, item, 'Hub', null, { description: 'URN of the parent hub.', readOnly: true });

        const nameInput = createFormGroup(form, item, 'Name', createInputElement('input'), {
            description: 'The name of the service. Used as part of the URN.',
            example: 'user-auth-service', required: true,
            readOnly: !!item.urn
        });
        if (nameInput) {
            nameInput.size = 50;
            nameInput.value = item.name;
        }

        const titleInput = createFormGroup(form, item, 'Title', createInputElement('input'), {
            description: 'A human-readable title for the service.',
            example: 'CMS System Authorization Service',
            required: true
        });
        titleInput.size = 50;
        titleInput.value = item.title || '';

        const descriptionInput = createFormGroup(form, item, 'Description', createInputElement('textarea'), {
            description: 'A description for the service.',
            example: 'Handles user authentication and authorization.'
        });
        descriptionInput.value = item.description || '';

        const providerInput = createFormGroup(form, item, 'Provider (URNs, comma-separated)', createInputElement('textarea'), {
            description: 'A list of provider identifiers (URNs) for the service.',
            example: 'oidc-idp-1,oidc-idp-2',
            required: true
        });
        providerInput.value = (item.provider || []).join(', ');

        const consumersInput = createFormGroup(form, item, 'Consumers (URNs, comma-separated)', createInputElement('textarea'), {
            description: 'A list of consumer identifiers (URNs) for the service.',
            example: 'urn:chip-in:end-point:example-zone:example.com:api-gateway',
            required: true
        });
        consumersInput.value = (item.consumers || []).join(', ');

        const singletonGroup = document.createElement('div');
        singletonGroup.className = 'form-group form-group-checkbox';
        const singletonInput = createInputElement('input');
        singletonInput.type = 'checkbox';
        singletonInput.checked = !!item.singleton;
        const singletonLabel = document.createElement('label');
        singletonLabel.appendChild(singletonInput);
        singletonLabel.appendChild(document.createTextNode(' Singleton Service'));
        singletonLabel.title = 'If checked, only one instance of this service will run in the SPN.';
        singletonGroup.appendChild(singletonLabel);
        form.appendChild(singletonGroup);

        // --- Availability Management Section ---
        const amTitle = document.createElement('h3');
        amTitle.textContent = 'Availability Management';
        form.appendChild(amTitle);

        const am = item.availabilityManagement || {};
        const clusterManagerUrnInput = createFormGroup(form, am, 'Cluster Manager URN', createInputElement('input'), {
            description: 'The URN of the cluster manager service for the container cluster.',
            example: 'urn:chip-in:service:master:cluster-manager'
        });
        clusterManagerUrnInput.size = 50;
        clusterManagerUrnInput.value = am.clusterManagerUrn || '';

        const serviceIdInput = createFormGroup(form, am, 'Service ID in Cluster', createInputElement('input'), {
            description: 'The ID of the microservice within the cluster.',
            example: 'authz-rbac-service'
        });
        serviceIdInput.size = 50;
        serviceIdInput.value = am.serviceId || '';

        const startAtInput = createFormGroup(form, am, 'Start At', createInputElement('input'), {
            description: 'Time to start the service (ISO 8601).',
            example: '2024-01-01T09:00:00Z'
        });
        startAtInput.size = 50;
        startAtInput.value = am.startAt || '';

        const stopAtInput = createFormGroup(form, am, 'Stop At', createInputElement('input'), {
            description: 'Time to stop the service (ISO 8601).',
            example: '2024-01-01T18:00:00Z'
        });
        stopAtInput.size = 50;
        stopAtInput.value = am.stopAt || '';

        const ondemandStartGroup = document.createElement('div');
        ondemandStartGroup.className = 'form-group form-group-checkbox';
        const ondemandStartInput = createInputElement('input');
        ondemandStartInput.type = 'checkbox';
        ondemandStartInput.checked = !!am.ondemandStart;
        const ondemandStartLabel = document.createElement('label');
        ondemandStartLabel.appendChild(ondemandStartInput);
        ondemandStartLabel.appendChild(document.createTextNode(' Ondemand Start'));
        ondemandStartLabel.title = 'If checked, the service can be started on-demand.';
        ondemandStartGroup.appendChild(ondemandStartLabel);
        form.appendChild(ondemandStartGroup);

        const idleTimeoutInput = createFormGroup(form, am, 'Idle Timeout (sec)', createInputElement('input'), {
            description: 'Timeout in seconds to stop the service after being idle.',
            example: '300'
        });
        idleTimeoutInput.type = 'number';
        idleTimeoutInput.value = am.idleTimeout || '';

        const imageInput = createFormGroup(form, am, 'Container Image', createInputElement('input'), {
            description: 'The container image to run for the service.',
            example: 'nginx:latest'
        });
        imageInput.size = 50;
        imageInput.value = am.image || '';

        const commandInput = createFormGroup(form, am, 'Command (JSON array)', createInputElement('textarea'), {
            description: 'The command to run in the container, as a JSON array of strings.',
            example: '["/bin/sh", "-c", "echo hello"]'
        });
        commandInput.value = am.command ? JSON.stringify(am.command) : '';

        const envInput = createFormGroup(form, am, 'Environment (JSON array)', createInputElement('textarea'), {
            description: 'Environment variables as a JSON array of {name, value} objects.',
            example: '[{"name": "ENV_VAR", "value": "some_value"}]'
        });
        envInput.value = am.env ? JSON.stringify(am.env, null, 2) : '';

        const mountPointsInput = createFormGroup(form, am, 'Mount Points (JSON array)', createInputElement('textarea'), {
            description: 'Mount points as a JSON array of {volumeSize, target} objects.',
            example: '[{"volumeSize": 10, "target": "/data"}]'
        });
        mountPointsInput.value = am.mountPoints ? JSON.stringify(am.mountPoints, null, 2) : '';

        detailsContainer.appendChild(form);

        const attributesContainer = document.createElement('div'); // Dummy container for compatibility

        return { form, nameInput, titleInput, descriptionInput, providerInput, consumersInput, singletonInput, clusterManagerUrnInput, serviceIdInput, startAtInput, stopAtInput, ondemandStartInput, idleTimeoutInput, imageInput, commandInput, envInput, mountPointsInput, attributesContainer };
    };

    // --- Item Type Configuration ---
    // Central configuration object for each item type to improve maintainability.
    const itemConfigs = {
        'realms': {
            buildDetailsUI: buildRealmDetailsUI,
            displayName: 'Realm',
            buildSavePayload: (inputs) => {
                const sessionTimeout = inputs.sessionTimeoutInput.value;
                const administrators = inputs.administratorsInput.value;
                return {
                    title: inputs.titleInput.value,
                    description: inputs.descriptionInput.value,
                    cacert: inputs.cacertInput.value,
                    signingKey: inputs.signingKeyInput.value,
                    sessionTimeout: sessionTimeout ? parseInt(sessionTimeout, 10) : null,
                    administrators: administrators ? administrators.split(',').map(s => s.trim()).filter(Boolean) : null,
                    expiredAt: inputs.expiredAtInput.value || null,
                    disabled: inputs.disabledInput.checked,
                };
            },
            children: ['zones', 'hubs', 'virtual-hosts', 'routing-chains'],
        },
        'zones': {
            buildDetailsUI: buildZoneDetailsUI,
            displayName: 'Zone',
            buildSavePayload: (inputs, item) => ({
                name: item.name,
                title: inputs.titleInput.value,
                description: inputs.descriptionInput.value,
                dnsProvider: inputs.dnsProviderInput.value || null,
                acmeCertificateProvider: inputs.acmeProviderInput.value || null,
            }),
            children: ['subdomains'],
            buildCreatePayload: (name) => ({ name: name, title: `New Zone ${name}` }),
        },
        'hubs': {
            buildDetailsUI: buildHubDetailsUI,
            displayName: 'Hub',
            buildSavePayload: (inputs, item) => ({
                name: item.name,
                title: inputs.titleInput.value,
                description: inputs.descriptionInput.value,
                fqdn: inputs.fqdnInput.value,
                serverPort: parseInt(inputs.serverPortInput.value, 10) || 443,
                serverCert: inputs.serverCertInput.value,
                serverCertKey: inputs.serverCertKeyInput.value,
            }),
            children: ['services'],
            buildCreatePayload: (name) => ({
                name: name,
                title: `New Hub ${name}`,
                fqdn: 'example.com',
                serverCert: '-----BEGIN CERTIFICATE-----\n-----END CERTIFICATE-----',
                serverCertKey: '-----BEGIN RSA PRIVATE KEY-----\n-----END RSA PRIVATE KEY-----',
            }),
        },
        'virtual-hosts': {
            buildDetailsUI: buildVirtualHostDetailsUI,
            displayName: 'Virtual-Host',
            buildSavePayload: (inputs, item) => {
                let accessLogFormat = null;
                const accessLogFormatText = inputs.accessLogFormatInput.value;
                if (accessLogFormatText) {
                    try {
                        accessLogFormat = JSON.parse(accessLogFormatText);
                    } catch (e) {
                        throw new Error(`Invalid JSON in Access Log Format: ${e.message}`);
                    }
                }
                const certs = inputs.certificateInput.value;
                const certificate = certs ? certs.split('\n').filter(c => c.trim() !== '') : null;
                const accessLogMaxValueLength = inputs.accessLogMaxValueLengthInput.value;

                return {
                    name: item.name,
                    title: inputs.titleInput.value,
                    description: inputs.descriptionInput.value,
                    subdomain: inputs.subdomainInput.value,
                    routingChain: inputs.routingChainInput.value,
                    accessLogRecorder: inputs.accessLogRecorderInput.value || null,
                    accessLogMaxValueLength: accessLogMaxValueLength ? parseInt(accessLogMaxValueLength, 10) : null,
                    accessLogFormat: accessLogFormat,
                    certificate: certificate,
                    key: inputs.keyInput.value || null,
                    disabled: inputs.disabledInput.checked,
                };
            },
            buildCreatePayload: (name) => ({
                name: name,
                title: `New Virtual Host ${name}`,
                subdomain: 'urn:chip-in:subdomain:your-realm:your-subdomain',
                routingChain: 'urn:chip-in:service:your-realm:your-routing-chain',
            }),
        },
        'routing-chains': {
            buildDetailsUI: buildRoutingChainDetailsUI,
            displayName: 'Routing-Chain',
            buildSavePayload: (inputs, item) => {
                let rules;
                try {
                    rules = JSON.parse(inputs.rulesInput.value);
                } catch (e) {
                    throw new Error(`Invalid JSON in Rules: ${e.message}`);
                }

                // Validate that each rule has 'match' and 'action' properties
                if (Array.isArray(rules)) {
                    rules.forEach((rule, index) => {
                        if (typeof rule !== 'object' || rule === null || !('match' in rule) || !('action' in rule)) {
                            throw new Error(`Invalid rule at index ${index}: Each rule must be an object with 'match' and 'action' properties.`);
                        }
                    });
                }

                return {
                    name: item.name,
                    title: inputs.titleInput.value,
                    description: inputs.descriptionInput.value,
                    rules: rules,
                };
            },
            buildCreatePayload: (name) => ({ name: name, title: `New ${name}`, rules: [] }),
        },
        'subdomains': {
            buildDetailsUI: buildSubdomainDetailsUI,
            displayName: 'Subdomain',
            buildSavePayload: (inputs, item) => ({
                name: item.name || inputs.nameInput.value,
                title: inputs.titleInput.value,
                description: inputs.descriptionInput.value,
                destinationRealm: inputs.destinationRealmInput.value || null,
                shareCookie: inputs.shareCookieInput.checked,
            }),
            buildCreatePayload: (name) => ({ name: name, title: name, description: `New Subdomain ${name}` }),
        },
        'services': {
            buildDetailsUI: buildServiceDetailsUI,
            displayName: 'Service',
            buildSavePayload: (inputs, item) => {
                let availabilityManagement = null;
                const amUrn = inputs.clusterManagerUrnInput.value;
                const amSid = inputs.serviceIdInput.value;

                // Only build the availabilityManagement object if the core identifiers are present.
                if (amUrn && amSid) {
                    availabilityManagement = {
                        clusterManagerUrn: amUrn,
                        serviceId: amSid,
                        ondemandStart: inputs.ondemandStartInput.checked,
                    };

                    if (inputs.startAtInput.value) availabilityManagement.startAt = inputs.startAtInput.value;
                    if (inputs.stopAtInput.value) availabilityManagement.stopAt = inputs.stopAtInput.value;
                    if (inputs.idleTimeoutInput.value) availabilityManagement.idleTimeout = parseInt(inputs.idleTimeoutInput.value, 10);
                    if (inputs.imageInput.value) availabilityManagement.image = inputs.imageInput.value;

                    try {
                        if (inputs.commandInput.value) availabilityManagement.command = JSON.parse(inputs.commandInput.value);
                        if (inputs.envInput.value) availabilityManagement.env = JSON.parse(inputs.envInput.value);
                        if (inputs.mountPointsInput.value) availabilityManagement.mountPoints = JSON.parse(inputs.mountPointsInput.value);
                    } catch (e) {
                        throw new Error(`Invalid JSON in Availability Management fields: ${e.message}`);
                    }
                }

                return {
                    name: item.name,
                    title: inputs.titleInput.value,
                    description: inputs.descriptionInput.value,
                    provider: inputs.providerInput.value.split(',').map(s => s.trim()).filter(Boolean),
                    consumers: inputs.consumersInput.value.split(',').map(s => s.trim()).filter(Boolean),
                    singleton: inputs.singletonInput.checked,
                    availabilityManagement: availabilityManagement,
                };
            },
            buildCreatePayload: (name, parentItem) => ({
                name: name,
                title: name,
                description: `New Service ${name}`,
                provider: [],
                consumers: [],
            }),
        },
    };

    // --- Generic Save Handler ---
    const saveItem = async (itemType, path, item, inputs) => {
        const config = itemConfigs[itemType];
        if (!config || !config.buildSavePayload) {
            alert(`Save functionality not configured for ${itemType}.`);
            return;
        }

        try {
            const payload = config.buildSavePayload(inputs, item);
            await apiFetch(path, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            alert('Successfully saved!');
            loadAllItems();
        } catch (error) {
            console.error(`Error saving ${itemType}:`, error);
            alert(`Error saving: ${error.message}`);
        }
    };

    // --- Details Display Function ---
    const displayDetails = (item, itemType, path) => {
        detailsContainer.innerHTML = ''; // Clear previous content
    
        const title = document.createElement('h2');
        title.textContent = `Details for ${item.name}`;
        detailsContainer.appendChild(title);
    
        // Generic attribute renderer to be used by different UI builders
        const renderAttribute = (container, key = '', value = '') => {
            const attrDiv = document.createElement('div');
            attrDiv.className = 'attribute-pair';
            
            const keyInput = createInputElement('input', '', 'key');
            keyInput.className = 'attr-key';
            keyInput.value = key;

            const createValueInput = (val) => {
                const type = typeof val;
                if (type === 'boolean') {
                    const checkbox = createInputElement('input');
                    checkbox.type = 'checkbox';
                    checkbox.checked = val;
                    checkbox.className = 'attr-value';
                    return checkbox;
                } else if (type === 'number') {
                    const numInput = createInputElement('input');
                    numInput.type = 'number';
                    numInput.value = val;
                    numInput.className = 'attr-value';
                    return numInput;
                } else if (val !== null && type === 'object') {
                    const textarea = createInputElement('textarea');
                    textarea.value = JSON.stringify(val, null, 2);
                    textarea.className = 'attr-value attr-value-json';
                    textarea.rows = 4;
                    return textarea;
                } else {
                    const textInput = createInputElement('input');
                    textInput.type = 'text';
                    textInput.value = val === null ? '' : val;
                    textInput.placeholder = val === null ? 'null' : 'value';
                    textInput.className = 'attr-value';
                    return textInput;
                }
            };
            let valueInput = createValueInput(value);

            const removeBtn = createButton('Remove', 'remove-attr-btn', () => attrDiv.remove());

            attrDiv.appendChild(keyInput);
            attrDiv.appendChild(valueInput);
            attrDiv.appendChild(removeBtn);
            container.appendChild(attrDiv);
        };

        // --- Build UI based on item type ---
        const config = itemConfigs[itemType];
        if (!config || !config.buildDetailsUI) {
            showDetailsPlaceholder();
            return;
        }

        const uiElements = config.buildDetailsUI(item, path);
        const form = uiElements.form;

        // --- Save Button ---
        const saveBtn = createButton('Save Changes', 'save-btn', () => saveItem(itemType, path, item, uiElements));
        form.appendChild(saveBtn);

        detailsContainer.appendChild(form);
    };

    // --- Render Functions for Static List ---
    const createItemElement = (item, itemType, path, level) => {
        const itemDiv = document.createElement('div');
        itemDiv.className = 'item';
        itemDiv.style.marginLeft = `${level * 20}px`; // Indent based on level

        const headerContainer = document.createElement('div');
        headerContainer.className = 'item-header';
        const config = itemConfigs[itemType];

        const idSpan = document.createElement('span');
        idSpan.className = 'item-id';
        const itemName = item.name || 'Unnamed';
        idSpan.textContent = `[${config.displayName}] ${itemName}`;
        idSpan.addEventListener('click', () => {
            // Remove highlight from the previously selected item
            if (selectedItemHeader) {
                selectedItemHeader.classList.remove('selected');
            }
            // Add highlight to the current item's header
            headerContainer.classList.add('selected');
            // Update the reference to the currently selected item's header
            selectedItemHeader = headerContainer;
            displayDetails(item, itemType, path);
        });
        headerContainer.appendChild(idSpan);

        // Add Delete button
        const deleteBtn = createButton('Delete', 'delete-btn', async () => {
            if (confirm(`Are you sure you want to delete ${itemName}?`)) {
                try {
                    await apiFetch(path, { method: 'DELETE' });
                    loadAllItems();
                } catch (error) {
                    alert(`Error deleting: ${error.message}`);
                }
            }
        });
        headerContainer.appendChild(deleteBtn);
        
        // Add forms for creating children
        if (config && config.children) {
            for (const childType of config.children) {
                const childConfig = itemConfigs[childType];
                const addBtn = createButton(`Add ${childConfig.displayName}`, '', async () => {
                    const name = prompt(`Enter new name for new ${childConfig.displayName}:`);
                    if (!name) return; // User cancelled

                    // Validate ID pattern based on type
                    if (childType === 'zones') {
                        // Hostname-like, allows periods.
                        const zonePattern = /^[a-z0-9.-]+$/; // Allows lowercase letters, numbers, dots, and hyphens.
                        if (!zonePattern.test(name)) {
                            alert('Invalid Zone name format.\n\nName must only contain lowercase alphanumeric characters, hyphens (-), and periods (.).');
                            return;
                        }
                    }
                    
                    if (!childConfig || !childConfig.buildCreatePayload) {
                        alert(`Create functionality not configured for ${childConfig.displayName}.`);
                        return;
                    }

                    try {
                        const requestBody = childConfig.buildCreatePayload(name, item);
                        const childPath = `${path}/${childType}`;
                        await apiFetch(childPath, {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify(requestBody),
                        });
                        loadAllItems();
                    } catch (error) {
                        alert(`Failed to add ${childConfig.displayName}.\n\nError: ${error.message}`);
                    }
                });
                headerContainer.appendChild(addBtn);
            }
        }

        itemDiv.appendChild(headerContainer);

        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'children-container';
        itemDiv.appendChild(childrenContainer);

        return itemDiv;
    };

    const loadAndRenderChildren = async (parentEl, path, itemType, level) => {
        const childrenContainer = parentEl.querySelector('.children-container');
        if (!childrenContainer) return;

        try {
            const items = await apiFetch(path);
            for (const item of items) {
                const itemName = item.name;
                const itemPath = `${path}/${itemName}`;
                const itemEl = createItemElement(item, itemType, itemPath, level);
                childrenContainer.appendChild(itemEl);

                // Recursively load grand-children
                const config = itemConfigs[itemType];
                if (config && config.children) {
                    await loadChildrenForItem(itemEl, itemType, itemPath, level + 1);
                }
            }
        } catch (error) {
            childrenContainer.innerHTML = `<p class="error" style="margin-left: ${level * 20}px;">Error loading ${itemConfigs[itemType].displayName}: ${error.message}</p>`;
        }
    };

    const loadChildrenForItem = async (itemEl, itemType, itemPath, level) => {
        const config = itemConfigs[itemType];
        if (!config || !config.children) return;

        for (const childType of config.children) {
            const childPath = `${itemPath}/${childType}`;
            await loadAndRenderChildren(itemEl, childPath, childType, level);
        }
    };

    const loadAllItems = async () => {
        realmsList.innerHTML = '';
        showDetailsPlaceholder();
        selectedItemHeader = null; // Reset selection on full reload

        try {
            const realms = await apiFetch('/realms');
            for (const realm of realms) {
                const realmPath = `/realms/${realm.name}`;
                const realmEl = createItemElement(realm, 'realms', realmPath, 0);
                realmsList.appendChild(realmEl);
                await loadChildrenForItem(realmEl, 'realms', realmPath, 1);
            }
        } catch (error) {
            realmsList.innerHTML = `<p class="error">Error loading realms: ${error.message}</p>`;
        }
    };

    addRealmBtn.addEventListener('click', async () => {
        const name = prompt('Enter new name for Realm:');
        if (!name) return; // User cancelled

        // Validate Realm name according to the schema pattern
        const idPattern = /^[a-zA-Z0-9_-]+$/;
        if (!idPattern.test(name)) {
            alert('Invalid name format.\n\nName must only contain alphanumeric characters, hyphens (-), and underscores (_).');
            return;
        }

        try {
            await apiFetch('/realms', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: name,
                    title: name, // Use name as title for new realms
                    description: `New realm ${name}`,
                    cacert: '-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----',
                    signingKey: 'a-very-secret-signing-key-that-is-long-enough',
                    disabled: false,
                }),
            });
            loadAllItems();
        } catch (error) {
            alert(`Failed to add Realm.\n\nError: ${error.message}`);
        }
    });

    loadAllItems();
});