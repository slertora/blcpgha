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
    <title>BLC PostgreSQL HA - Nodes</title>
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
                            <a href="/nodes" class="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium">Nodes</a>
                            <a href="/cluster" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Cluster</a>
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
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-white">Nodes</h1>
                <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                    Add Node
                </button>
            </div>

            <!-- Nodes Grid -->
            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
                <!-- Node 1 (Primary) -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h3 class="text-lg font-medium text-white">node-1</h3>
                                <p class="text-sm text-gray-400">Primary Node</p>
                            </div>
                            <div class="flex items-center">
                                <div class="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                                <span class="text-sm text-gray-300">Online</span>
                            </div>
                        </div>
                        
                        <div class="space-y-3">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Health Score:</span>
                                <span class="text-sm text-white">95%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Connections:</span>
                                <span class="text-sm text-white">12/100</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">CPU Usage:</span>
                                <span class="text-sm text-white">45%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Memory Usage:</span>
                                <span class="text-sm text-white">67%</span>
                            </div>
                        </div>

                        <div class="mt-6 flex space-x-2">
                            <button class="flex-1 bg-yellow-600 hover:bg-yellow-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Demote
                            </button>
                            <button class="flex-1 bg-red-600 hover:bg-red-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Disable
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Node 2 (Replica) -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h3 class="text-lg font-medium text-white">node-2</h3>
                                <p class="text-sm text-gray-400">Replica Node</p>
                            </div>
                            <div class="flex items-center">
                                <div class="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                                <span class="text-sm text-gray-300">Online</span>
                            </div>
                        </div>
                        
                        <div class="space-y-3">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Health Score:</span>
                                <span class="text-sm text-white">90%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Replication Lag:</span>
                                <span class="text-sm text-white">0.2s</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">CPU Usage:</span>
                                <span class="text-sm text-white">38%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Memory Usage:</span>
                                <span class="text-sm text-white">72%</span>
                            </div>
                        </div>

                        <div class="mt-6 flex space-x-2">
                            <button class="flex-1 bg-green-600 hover:bg-green-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Promote
                            </button>
                            <button class="flex-1 bg-red-600 hover:bg-red-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Disable
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Node 3 (Replica) -->
                <div class="bg-gray-800 shadow rounded-lg">
                    <div class="p-6">
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h3 class="text-lg font-medium text-white">node-3</h3>
                                <p class="text-sm text-gray-400">Replica Node</p>
                            </div>
                            <div class="flex items-center">
                                <div class="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                                <span class="text-sm text-gray-300">Online</span>
                            </div>
                        </div>
                        
                        <div class="space-y-3">
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Health Score:</span>
                                <span class="text-sm text-white">88%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Replication Lag:</span>
                                <span class="text-sm text-white">0.5s</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">CPU Usage:</span>
                                <span class="text-sm text-white">42%</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-gray-400">Memory Usage:</span>
                                <span class="text-sm text-white">65%</span>
                            </div>
                        </div>

                        <div class="mt-6 flex space-x-2">
                            <button class="flex-1 bg-green-600 hover:bg-green-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Promote
                            </button>
                            <button class="flex-1 bg-red-600 hover:bg-red-700 text-white px-3 py-2 rounded-md text-sm font-medium">
                                Disable
                            </button>
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