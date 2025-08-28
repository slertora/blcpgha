// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn dashboard_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dashboard - BLC PostgreSQL HA</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100">
    <div class="min-h-screen">
        <!-- Navigation -->
        <nav class="bg-blue-600 text-white p-4">
            <div class="container mx-auto flex justify-between items-center">
                <h1 class="text-2xl font-bold">BLC PostgreSQL HA</h1>
                <div class="space-x-4">
                    <a href="/" class="text-blue-200 font-semibold">Dashboard</a>
                    <a href="/nodes" class="hover:text-blue-200">Nodes</a>
                    <a href="/cluster" class="hover:text-blue-200">Cluster</a>
                    <a href="/metrics" class="hover:text-blue-200">Metrics</a>
                    <a href="/settings" class="hover:text-blue-200">Settings</a>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <div class="container mx-auto p-6">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-3xl font-bold text-gray-800">Cluster Overview</h2>
                <button id="refreshBtn" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                    Refresh
                </button>
            </div>

            <!-- Loading State -->
            <div id="loadingState" class="text-center py-12">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p class="mt-4 text-gray-600">Loading cluster status...</p>
            </div>

            <!-- Error State -->
            <div id="errorState" class="hidden bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                <p id="errorMessage"></p>
            </div>

            <!-- Dashboard Content -->
            <div id="dashboardContent" class="hidden">
                <!-- Overview Cards -->
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full bg-blue-100 text-blue-600">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                </svg>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">Cluster Health</p>
                                <p id="clusterHealth" class="text-2xl font-semibold text-gray-900">--</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full bg-green-100 text-green-600">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
                                </svg>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">Active Nodes</p>
                                <p id="activeNodes" class="text-2xl font-semibold text-gray-900">--</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full bg-purple-100 text-purple-600">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"></path>
                                </svg>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">Primary Node</p>
                                <p id="primaryNode" class="text-2xl font-semibold text-gray-900">--</p>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6">
                        <div class="flex items-center">
                            <div class="p-3 rounded-full bg-yellow-100 text-yellow-600">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                </svg>
                            </div>
                            <div class="ml-4">
                                <p class="text-sm font-medium text-gray-600">Replication Lag</p>
                                <p id="replicationLag" class="text-2xl font-semibold text-gray-900">--</p>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Node Status and Recent Events -->
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <!-- Node Status -->
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h3 class="text-lg font-semibold text-gray-800 mb-4">Node Status</h3>
                        <div id="nodeStatusList" class="space-y-3">
                            <!-- Node status will be dynamically inserted here -->
                        </div>
                    </div>

                    <!-- Recent Events -->
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h3 class="text-lg font-semibold text-gray-800 mb-4">Recent Events</h3>
                        <div id="recentEventsList" class="space-y-3">
                            <!-- Events will be dynamically inserted here -->
                        </div>
                    </div>
                </div>

                <!-- Quick Actions -->
                <div class="mt-8 bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Quick Actions</h3>
                    <div class="flex flex-wrap gap-4">
                        <button onclick="window.location.href='/nodes'" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                            Manage Nodes
                        </button>
                        <button onclick="window.location.href='/cluster'" class="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700">
                            Cluster Info
                        </button>
                        <button onclick="window.location.href='/metrics'" class="bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700">
                            View Metrics
                        </button>
                        <button onclick="window.location.href='/settings'" class="bg-gray-600 text-white px-4 py-2 rounded hover:bg-gray-700">
                            Settings
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Global state
        let clusterData = null;
        let refreshInterval;

        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', function() {
            loadDashboard();
            setupEventListeners();
            startAutoRefresh();
        });

        function setupEventListeners() {
            document.getElementById('refreshBtn').addEventListener('click', loadDashboard);
        }

        function startAutoRefresh() {
            // Refresh every 30 seconds
            refreshInterval = setInterval(loadDashboard, 30000);
        }

        function stopAutoRefresh() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        }

        async function loadDashboard() {
            showLoading();
            hideError();

            try {
                const response = await fetch('/api/v1/cluster/status');
                const result = await response.json();

                if (result.success && result.data) {
                    clusterData = result.data;
                    renderDashboard();
                    showDashboardContent();
                } else {
                    showError(result.error || 'Failed to load cluster status');
                }
            } catch (error) {
                showError('Network error: ' + error.message);
            }
        }

        function renderDashboard() {
            if (!clusterData) return;

            // Update overview cards
            updateOverviewCards();
            
            // Update node status
            updateNodeStatus();
            
            // Update recent events
            updateRecentEvents();
        }

        function updateOverviewCards() {
            // Cluster Health
            const healthElement = document.getElementById('clusterHealth');
            if (clusterData.health_score !== undefined) {
                healthElement.textContent = `${clusterData.health_score}%`;
                healthElement.className = `text-2xl font-semibold ${getHealthScoreClass(clusterData.health_score)}`;
            }

            // Active Nodes
            const activeNodesElement = document.getElementById('activeNodes');
            if (clusterData.active_nodes !== undefined && clusterData.total_nodes !== undefined) {
                activeNodesElement.textContent = `${clusterData.active_nodes}/${clusterData.total_nodes}`;
            }

            // Primary Node
            const primaryNodeElement = document.getElementById('primaryNode');
            if (clusterData.primary_node) {
                primaryNodeElement.textContent = clusterData.primary_node;
            }

            // Replication Lag
            const replicationLagElement = document.getElementById('replicationLag');
            if (clusterData.replication_lag_seconds !== undefined) {
                replicationLagElement.textContent = `${clusterData.replication_lag_seconds}s`;
            }
        }

        function updateNodeStatus() {
            const nodeStatusList = document.getElementById('nodeStatusList');
            
            if (!clusterData.nodes || clusterData.nodes.length === 0) {
                nodeStatusList.innerHTML = '<p class="text-gray-500">No node information available</p>';
                return;
            }

            nodeStatusList.innerHTML = clusterData.nodes.map(node => `
                <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                    <div class="flex items-center space-x-3">
                        <div class="w-3 h-3 rounded-full ${getNodeStatusColor(node.status)}"></div>
                        <span class="font-medium text-gray-800">${node.node_id}</span>
                        ${node.is_primary ? '<span class="px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded-full">Primary</span>' : ''}
                    </div>
                    <div class="text-right">
                        <div class="text-sm font-medium text-gray-800">${node.status}</div>
                        <div class="text-xs text-gray-500">${node.health_score}% health</div>
                    </div>
                </div>
            `).join('');
        }

        function updateRecentEvents() {
            const recentEventsList = document.getElementById('recentEventsList');
            
            if (!clusterData.recent_events || clusterData.recent_events.length === 0) {
                recentEventsList.innerHTML = '<p class="text-gray-500">No recent events</p>';
                return;
            }

            recentEventsList.innerHTML = clusterData.recent_events.slice(0, 5).map(event => `
                <div class="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
                    <div class="w-2 h-2 rounded-full mt-2 ${getEventSeverityColor(event.severity)}"></div>
                    <div class="flex-1">
                        <div class="text-sm font-medium text-gray-800">${event.message}</div>
                        <div class="text-xs text-gray-500">${formatTimestamp(event.timestamp)}</div>
                    </div>
                </div>
            `).join('');
        }

        function getHealthScoreClass(score) {
            if (score >= 90) return 'text-green-600';
            if (score >= 70) return 'text-yellow-600';
            return 'text-red-600';
        }

        function getNodeStatusColor(status) {
            switch (status) {
                case 'Online': return 'bg-green-500';
                case 'Offline': return 'bg-red-500';
                case 'Maintenance': return 'bg-yellow-500';
                default: return 'bg-gray-500';
            }
        }

        function getEventSeverityColor(severity) {
            switch (severity) {
                case 'Critical': return 'bg-red-500';
                case 'Warning': return 'bg-yellow-500';
                case 'Info': return 'bg-blue-500';
                default: return 'bg-gray-500';
            }
        }

        function formatTimestamp(timestamp) {
            if (!timestamp) return 'Unknown';
            
            try {
                const date = new Date(timestamp * 1000);
                return date.toLocaleString();
            } catch {
                return 'Invalid timestamp';
            }
        }

        function showLoading() {
            document.getElementById('loadingState').classList.remove('hidden');
            document.getElementById('dashboardContent').classList.add('hidden');
            document.getElementById('errorState').classList.add('hidden');
        }

        function showDashboardContent() {
            document.getElementById('loadingState').classList.add('hidden');
            document.getElementById('dashboardContent').classList.remove('hidden');
        }

        function showError(message) {
            document.getElementById('errorMessage').textContent = message;
            document.getElementById('errorState').classList.remove('hidden');
            document.getElementById('loadingState').classList.add('hidden');
        }

        function hideError() {
            document.getElementById('errorState').classList.add('hidden');
        }

        // Cleanup on page unload
        window.addEventListener('beforeunload', function() {
            stopAutoRefresh();
        });
    </script>
</body>
</html>
    "#)
} 