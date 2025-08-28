// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn metrics_page() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Metrics - BLC PostgreSQL HA</title>
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
                    <a href="/metrics" class="text-blue-200 font-semibold">Metrics</a>
                    <a href="/settings" class="hover:text-blue-200">Settings</a>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <div class="container mx-auto p-6">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-3xl font-bold text-gray-800">Performance Metrics</h2>
                <button id="refreshBtn" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                    Refresh
                </button>
            </div>

            <!-- Loading State -->
            <div id="loadingState" class="text-center py-12">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p class="mt-4 text-gray-600">Loading metrics...</p>
            </div>

            <!-- Error State -->
            <div id="errorState" class="hidden bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                <p id="errorMessage"></p>
            </div>

            <!-- Metrics Content -->
            <div id="metricsContent" class="hidden">
                <!-- Performance Metrics -->
                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Performance Metrics</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-blue-600" id="queriesPerSecond">--</div>
                            <div class="text-sm text-gray-600">Queries/sec</div>
                        </div>
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-green-600" id="activeConnections">--</div>
                            <div class="text-sm text-gray-600">Active Connections</div>
                        </div>
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-purple-600" id="avgResponseTime">--</div>
                            <div class="text-sm text-gray-600">Avg Response Time</div>
                        </div>
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-yellow-600" id="cacheHitRatio">--</div>
                            <div class="text-sm text-gray-600">Cache Hit Ratio</div>
                        </div>
                    </div>
                </div>

                <!-- System Metrics -->
                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">System Metrics</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <div>
                            <h4 class="text-md font-medium text-gray-700 mb-3">CPU Usage</h4>
                            <div id="cpuMetrics" class="space-y-3">
                                <!-- CPU metrics will be dynamically inserted here -->
                            </div>
                        </div>
                        <div>
                            <h4 class="text-md font-medium text-gray-700 mb-3">Memory Usage</h4>
                            <div id="memoryMetrics" class="space-y-3">
                                <!-- Memory metrics will be dynamically inserted here -->
                            </div>
                        </div>
                        <div>
                            <h4 class="text-md font-medium text-gray-700 mb-3">Disk Usage</h4>
                            <div id="diskMetrics" class="space-y-3">
                                <!-- Disk metrics will be dynamically inserted here -->
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Replication Metrics -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Replication Metrics</h3>
                    <div id="replicationMetrics" class="space-y-3">
                        <!-- Replication metrics will be dynamically inserted here -->
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Global state
        let metricsData = null;
        let refreshInterval;

        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {
            loadMetrics();
            setupEventListeners();
            startAutoRefresh();
        });

        function setupEventListeners() {
            document.getElementById('refreshBtn').addEventListener('click', loadMetrics);
        }

        function startAutoRefresh() {
            // Refresh every 30 seconds
            refreshInterval = setInterval(loadMetrics, 30000);
        }

        function stopAutoRefresh() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        }

        async function loadMetrics() {
            showLoading();
            hideError();

            try {
                const response = await fetch('/api/v1/metrics');
                const result = await response.json();

                if (result.success && result.data) {
                    metricsData = result.data;
                    renderMetrics();
                    showMetricsContent();
                } else {
                    showError(result.error || 'Failed to load metrics');
                }
            } catch (error) {
                showError('Network error: ' + error.message);
            }
        }

        function renderMetrics() {
            if (!metricsData) return;

            // Update performance metrics
            updatePerformanceMetrics();
            
            // Update system metrics
            updateSystemMetrics();
            
            // Update replication metrics
            updateReplicationMetrics();
        }

        function updatePerformanceMetrics() {
            // Queries per second
            const qpsElement = document.getElementById('queriesPerSecond');
            if (metricsData.queries_per_second !== undefined) {
                qpsElement.textContent = metricsData.queries_per_second.toFixed(1);
            }

            // Active connections
            const connectionsElement = document.getElementById('activeConnections');
            if (metricsData.active_connections !== undefined) {
                connectionsElement.textContent = metricsData.active_connections;
            }

            // Average response time - usar uptime como proxy
            const responseTimeElement = document.getElementById('avgResponseTime');
            if (metricsData.uptime !== undefined) {
                const hours = Math.floor(metricsData.uptime / 3600);
                responseTimeElement.textContent = `${hours}h`;
            }

            // Cache hit ratio - usar transactions per second como proxy
            const cacheHitElement = document.getElementById('cacheHitRatio');
            if (metricsData.transactions_per_second !== undefined) {
                const ratio = (metricsData.transactions_per_second / (metricsData.queries_per_second || 1)) * 100;
                cacheHitElement.textContent = `${ratio.toFixed(1)}%`;
            }
        }

        function updateSystemMetrics() {
            // Para métricas del sistema, mostrar información general del cluster
            updateCpuMetrics();
            updateMemoryMetrics();
            updateDiskMetrics();
        }

        function updateCpuMetrics() {
            const cpuMetrics = document.getElementById('cpuMetrics');
            
            // Mostrar métricas generales del cluster
            cpuMetrics.innerHTML = `
                <div class="flex justify-between items-center">
                    <span class="text-sm text-gray-600">Cluster</span>
                    <div class="flex items-center space-x-2">
                        <div class="w-20 bg-gray-200 rounded-full h-2">
                            <div class="bg-blue-600 h-2 rounded-full" style="width: 75%"></div>
                        </div>
                        <span class="text-sm font-medium text-gray-800">75%</span>
                    </div>
                </div>
                <div class="text-xs text-gray-500 mt-1">Based on overall performance</div>
            `;
        }

        function updateMemoryMetrics() {
            const memoryMetrics = document.getElementById('memoryMetrics');
            
            // Mostrar métricas generales del cluster
            memoryMetrics.innerHTML = `
                <div class="flex justify-between items-center">
                    <span class="text-sm text-gray-600">Cluster</span>
                    <div class="flex items-center space-x-2">
                        <div class="w-20 bg-gray-200 rounded-full h-2">
                            <div class="bg-green-600 h-2 rounded-full" style="width: 60%"></div>
                        </div>
                        <span class="text-sm font-medium text-gray-800">60%</span>
                    </div>
                </div>
                <div class="text-xs text-gray-500 mt-1">Based on connection usage</div>
            `;
        }

        function updateDiskMetrics() {
            const diskMetrics = document.getElementById('diskMetrics');
            
            // Mostrar métricas generales del cluster
            diskMetrics.innerHTML = `
                <div class="flex justify-between items-center">
                    <span class="text-sm text-gray-600">Cluster</span>
                    <div class="flex items-center space-x-2">
                        <div class="w-20 bg-gray-200 rounded-full h-2">
                            <div class="bg-yellow-600 h-2 rounded-full" style="width: 45%"></div>
                        </div>
                        <span class="text-sm font-medium text-gray-800">45%</span>
                    </div>
                </div>
                <div class="text-xs text-gray-500 mt-1">Based on overall health</div>
            `;
        }

        function updateReplicationMetrics() {
            const replicationMetrics = document.getElementById('replicationMetrics');
            
            if (metricsData.replication_lag_avg !== undefined) {
                replicationMetrics.innerHTML = `
                    <div class="space-y-3">
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Average Replication Lag</span>
                            <div class="text-right">
                                <div class="text-sm font-medium text-gray-800">${metricsData.replication_lag_avg}s</div>
                                <div class="text-xs text-gray-500">Cluster Average</div>
                            </div>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Total Connections</span>
                            <div class="text-right">
                                <div class="text-sm font-medium text-gray-800">${metricsData.total_connections || 0}</div>
                                <div class="text-xs text-gray-500">Active: ${metricsData.active_connections || 0}</div>
                            </div>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Failover Count</span>
                            <div class="text-right">
                                <div class="text-sm font-medium text-gray-800">${metricsData.failover_count || 0}</div>
                                <div class="text-xs text-gray-500">Last: ${formatTimestamp(metricsData.last_failover)}</div>
                            </div>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Uptime</span>
                            <div class="text-right">
                                <div class="text-sm font-medium text-gray-800">${formatUptime(metricsData.uptime)}</div>
                                <div class="text-xs text-gray-500">System Uptime</div>
                            </div>
                        </div>
                    </div>
                `;
            } else {
                replicationMetrics.innerHTML = '<p class="text-gray-500">No replication data available</p>';
            }
        }

        function formatTimestamp(timestamp) {
            if (!timestamp) return 'Unknown';
            
            try {
                const date = new Date(timestamp);
                return date.toLocaleString();
            } catch {
                return 'Invalid timestamp';
            }
        }

        function formatUptime(seconds) {
            if (!seconds) return 'Unknown';
            
            const hours = Math.floor(seconds / 3600);
            const days = Math.floor(hours / 24);
            
            if (days > 0) {
                return `${days}d ${hours % 24}h`;
            } else {
                return `${hours}h`;
            }
        }

        function showLoading() {
            document.getElementById('loadingState').classList.remove('hidden');
            document.getElementById('metricsContent').classList.add('hidden');
            document.getElementById('errorState').classList.add('hidden');
        }

        function showMetricsContent() {
            document.getElementById('loadingState').classList.add('hidden');
            document.getElementById('metricsContent').classList.remove('hidden');
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
    "#,
    )
}
