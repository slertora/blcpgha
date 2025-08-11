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
    <title>BLC PostgreSQL HA - Metrics</title>
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
                            <a href="/cluster" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Cluster</a>
                            <a href="/metrics" class="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium">Metrics</a>
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
            <h1 class="text-2xl font-bold text-white mb-6">Metrics</h1>

            <!-- Metrics Overview -->
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                <!-- Performance Metrics -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <h3 class="text-lg font-medium text-white mb-4">Performance</h3>
                        <div class="space-y-4">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Queries/sec:</span>
                                <span class="text-sm text-white">1,234</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Transactions/sec:</span>
                                <span class="text-sm text-white">567</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Active Connections:</span>
                                <span class="text-sm text-white">45</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Max Connections:</span>
                                <span class="text-sm text-white">100</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- System Metrics -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <h3 class="text-lg font-medium text-white mb-4">System</h3>
                        <div class="space-y-4">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">CPU Usage:</span>
                                <span class="text-sm text-white">45%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Memory Usage:</span>
                                <span class="text-sm text-white">67%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Disk Usage:</span>
                                <span class="text-sm text-white">23%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Network I/O:</span>
                                <span class="text-sm text-white">1.2 MB/s</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Replication Metrics -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <h3 class="text-lg font-medium text-white mb-4">Replication</h3>
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
                                <span class="text-sm text-gray-400">WAL Rate:</span>
                                <span class="text-sm text-white">2.1 MB/s</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Sync Status:</span>
                                <span class="text-sm text-white">Healthy</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Charts Placeholder -->
            <div class="mt-6 bg-gray-800 shadow rounded-lg">
                <div class="p-6">
                    <h3 class="text-lg font-medium text-white mb-4">Performance Charts</h3>
                    <div class="h-64 bg-gray-700 rounded-lg flex items-center justify-center">
                        <span class="text-gray-400">Charts will be displayed here</span>
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