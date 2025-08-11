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
    <title>BLC PostgreSQL HA - Settings</title>
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
                            <a href="/metrics" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium">Metrics</a>
                            <a href="/settings" class="bg-gray-900 text-white px-3 py-2 rounded-md text-sm font-medium">Settings</a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </nav>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <div class="px-4 py-6 sm:px-0">
            <h1 class="text-2xl font-bold text-white mb-6">Settings</h1>

            <!-- Settings Tabs -->
            <div class="bg-gray-800 shadow rounded-lg">
                <div class="border-b border-gray-700">
                    <nav class="-mb-px flex space-x-8 px-6">
                        <button class="border-b-2 border-blue-500 py-4 px-1 text-sm font-medium text-white">
                            General
                        </button>
                        <button class="border-b-2 border-transparent py-4 px-1 text-sm font-medium text-gray-400 hover:text-white">
                            Cluster
                        </button>
                        <button class="border-b-2 border-transparent py-4 px-1 text-sm font-medium text-gray-400 hover:text-white">
                            Notifications
                        </button>
                        <button class="border-b-2 border-transparent py-4 px-1 text-sm font-medium text-gray-400 hover:text-white">
                            Security
                        </button>
                    </nav>
                </div>

                <div class="p-6">
                    <!-- General Settings -->
                    <div class="space-y-6">
                        <div>
                            <h3 class="text-lg font-medium text-white mb-4">General Settings</h3>
                            
                            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        GUI Title
                                    </label>
                                    <input type="text" value="BLC PostgreSQL HA" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        Theme
                                    </label>
                                    <select class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                        <option value="dark" selected>Dark</option>
                                        <option value="light">Light</option>
                                        <option value="auto">Auto</option>
                                    </select>
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        Refresh Interval (seconds)
                                    </label>
                                    <input type="number" value="5" min="1" max="60" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        Auto Refresh
                                    </label>
                                    <div class="flex items-center">
                                        <input type="checkbox" checked class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded">
                                        <span class="ml-2 text-sm text-gray-300">Enable auto refresh</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="border-t border-gray-700 pt-6">
                            <h3 class="text-lg font-medium text-white mb-4">API Configuration</h3>
                            
                            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        API Base URL
                                    </label>
                                    <input type="url" value="http://localhost:8080" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        API Timeout (seconds)
                                    </label>
                                    <input type="number" value="30" min="1" max="300" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium text-gray-300 mb-2">
                                        API Retries
                                    </label>
                                    <input type="number" value="3" min="0" max="10" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500">
                                </div>
                            </div>
                        </div>

                        <div class="border-t border-gray-700 pt-6">
                            <div class="flex justify-end space-x-3">
                                <button class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                                    Cancel
                                </button>
                                <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                                    Save Settings
                                </button>
                            </div>
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