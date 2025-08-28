// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn nodes_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nodes - BLC PostgreSQL HA</title>
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
                    <a href="/nodes" class="text-blue-200 font-semibold">Nodes</a>
                    <a href="/cluster" class="hover:text-blue-200">Cluster</a>
                    <a href="/metrics" class="hover:text-blue-200">Metrics</a>
                    <a href="/settings" class="hover:text-blue-200">Settings</a>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <div class="container mx-auto p-6">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-3xl font-bold text-gray-800">Cluster Nodes</h2>
                <button id="refreshBtn" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                    Refresh
                </button>
            </div>

            <!-- Loading State -->
            <div id="loadingState" class="text-center py-12">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p class="mt-4 text-gray-600">Loading nodes...</p>
            </div>

            <!-- Error State -->
            <div id="errorState" class="hidden bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                <p id="errorMessage"></p>
            </div>

            <!-- Nodes List -->
            <div id="nodesList" class="hidden space-y-4">
                <!-- Nodes will be dynamically inserted here -->
            </div>

            <!-- Add Node Button -->
            <div class="mt-8">
                <button id="addNodeBtn" class="bg-green-600 text-white px-6 py-3 rounded-lg hover:bg-green-700 flex items-center">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
                    </svg>
                    Add Node
                </button>
            </div>
        </div>
    </div>

    <script>
        // Global state
        let nodes = [];
        let refreshInterval;

        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {
            loadNodes();
            setupEventListeners();
            startAutoRefresh();
        });

        function setupEventListeners() {
            document.getElementById('refreshBtn').addEventListener('click', loadNodes);
            document.getElementById('addNodeBtn').addEventListener('click', showAddNodeModal);
        }

        function startAutoRefresh() {
            // Refresh every 30 seconds
            refreshInterval = setInterval(loadNodes, 30000);
        }

        function stopAutoRefresh() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        }

        async function loadNodes() {
            showLoading();
            hideError();

            try {
                const response = await fetch('/api/v1/nodes');
                const result = await response.json();

                if (result.success && result.data) {
                    nodes = result.data;
                    renderNodes();
                    showNodesList();
                } else {
                    showError(result.error || 'Failed to load nodes');
                }
            } catch (error) {
                showError('Network error: ' + error.message);
            }
        }

        function renderNodes() {
            const nodesList = document.getElementById('nodesList');
            
            if (nodes.length === 0) {
                nodesList.innerHTML = `
                    <div class="text-center py-12 text-gray-500">
                        <p class="text-lg">No nodes found</p>
                        <p class="text-sm">Add your first node to get started</p>
                    </div>
                `;
                return;
            }

            nodesList.innerHTML = nodes.map(node => `
                <div class="bg-white rounded-lg shadow-md p-6 border-l-4 ${getNodeBorderColor(node)}">
                    <div class="flex justify-between items-start">
                        <div class="flex-1">
                            <div class="flex items-center space-x-3 mb-4">
                                <h3 class="text-xl font-semibold text-gray-800">${node.node_id}</h3>
                                <span class="px-2 py-1 text-xs font-medium rounded-full ${getStatusBadgeClass(node.status)}">
                                    ${node.status}
                                </span>
                                ${node.is_primary ? '<span class="px-2 py-1 text-xs font-medium bg-purple-100 text-purple-800 rounded-full">Primary</span>' : ''}
                            </div>
                            
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
                                <div class="text-sm">
                                    <span class="text-gray-500">Host:</span>
                                    <span class="ml-2 font-medium">${node.host}:${node.port}</span>
                                </div>
                                <div class="text-sm">
                                    <span class="text-gray-500">Health Score:</span>
                                    <span class="ml-2 font-medium ${getHealthScoreClass(node.health_score)}">${node.health_score}%</span>
                                </div>
                                <div class="text-sm">
                                    <span class="text-gray-500">Connections:</span>
                                    <span class="ml-2 font-medium">${node.metrics?.active_connections || 0}/${node.metrics?.max_connections || 100}</span>
                                </div>
                                ${node.replication_lag_seconds ? `
                                    <div class="text-sm">
                                        <span class="text-gray-500">Replication Lag:</span>
                                        <span class="ml-2 font-medium">${node.replication_lag_seconds}s</span>
                                    </div>
                                ` : ''}
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div class="text-sm">
                                    <span class="text-gray-500">CPU:</span>
                                    <div class="w-full bg-gray-200 rounded-full h-2 mt-1">
                                        <div class="bg-blue-600 h-2 rounded-full" style="width: ${node.metrics?.cpu_usage || 0}%"></div>
                                    </div>
                                    <span class="text-xs text-gray-600">${node.metrics?.cpu_usage || 0}%</span>
                                </div>
                                <div class="text-sm">
                                    <span class="text-gray-500">Memory:</span>
                                    <div class="w-full bg-gray-200 rounded-full h-2 mt-1">
                                        <div class="bg-green-600 h-2 rounded-full" style="width: ${node.metrics?.memory_usage || 0}%"></div>
                                    </div>
                                    <span class="text-xs text-gray-600">${node.metrics?.memory_usage || 0}%</span>
                                </div>
                                <div class="text-sm">
                                    <span class="text-gray-500">Disk:</span>
                                    <div class="w-full bg-gray-200 rounded-full h-2 mt-1">
                                        <div class="bg-yellow-600 h-2 rounded-full" style="width: ${node.metrics?.disk_usage || 0}%"></div>
                                    </div>
                                    <span class="text-xs text-gray-600">${node.metrics?.disk_usage || 0}%</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="flex flex-col space-y-2 ml-4">
                            ${!node.is_primary ? `
                                <button onclick="promoteNode('${node.node_id}')" class="bg-purple-600 text-white px-3 py-2 rounded text-sm hover:bg-purple-700">
                                    Promote
                                </button>
                            ` : ''}
                            ${node.is_primary ? `
                                <button onclick="demoteNode('${node.node_id}')" class="bg-yellow-600 text-white px-3 py-2 rounded text-sm hover:bg-yellow-700">
                                    Demote
                                </button>
                            ` : ''}
                            <button onclick="toggleNode('${node.node_id}', ${node.status === 'Online'})" class="${node.status === 'Online' ? 'bg-red-600 hover:bg-red-700' : 'bg-green-600 hover:bg-green-700'} text-white px-3 py-2 rounded text-sm">
                                ${node.status === 'Online' ? 'Disable' : 'Enable'}
                            </button>
                        </div>
                    </div>
                </div>
            `).join('');
        }

        function getNodeBorderColor(node) {
            if (!node.is_healthy) return 'border-red-500';
            if (node.is_primary) return 'border-purple-500';
            return 'border-green-500';
        }

        function getStatusBadgeClass(status) {
            switch (status) {
                case 'Online': return 'bg-green-100 text-green-800';
                case 'Offline': return 'bg-red-100 text-red-800';
                case 'Maintenance': return 'bg-yellow-100 text-yellow-800';
                default: return 'bg-gray-100 text-gray-800';
            }
        }

        function getHealthScoreClass(score) {
            if (score >= 90) return 'text-green-600';
            if (score >= 70) return 'text-yellow-600';
            return 'text-red-600';
        }

        async function promoteNode(nodeId) {
            if (!confirm(`Are you sure you want to promote ${nodeId} to primary?`)) return;
            
            try {
                const response = await fetch(`/api/v1/nodes/${nodeId}/promote`, { method: 'POST' });
                const result = await response.json();
                
                if (result.success) {
                    showNotification('Node promoted successfully', 'success');
                    loadNodes();
                } else {
                    showNotification(result.error || 'Failed to promote node', 'error');
                }
            } catch (error) {
                showNotification('Network error: ' + error.message, 'error');
            }
        }

        async function demoteNode(nodeId) {
            if (!confirm(`Are you sure you want to demote ${nodeId} from primary?`)) return;
            
            try {
                const response = await fetch(`/api/v1/nodes/${nodeId}/demote`, { method: 'POST' });
                const result = await response.json();
                
                if (result.success) {
                    showNotification('Node demoted successfully', 'success');
                    loadNodes();
                } else {
                    showNotification(result.error || 'Failed to demote node', 'error');
                }
            } catch (error) {
                showNotification('Network error: ' + error.message, 'error');
            }
        }

        async function toggleNode(nodeId, isOnline) {
            const action = isOnline ? 'disable' : 'enable';
            if (!confirm(`Are you sure you want to ${action} ${nodeId}?`)) return;
            
            try {
                const endpoint = isOnline ? 'disable' : 'enable';
                const response = await fetch(`/api/v1/nodes/${nodeId}/${endpoint}`, { method: 'POST' });
                const result = await response.json();
                
                if (result.success) {
                    showNotification(`Node ${action}d successfully`, 'success');
                    loadNodes();
                } else {
                    showNotification(result.error || `Failed to ${action} node`, 'error');
                }
            } catch (error) {
                showNotification('Network error: ' + error.message, 'error');
            }
        }

        function showAddNodeModal() {
            // TODO: Implement add node modal
            alert('Add node functionality coming soon!');
        }

        function showLoading() {
            document.getElementById('loadingState').classList.remove('hidden');
            document.getElementById('nodesList').classList.add('hidden');
            document.getElementById('errorState').classList.add('hidden');
        }

        function showNodesList() {
            document.getElementById('loadingState').classList.add('hidden');
            document.getElementById('nodesList').classList.remove('hidden');
        }

        function showError(message) {
            document.getElementById('errorMessage').textContent = message;
            document.getElementById('errorState').classList.remove('hidden');
            document.getElementById('loadingState').classList.add('hidden');
        }

        function hideError() {
            document.getElementById('errorState').classList.add('hidden');
        }

        function showNotification(message, type) {
            // Simple notification - could be enhanced with a proper toast library
            const notification = document.createElement('div');
            notification.className = `fixed top-4 right-4 p-4 rounded-lg text-white z-50 ${
                type === 'success' ? 'bg-green-600' : 'bg-red-600'
            }`;
            notification.textContent = message;
            
            document.body.appendChild(notification);
            
            setTimeout(() => {
                notification.remove();
            }, 3000);
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