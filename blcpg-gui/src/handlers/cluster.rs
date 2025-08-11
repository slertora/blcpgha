// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn index() -> Html<&'static str> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BLC PostgreSQL HA - Cluster</title>
    <script src="https://cdn.tailwindcss.com"></script>
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
                            <a href="/dashboard" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Dashboard</a>
                            <a href="/nodes" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Nodes</a>
                            <a href="/cluster" class="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium">Cluster</a>
                            <a href="/metrics" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Metrics</a>
                            <a href="/settings" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Settings</a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </nav>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <div class="px-4 py-6 sm:px-0">
            <h1 class="text-2xl font-bold text-white mb-6">Cluster Information</h1>

            <!-- Cluster Overview -->
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
                <!-- Cluster Status -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <h3 class="text-lg font-medium text-white mb-4">Cluster Status</h3>
                        <div class="space-y-4">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Cluster Name:</span>
                                <span class="text-sm text-white">blc-cluster</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Consensus Backend:</span>
                                <span class="text-sm text-white">Raft</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Leader:</span>
                                <span class="text-sm text-white">node-1</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Total Nodes:</span>
                                <span class="text-sm text-white">3</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Healthy Nodes:</span>
                                <span class="text-sm text-white">3</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Health Score:</span>
                                <span class="text-sm text-white">91%</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Replication Status -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <h3 class="text-lg font-medium text-white mb-4">Replication Status</h3>
                        <div class="space-y-4">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Average Lag:</span>
                                <span class="text-sm text-white">0.35s</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Max Lag:</span>
                                <span class="text-sm text-white">0.5s</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Sync Replicas:</span>
                                <span class="text-sm text-white">2</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Async Replicas:</span>
                                <span class="text-sm text-white">0</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Replication Mode:</span>
                                <span class="text-sm text-white">Synchronous</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Cluster Actions -->
            <div class="mt-6 bg-gray-800 shadow rounded-lg">
                <div class="p-6">
                    <h3 class="text-lg font-medium text-white mb-4">Cluster Actions</h3>
                    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                        <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Switchover
                        </button>
                        <button class="bg-yellow-600 hover:bg-yellow-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Failover
                        </button>
                        <button class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Add Replica
                        </button>
                        <button class="bg-purple-600 hover:bg-purple-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Expand Cluster
                        </button>
                    </div>
                </div>
            </div>

            <!-- Recent Events -->
            <div class="mt-6 bg-gray-800 shadow rounded-lg">
                <div class="p-6">
                    <h3 class="text-lg font-medium text-white mb-4">Recent Cluster Events</h3>
                    <div class="space-y-3">
                        <div class="flex items-center space-x-3">
                            <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                            <span class="text-sm text-gray-300">Cluster health check completed successfully</span>
                            <span class="text-xs text-gray-500">2 minutes ago</span>
                        </div>
                        <div class="flex items-center space-x-3">
                            <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                            <span class="text-sm text-gray-300">Replication lag decreased to 0.2s on node-2</span>
                            <span class="text-xs text-gray-500">5 minutes ago</span>
                        </div>
                        <div class="flex items-center space-x-3">
                            <div class="w-2 h-2 bg-yellow-500 rounded-full"></div>
                            <span class="text-sm text-gray-300">Node node-3 replication lag increased to 0.5s</span>
                            <span class="text-xs text-gray-500">10 minutes ago</span>
                        </div>
                        <div class="flex items-center space-x-3">
                            <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                            <span class="text-sm text-gray-300">Cluster configuration updated</span>
                            <span class="text-xs text-gray-500">15 minutes ago</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </main>
</body>
</html>
    "#;

    Html(html)
} 