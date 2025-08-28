// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn settings_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Settings - BLC PostgreSQL HA</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100">
    <div class="min-h-screen">
        <!-- Navigation -->
        <nav class="bg-blue-600 text-white p-4">
            <div class="container mx-auto flex justify-between items-center">
                <h1 class="text-2xl font-bold">BLC PostgreSQL HA</h1>
                <div class="space-x-4">
                    <a href="/" class="hover:text-blue-200">Dashboard</a>
                    <a href="/nodes" class="hover:text-blue-200">Nodes</a>
                    <a href="/cluster" class="hover:text-blue-200">Cluster</a>
                    <a href="/metrics" class="hover:text-blue-200">Metrics</a>
                    <a href="/settings" class="text-blue-200 font-semibold">Settings</a>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <div class="container mx-auto p-6">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-3xl font-bold text-gray-800">Settings</h2>
                <button id="saveBtn" class="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700">
                    Save Changes
                </button>
            </div>

            <!-- Loading State -->
            <div id="loadingState" class="text-center py-12">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p class="mt-4 text-gray-600">Loading settings...</p>
            </div>

            <!-- Error State -->
            <div id="errorState" class="hidden bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                <p id="errorMessage"></p>
            </div>

            <!-- Success State -->
            <div id="successState" class="hidden bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-6">
                <p id="successMessage"></p>
            </div>

            <!-- Settings Content -->
            <div id="settingsContent" class="hidden space-y-6">
                <!-- General Settings -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">General Settings</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="guiTitle" class="block text-sm font-medium text-gray-700 mb-2">GUI Title</label>
                            <input type="text" id="guiTitle" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="refreshInterval" class="block text-sm font-medium text-gray-700 mb-2">Refresh Interval (seconds)</label>
                            <input type="number" id="refreshInterval" min="5" max="300" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="theme" class="block text-sm font-medium text-gray-700 mb-2">Theme</label>
                            <select id="theme" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="light">Light</option>
                                <option value="dark">Dark</option>
                                <option value="auto">Auto</option>
                            </select>
                        </div>
                        <div class="flex items-center">
                            <input type="checkbox" id="autoRefresh" class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded">
                            <label for="autoRefresh" class="ml-2 block text-sm text-gray-700">Auto-refresh enabled</label>
                        </div>
                    </div>
                </div>

                <!-- API Configuration -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">API Configuration</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="apiBaseUrl" class="block text-sm font-medium text-gray-700 mb-2">API Base URL</label>
                            <input type="url" id="apiBaseUrl" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="apiTimeout" class="block text-sm font-medium text-gray-700 mb-2">API Timeout (seconds)</label>
                            <input type="number" id="apiTimeout" min="5" max="120" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="apiRetries" class="block text-sm font-medium text-gray-700 mb-2">API Retries</label>
                            <input type="number" id="apiRetries" min="0" max="10" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="logLevel" class="block text-sm font-medium text-gray-700 mb-2">Log Level</label>
                            <select id="logLevel" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="debug">Debug</option>
                                <option value="info">Info</option>
                                <option value="warn">Warning</option>
                                <option value="error">Error</option>
                            </select>
                        </div>
                    </div>
                </div>

                <!-- Database Configuration -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Database Configuration</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="dbUrl" class="block text-sm font-medium text-gray-700 mb-2">Database URL</label>
                            <input type="text" id="dbUrl" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="maxConnections" class="block text-sm font-medium text-gray-700 mb-2">Max Connections</label>
                            <input type="number" id="maxConnections" min="1" max="100" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                    </div>
                </div>

                <!-- Server Configuration -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Server Configuration</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="serverHost" class="block text-sm font-medium text-gray-700 mb-2">Host</label>
                            <input type="text" id="serverHost" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label for="serverPort" class="block text-sm font-medium text-gray-700 mb-2">Port</label>
                            <input type="number" id="serverPort" min="1024" max="65535" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                    </div>
                </div>

                <!-- Actions -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Actions</h3>
                    <div class="flex flex-wrap gap-4">
                        <button onclick="testConnection()" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                            Test Connection
                        </button>
                        <button onclick="resetToDefaults()" class="bg-yellow-600 text-white px-4 py-2 rounded hover:bg-yellow-700">
                            Reset to Defaults
                        </button>
                        <button onclick="exportConfig()" class="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700">
                            Export Config
                        </button>
                        <button onclick="importConfig()" class="bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700">
                            Import Config
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Global state
        let settingsData = null;
        let hasChanges = false;

        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {
            loadSettings();
            setupEventListeners();
        });

        function setupEventListeners() {
            document.getElementById('saveBtn').addEventListener('click', saveSettings);
            
            // Track changes
            const inputs = document.querySelectorAll('input, select');
            inputs.forEach(input => {
                input.addEventListener('change', markAsChanged);
                input.addEventListener('input', markAsChanged);
            });
        }

        function markAsChanged() {
            hasChanges = true;
            document.getElementById('saveBtn').textContent = 'Save Changes*';
            document.getElementById('saveBtn').classList.add('bg-yellow-600', 'hover:bg-yellow-700');
        }

        async function loadSettings() {
            showLoading();
            hideMessages();

            try {
                // For now, we'll use default values since we don't have a settings API yet
                // In a real implementation, this would fetch from /api/v1/settings
                settingsData = {
                    gui: {
                        title: 'BLC PostgreSQL HA',
                        refresh_interval: 30,
                        theme: 'dark',
                        auto_refresh: true
                    },
                    api: {
                        base_url: 'http://localhost:8080',
                        timeout: 30,
                        retries: 3
                    },
                    database: {
                        url: 'sqlite:gui.db',
                        max_connections: 10
                    },
                    server: {
                        host: '0.0.0.0',
                        port: 3000
                    },
                    log_level: 'info'
                };

                renderSettings();
                showSettingsContent();
            } catch (error) {
                showError('Failed to load settings: ' + error.message);
            }
        }

        function renderSettings() {
            if (!settingsData) return;

            // GUI Settings
            document.getElementById('guiTitle').value = settingsData.gui.title || '';
            document.getElementById('refreshInterval').value = settingsData.gui.refresh_interval || 30;
            document.getElementById('theme').value = settingsData.gui.theme || 'dark';
            document.getElementById('autoRefresh').checked = settingsData.gui.auto_refresh || false;

            // API Settings
            document.getElementById('apiBaseUrl').value = settingsData.api.base_url || '';
            document.getElementById('apiTimeout').value = settingsData.api.timeout || 30;
            document.getElementById('apiRetries').value = settingsData.api.retries || 3;

            // Database Settings
            document.getElementById('dbUrl').value = settingsData.database.url || '';
            document.getElementById('maxConnections').value = settingsData.database.max_connections || 10;

            // Server Settings
            document.getElementById('serverHost').value = settingsData.server.host || '';
            document.getElementById('serverPort').value = settingsData.server.port || 3000;

            // Log Level
            document.getElementById('logLevel').value = settingsData.log_level || 'info';

            hasChanges = false;
            document.getElementById('saveBtn').textContent = 'Save Changes';
            document.getElementById('saveBtn').classList.remove('bg-yellow-600', 'hover:bg-yellow-700');
            document.getElementById('saveBtn').classList.add('bg-green-600', 'hover:bg-green-700');
        }

        async function saveSettings() {
            if (!hasChanges) {
                showSuccess('No changes to save');
                return;
            }

            try {
                const newSettings = {
                    gui: {
                        title: document.getElementById('guiTitle').value,
                        refresh_interval: parseInt(document.getElementById('refreshInterval').value),
                        theme: document.getElementById('theme').value,
                        auto_refresh: document.getElementById('autoRefresh').checked
                    },
                    api: {
                        base_url: document.getElementById('apiBaseUrl').value,
                        timeout: parseInt(document.getElementById('apiTimeout').value),
                        retries: parseInt(document.getElementById('apiRetries').value)
                    },
                    database: {
                        url: document.getElementById('dbUrl').value,
                        max_connections: parseInt(document.getElementById('maxConnections').value)
                    },
                    server: {
                        host: document.getElementById('serverHost').value,
                        port: parseInt(document.getElementById('serverPort').value)
                    },
                    log_level: document.getElementById('logLevel').value
                };

                // In a real implementation, this would POST to /api/v1/settings
                // For now, we'll just update our local state
                settingsData = newSettings;
                hasChanges = false;
                document.getElementById('saveBtn').textContent = 'Save Changes';
                document.getElementById('saveBtn').classList.remove('bg-yellow-600', 'hover:bg-yellow-700');
                document.getElementById('saveBtn').classList.add('bg-green-600', 'hover:bg-green-700');

                showSuccess('Settings saved successfully');
            } catch (error) {
                showError('Failed to save settings: ' + error.message);
            }
        }

        function testConnection() {
            // TODO: Implement connection test
            alert('Connection test functionality coming soon!');
        }

        function resetToDefaults() {
            if (confirm('Are you sure you want to reset all settings to defaults?')) {
                loadSettings();
            }
        }

        function exportConfig() {
            if (!settingsData) return;

            const dataStr = JSON.stringify(settingsData, null, 2);
            const dataBlob = new Blob([dataStr], { type: 'application/json' });
            const url = URL.createObjectURL(dataBlob);
            const link = document.createElement('a');
            link.href = url;
            link.download = 'blcpg-gui-config.json';
            link.click();
            URL.revokeObjectURL(url);
        }

        function importConfig() {
            const input = document.createElement('input');
            input.type = 'file';
            input.accept = '.json';
            input.onchange = function(e) {
                const file = e.target.files[0];
                if (file) {
                    const reader = new FileReader();
                    reader.onload = function(e) {
                        try {
                            const config = JSON.parse(e.target.result);
                            settingsData = config;
                            renderSettings();
                            showSuccess('Configuration imported successfully');
                        } catch (error) {
                            showError('Invalid configuration file: ' + error.message);
                        }
                    };
                    reader.readAsText(file);
                }
            };
            input.click();
        }

        function showLoading() {
            document.getElementById('loadingState').classList.remove('hidden');
            document.getElementById('settingsContent').classList.add('hidden');
            hideMessages();
        }

        function showSettingsContent() {
            document.getElementById('loadingState').classList.add('hidden');
            document.getElementById('settingsContent').classList.remove('hidden');
        }

        function showError(message) {
            document.getElementById('errorMessage').textContent = message;
            document.getElementById('errorState').classList.remove('hidden');
            document.getElementById('successState').classList.add('hidden');
        }

        function showSuccess(message) {
            document.getElementById('successMessage').textContent = message;
            document.getElementById('successState').classList.remove('hidden');
            document.getElementById('errorState').classList.add('hidden');
        }

        function hideMessages() {
            document.getElementById('errorState').classList.add('hidden');
            document.getElementById('successState').classList.add('hidden');
        }
    </script>
</body>
</html>
    "#)
} 