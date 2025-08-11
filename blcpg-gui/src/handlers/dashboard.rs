// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::{
    http::StatusCode,
    response::Html,
};
use crate::models::{ClusterStatus, ClusterMetrics, VipStatus, HealthStatus, Event, DashboardData};

pub async fn index() -> Html<&'static str> {
    // In a real implementation, this would fetch data from the API
    // For now, we'll return a simple HTML dashboard
    
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BLC PostgreSQL HA - Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script src="https://unpkg.com/hyperscript.org@0.9.12"></script>
</head>
<body class="bg-gray-900 text-white min-h-screen">
    <!-- Navigation -->
    <nav class="bg-gray-800 border-b border-gray-700">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div class="flex items-center justify-between h-16">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <h1 class="text-xl font-bold text-white">BLC PostgreSQL HA</h1>
                    </div>
                    <div class="hidden md:block">
                        <div class="ml-10 flex items-baseline space-x-4">
                            <a href="/dashboard" class="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium">Dashboard</a>
                            <a href="/nodes" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Nodes</a>
                            <a href="/cluster" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Cluster</a>
                            <a href="/metrics" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Metrics</a>
                            <a href="/settings" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Settings</a>
                        </div>
                    </div>
                </div>
                <div class="hidden md:block">
                    <div class="ml-4 flex items-center md:ml-6">
                        <span class="text-gray-300 text-sm" id="last-updated">Last updated: Just now</span>
                    </div>
                </div>
            </div>
        </div>
    </nav>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <!-- Status Overview -->
        <div class="px-4 py-6 sm:px-0">
            <div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
                <!-- Cluster Health -->
                <div class="bg-gray-800 overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                                    </svg>
                                </div>
                            </div>
                            <div class="ml-5 w-0 flex-1">
                                <dl>
                                    <dt class="text-sm font-medium text-gray-400 truncate">Cluster Health</dt>
                                    <dd class="text-lg font-medium text-white">Healthy</dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Active Nodes -->
                <div class="bg-gray-800 overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                    </svg>
                                </div>
                            </div>
                            <div class="ml-5 w-0 flex-1">
                                <dl>
                                    <dt class="text-sm font-medium text-gray-400 truncate">Active Nodes</dt>
                                    <dd class="text-lg font-medium text-white">3/3</dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Primary Node -->
                <div class="bg-gray-800 overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="w-8 h-8 bg-yellow-500 rounded-full flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                                    </svg>
                                </div>
                            </div>
                            <div class="ml-5 w-0 flex-1">
                                <dl>
                                    <dt class="text-sm font-medium text-gray-400 truncate">Primary Node</dt>
                                    <dd class="text-lg font-medium text-white">node-1</dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Replication Lag -->
                <div class="bg-gray-800 overflow-hidden shadow rounded-lg">
                    <div class="p-5">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <div class="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13a1 1 0 102 0V9.414l1.293 1.293a1 1 0 001.414-1.414z" clip-rule="evenodd"></path>
                                    </svg>
                                </div>
                            </div>
                            <div class="ml-5 w-0 flex-1">
                                <dl>
                                    <dt class="text-sm font-medium text-gray-400 truncate">Replication Lag</dt>
                                    <dd class="text-lg font-medium text-white">0.2s</dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Node Status -->
        <div class="px-4 py-6 sm:px-0">
            <div class="bg-gray-800 shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <h3 class="text-lg leading-6 font-medium text-white">Node Status</h3>
                    <div class="mt-4">
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                            <!-- Node 1 (Primary) -->
                            <div class="bg-gray-700 rounded-lg p-4">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h4 class="text-sm font-medium text-white">node-1</h4>
                                        <p class="text-xs text-gray-400">Primary</p>
                                    </div>
                                    <div class="flex items-center">
                                        <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                                        <span class="text-xs text-gray-400">Online</span>
                                    </div>
                                </div>
                                <div class="mt-2">
                                    <div class="flex justify-between text-xs text-gray-400">
                                        <span>Health: 95%</span>
                                        <span>Connections: 12/100</span>
                                    </div>
                                </div>
                            </div>

                            <!-- Node 2 (Replica) -->
                            <div class="bg-gray-700 rounded-lg p-4">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h4 class="text-sm font-medium text-white">node-2</h4>
                                        <p class="text-xs text-gray-400">Replica</p>
                                    </div>
                                    <div class="flex items-center">
                                        <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                                        <span class="text-xs text-gray-400">Online</span>
                                    </div>
                                </div>
                                <div class="mt-2">
                                    <div class="flex justify-between text-xs text-gray-400">
                                        <span>Health: 90%</span>
                                        <span>Lag: 0.2s</span>
                                    </div>
                                </div>
                            </div>

                            <!-- Node 3 (Replica) -->
                            <div class="bg-gray-700 rounded-lg p-4">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h4 class="text-sm font-medium text-white">node-3</h4>
                                        <p class="text-xs text-gray-400">Replica</p>
                                    </div>
                                    <div class="flex items-center">
                                        <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
                                        <span class="text-xs text-gray-400">Online</span>
                                    </div>
                                </div>
                                <div class="mt-2">
                                    <div class="flex justify-between text-xs text-gray-400">
                                        <span>Health: 88%</span>
                                        <span>Lag: 0.5s</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Recent Events -->
        <div class="px-4 py-6 sm:px-0">
            <div class="bg-gray-800 shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <h3 class="text-lg leading-6 font-medium text-white">Recent Events</h3>
                    <div class="mt-4">
                        <div class="space-y-3">
                            <div class="flex items-center space-x-3">
                                <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                                <span class="text-sm text-gray-300">Node node-2 came online</span>
                                <span class="text-xs text-gray-500">2 minutes ago</span>
                            </div>
                            <div class="flex items-center space-x-3">
                                <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                                <span class="text-sm text-gray-300">Replication lag decreased to 0.2s</span>
                                <span class="text-xs text-gray-500">5 minutes ago</span>
                            </div>
                            <div class="flex items-center space-x-3">
                                <div class="w-2 h-2 bg-yellow-500 rounded-full"></div>
                                <span class="text-sm text-gray-300">Health check completed</span>
                                <span class="text-xs text-gray-500">10 minutes ago</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <script>
        // Auto-refresh every 5 seconds
        setInterval(function() {
            // Update last updated time
            const now = new Date();
            document.getElementById('last-updated').textContent = 'Last updated: ' + now.toLocaleTimeString();
            
            // In a real implementation, this would fetch fresh data from the API
            // and update the UI accordingly
        }, 5000);
    </script>
</body>
</html>
    "#;

    Html(html)
} 