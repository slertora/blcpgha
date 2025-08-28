// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use axum::response::Html;

pub async fn cluster_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cluster - BLC PostgreSQL HA</title>
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
                    <a href="/cluster" class="text-blue-200 font-semibold">Cluster</a>
                    <a href="/metrics" class="hover:text-blue-200">Metrics</a>
                    <a href="/settings" class="hover:text-blue-200">Settings</a>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <div class="container mx-auto p-6">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-3xl font-bold text-gray-800">Cluster Information</h2>
                <button id="refreshBtn" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                    Refresh
                </button>
            </div>

            <!-- Loading State -->
            <div id="loadingState" class="text-center py-12">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                <p class="mt-4 text-gray-600">Loading cluster information...</p>
            </div>

            <!-- Error State -->
            <div id="errorState" class="hidden bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
                <p id="errorMessage"></p>
            </div>

            <!-- Cluster Content -->
            <div id="clusterContent" class="hidden">
                <!-- Cluster Status -->
                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Cluster Status</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-blue-600" id="clusterHealth">--</div>
                            <div class="text-sm text-gray-600">Health Score</div>
                        </div>
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-green-600" id="activeNodes">--</div>
                            <div class="text-sm text-gray-600">Active Nodes</div>
                        </div>
                        <div class="text-center p-4 bg-gray-50 rounded-lg">
                            <div class="text-2xl font-bold text-purple-600" id="totalNodes">--</div>
                            <div class="text-sm text-gray-600">Total Nodes</div>
                        </div>
                    </div>
                </div>

                <!-- Replication Status -->
                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Replication Status</h3>
                    <div id="replicationStatus" class="space-y-3">
                        <!-- Replication status will be dynamically inserted here -->
                    </div>
                </div>

                <!-- Cluster Actions -->
                <div class="bg-white rounded-lg shadow-md p-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">Cluster Actions</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <button onclick="window.location.href='/nodes'" class="bg-blue-600 text-white p-4 rounded-lg hover:bg-blue-700 text-center">
                            <div class="text-lg font-semibold">Manage Nodes</div>
                            <div class="text-sm opacity-90">Add, remove, and configure nodes</div>
                        </button>
                        <button onclick="showSwitchoverModal()" class="bg-yellow-600 text-white p-4 rounded-lg hover:bg-yellow-700 text-center">
                            <div class="text-lg font-semibold">Switchover</div>
                            <div class="text-sm opacity-90">Planned primary change</div>
                        </button>
                        <button onclick="showFailoverModal()" class="bg-red-600 text-white p-4 rounded-lg hover:bg-red-700 text-center">
                            <div class="text-lg font-semibold">Failover</div>
                            <div class="text-sm opacity-90">Emergency primary change</div>
                        </button>
                        <button onclick="showAddReplicaModal()" class="bg-green-600 text-white p-4 rounded-lg hover:bg-green-700 text-center">
                            <div class="text-lg font-semibold">Add Replica</div>
                            <div class="text-sm opacity-90">Expand cluster</div>
                        </button>
                        <button onclick="showClusterExpandModal()" class="bg-purple-600 text-white p-4 rounded-lg hover:bg-purple-700 text-center">
                            <div class="text-lg font-semibold">Cluster Expand</div>
                            <div class="text-sm opacity-90">Add multiple nodes</div>
                        </button>
                        <button onclick="showBackupModal()" class="bg-indigo-600 text-white p-4 rounded-lg hover:bg-indigo-700 text-center">
                            <div class="text-lg font-semibold">Backup</div>
                            <div class="text-sm opacity-90">Create cluster backup</div>
                        </button>
                        <button onclick="showRestoreModal()" class="bg-orange-600 text-white p-4 rounded-lg hover:bg-orange-700 text-center">
                            <div class="text-lg font-semibold">Restore</div>
                            <div class="text-sm opacity-90">Restore from backup</div>
                        </button>
                        <button onclick="showMaintenanceModal()" class="bg-teal-600 text-white p-4 rounded-lg hover:bg-teal-700 text-center">
                            <div class="text-lg font-semibold">Maintenance</div>
                            <div class="text-sm opacity-90">Database maintenance</div>
                        </button>
                        <button onclick="showUpgradeModal()" class="bg-pink-600 text-white p-4 rounded-lg hover:bg-pink-700 text-center">
                            <div class="text-lg font-semibold">Upgrade</div>
                            <div class="text-sm opacity-90">PostgreSQL upgrade</div>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Switchover Modal -->
    <div id="switchoverModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-md w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Switchover Planificado</h3>
                    <button onclick="hideSwitchoverModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <p class="text-gray-600 mb-4">Selecciona el nodo que será el nuevo primario:</p>
                    <div id="switchoverCandidates" class="space-y-2 mb-4">
                        <!-- Candidates will be loaded here -->
                    </div>
                    <div class="flex justify-end space-x-3">
                        <button onclick="hideSwitchoverModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button id="executeSwitchoverBtn" onclick="executeSwitchover()" class="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700" disabled>Ejecutar Switchover</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Failover Modal -->
    <div id="failoverModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-md w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Failover de Emergencia</h3>
                    <button onclick="hideFailoverModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
                        <div class="flex">
                            <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                            </svg>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">¡ADVERTENCIA!</h3>
                                <p class="text-sm text-red-700 mt-1">El failover automático se ejecutará inmediatamente. Esta operación puede causar pérdida de datos si no hay replicación síncrona.</p>
                            </div>
                        </div>
                    </div>
                    <p class="text-gray-600 mb-4">¿Estás seguro de que quieres ejecutar un failover de emergencia?</p>
                    <div class="flex justify-end space-x-3">
                        <button onclick="hideFailoverModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeFailover()" class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700">Ejecutar Failover</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Add Replica Modal -->
    <div id="addReplicaModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Agregar Réplica</h3>
                    <button onclick="hideAddReplicaModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Host del nuevo nodo:</label>
                            <input type="text" id="newReplicaHost" placeholder="192.168.1.100" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Puerto PostgreSQL:</label>
                            <input type="number" id="newReplicaPort" value="5432" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Usuario SSH:</label>
                            <input type="text" id="newReplicaSshUser" placeholder="postgres" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Ruta SSH key:</label>
                            <input type="text" id="newReplicaSshKey" placeholder="/home/postgres/.ssh/id_rsa" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                    </div>
                    <div class="flex justify-end space-x-3 mt-6">
                        <button onclick="hideAddReplicaModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeAddReplica()" class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">Agregar Réplica</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Cluster Expand Modal -->
    <div id="clusterExpandModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-2xl w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Expandir Cluster</h3>
                    <button onclick="hideClusterExpandModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="mb-4">
                        <label class="block text-sm font-medium text-gray-700 mb-2">Nodos a agregar (uno por línea):</label>
                        <textarea id="clusterExpandNodes" rows="4" placeholder="192.168.1.100:5432:postgres:/home/postgres/.ssh/id_rsa&#10;192.168.1.101:5432:postgres:/home/postgres/.ssh/id_rsa&#10;192.168.1.102:5432:postgres:/home/postgres/.ssh/id_rsa" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"></textarea>
                        <p class="text-xs text-gray-500 mt-1">Formato: host:puerto:usuario:ruta_ssh_key</p>
                    </div>
                    <div class="flex justify-end space-x-3">
                        <button onclick="hideClusterExpandModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeClusterExpand()" class="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700">Expandir Cluster</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Backup Modal -->
    <div id="backupModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Crear Backup del Cluster</h3>
                    <button onclick="hideBackupModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Tipo de backup:</label>
                            <select id="backupType" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="full">Backup completo</option>
                                <option value="incremental">Backup incremental</option>
                                <option value="differential">Backup diferencial</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Compresión:</label>
                            <select id="backupCompression" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="gzip">Gzip</option>
                                <option value="bzip2">Bzip2</option>
                                <option value="none">Sin compresión</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Ruta de destino:</label>
                            <input type="text" id="backupPath" placeholder="/backups/cluster_$(date +%Y%m%d_%H%M%S)" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Retención (días):</label>
                            <input type="number" id="backupRetention" value="30" min="1" max="365" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                    </div>
                    <div class="flex justify-end space-x-3 mt-6">
                        <button onclick="hideBackupModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeBackup()" class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700">Crear Backup</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Restore Modal -->
    <div id="restoreModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Restaurar desde Backup</h3>
                    <button onclick="hideRestoreModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Archivo de backup:</label>
                            <input type="text" id="restoreFile" placeholder="/backups/cluster_20240812_143000.sql" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Tipo de restore:</label>
                            <select id="restoreType" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="full">Restore completo</option>
                                <option value="point-in-time">Restore a punto en el tiempo</option>
                                <option value="schema-only">Solo esquema</option>
                            </select>
                        </div>
                        <div id="pointInTimeDiv" class="hidden">
                            <label class="block text-sm font-medium text-gray-700 mb-2">Punto en el tiempo:</label>
                            <input type="datetime-local" id="restoreTimestamp" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Nodo destino:</label>
                            <select id="restoreTarget" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="primary">Nodo primario</option>
                                <option value="replica">Nodo réplica</option>
                            </select>
                        </div>
                        <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                            <div class="flex">
                                <svg class="w-5 h-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                                </svg>
                                <div class="ml-3">
                                    <h3 class="text-sm font-medium text-yellow-800">¡ADVERTENCIA!</h3>
                                    <p class="text-sm text-yellow-700 mt-1">El restore sobrescribirá completamente la base de datos. Asegúrate de tener un backup reciente.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="flex justify-end space-x-3 mt-6">
                        <button onclick="hideRestoreModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeRestore()" class="px-4 py-2 bg-orange-600 text-white rounded hover:bg-orange-700">Ejecutar Restore</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Maintenance Modal -->
    <div id="maintenanceModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Mantenimiento de Base de Datos</h3>
                    <button onclick="hideMaintenanceModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Tipo de mantenimiento:</label>
                            <select id="maintenanceType" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="vacuum">VACUUM</option>
                                <option value="analyze">ANALYZE</option>
                                <option value="reindex">REINDEX</option>
                                <option value="checkpoint">CHECKPOINT</option>
                                <option value="full">Mantenimiento completo</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Nivel de mantenimiento:</label>
                            <select id="maintenanceLevel" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="light">Ligero (rápido)</option>
                                <option value="full">Completo (estándar)</option>
                                <option value="aggressive">Agresivo (exhaustivo)</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Nodo objetivo:</label>
                            <select id="maintenanceTarget" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="all">Todos los nodos</option>
                                <option value="primary">Solo primario</option>
                                <option value="replicas">Solo réplicas</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Programar para:</label>
                            <select id="maintenanceSchedule" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="now">Ejecutar ahora</option>
                                <option value="maintenance-window">Ventana de mantenimiento</option>
                                <option value="low-traffic">Bajo tráfico</option>
                            </select>
                        </div>
                    </div>
                    <div class="flex justify-end space-x-3 mt-6">
                        <button onclick="hideMaintenanceModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeMaintenance()" class="px-4 py-2 bg-teal-600 text-white rounded hover:bg-teal-700">Ejecutar Mantenimiento</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Upgrade Modal -->
    <div id="upgradeModal" class="fixed inset-0 bg-gray-600 bg-opacity-50 hidden z-50">
        <div class="flex items-center justify-center min-h-screen p-4">
            <div class="bg-white rounded-lg shadow-xl max-w-lg w-full">
                <div class="flex justify-between items-center p-6 border-b">
                    <h3 class="text-lg font-semibold text-gray-800">Actualizar PostgreSQL</h3>
                    <button onclick="hideUpgradeModal()" class="text-gray-400 hover:text-gray-600">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Versión actual:</label>
                            <input type="text" id="currentVersion" value="PostgreSQL 15.4" readonly class="w-full px-3 py-2 bg-gray-100 border border-gray-300 rounded-md">
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Versión objetivo:</label>
                            <select id="targetVersion" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="16.2">PostgreSQL 16.2</option>
                                <option value="17.1">PostgreSQL 17.1</option>
                                <option value="18.0">PostgreSQL 18.0</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Estrategia de actualización:</label>
                            <select id="upgradeStrategy" class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500">
                                <option value="rolling">Rolling upgrade (sin downtime)</option>
                                <option value="all-at-once">Todos a la vez (con downtime)</option>
                                <option value="blue-green">Blue-green deployment</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Validación post-upgrade:</label>
                            <div class="space-y-2">
                                <label class="flex items-center">
                                    <input type="checkbox" id="validateConnections" checked class="mr-2">
                                    <span class="text-sm">Validar conexiones</span>
                                </label>
                                <label class="flex items-center">
                                    <input type="checkbox" id="validateReplication" checked class="mr-2">
                                    <span class="text-sm">Validar replicación</span>
                                </label>
                                <label class="flex items-center">
                                    <input type="checkbox" id="validatePerformance" class="mr-2">
                                    <span class="text-sm">Validar performance</span>
                                </label>
                            </div>
                        </div>
                        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                            <div class="flex">
                                <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                                </svg>
                                <div class="ml-3">
                                    <h3 class="text-sm font-medium text-red-800">¡OPERACIÓN CRÍTICA!</h3>
                                    <p class="text-sm text-red-700 mt-1">La actualización de PostgreSQL es una operación crítica. Asegúrate de tener backups completos y un plan de rollback.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="flex justify-end space-x-3 mt-6">
                        <button onclick="hideUpgradeModal()" class="px-4 py-2 text-gray-600 border border-gray-300 rounded hover:bg-gray-50">Cancelar</button>
                        <button onclick="executeUpgrade()" class="px-4 py-2 bg-pink-600 text-white rounded hover:bg-pink-700">Iniciar Actualización</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Success/Error Toast -->
    <div id="toast" class="fixed top-4 right-4 z-50 hidden">
        <div class="bg-white border rounded-lg shadow-lg p-4 max-w-sm">
            <div class="flex items-center">
                <div id="toastIcon" class="flex-shrink-0 mr-3"></div>
                <div>
                    <p id="toastMessage" class="text-sm font-medium"></p>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Global state
        let clusterData = null;
        let refreshInterval;
        let selectedSwitchoverCandidate = null;

        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {
            loadClusterInfo();
            setupEventListeners();
            startAutoRefresh();
        });

        function setupEventListeners() {
            document.getElementById('refreshBtn').addEventListener('click', loadClusterInfo);
        }

        function startAutoRefresh() {
            // Refresh every 30 seconds
            refreshInterval = setInterval(loadClusterInfo, 30000);
        }

        function stopAutoRefresh() {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        }

        async function loadClusterInfo() {
            showLoading();
            hideError();

            try {
                const response = await fetch('/api/v1/cluster/status');
                const result = await response.json();

                if (result.success && result.data) {
                    clusterData = result.data;
                    renderClusterInfo();
                    showClusterContent();
                } else {
                    showError(result.error || 'Failed to load cluster information');
                }
            } catch (error) {
                showError('Network error: ' + error.message);
            }
        }

        function renderClusterInfo() {
            if (!clusterData) return;

            // Update cluster status
            updateClusterStatus();
            
            // Update replication status
            updateReplicationStatus();
        }

        function updateClusterStatus() {
            // Cluster Health
            const healthElement = document.getElementById('clusterHealth');
            if (clusterData.health_score !== undefined) {
                healthElement.textContent = `${clusterData.health_score}%`;
                healthElement.className = `text-2xl font-bold ${getHealthScoreClass(clusterData.health_score)}`;
            }

            // Active Nodes
            const activeNodesElement = document.getElementById('activeNodes');
            if (clusterData.healthy_nodes !== undefined) {
                activeNodesElement.textContent = clusterData.healthy_nodes;
            }

            // Total Nodes
            const totalNodesElement = document.getElementById('totalNodes');
            if (clusterData.total_nodes !== undefined) {
                totalNodesElement.textContent = clusterData.total_nodes;
            }
        }

        function updateReplicationStatus() {
            const replicationStatus = document.getElementById('replicationStatus');
            
            if (!clusterData.nodes || clusterData.nodes.length === 0) {
                // Si no hay nodos específicos, mostrar información general del cluster
                replicationStatus.innerHTML = `
                    <div class="space-y-3">
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Cluster Name</span>
                            <span class="text-sm text-gray-600">${clusterData.cluster_name || 'Unknown'}</span>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Leader</span>
                            <span class="text-sm text-gray-600">${clusterData.leader || 'Unknown'}</span>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Health Score</span>
                            <span class="text-sm font-medium ${getHealthScoreClass(clusterData.health_score || 0)}">${clusterData.health_score || 0}%</span>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Healthy Nodes</span>
                            <span class="text-sm text-gray-600">${clusterData.healthy_nodes || 0}/${clusterData.total_nodes || 0}</span>
                        </div>
                        <div class="flex justify-between items-center p-3 bg-gray-50 rounded-lg">
                            <span class="font-medium text-gray-800">Last Updated</span>
                            <span class="text-sm text-gray-600">${formatTimestamp(clusterData.last_updated)}</span>
                        </div>
                    </div>
                `;
                return;
            }

            replicationStatus.innerHTML = clusterData.nodes.map(node => `
                <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                    <div class="flex items-center space-x-3">
                        <span class="font-medium text-gray-800">${node.node_id}</span>
                        ${node.is_primary ? '<span class="px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded-full">Primary</span>' : '<span class="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">Replica</span>'}
                    </div>
                    <div class="text-right">
                        <div class="text-sm font-medium text-gray-800">${node.status}</div>
                        ${node.replication_lag_seconds ? `<div class="text-xs text-gray-500">Lag: ${node.replication_lag_seconds}s</div>` : ''}
                    </div>
                </div>
            `).join('');
        }

        function getHealthScoreClass(score) {
            if (score >= 90) return 'text-green-600';
            if (score >= 70) return 'text-yellow-600';
            return 'text-red-600';
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

        // Modal functions
        function showSwitchoverModal() {
            loadSwitchoverCandidates();
            document.getElementById('switchoverModal').classList.remove('hidden');
        }

        function hideSwitchoverModal() {
            document.getElementById('switchoverModal').classList.add('hidden');
            selectedSwitchoverCandidate = null;
            document.getElementById('executeSwitchoverBtn').disabled = true;
        }

        function showFailoverModal() {
            document.getElementById('failoverModal').classList.remove('hidden');
        }

        function hideFailoverModal() {
            document.getElementById('failoverModal').classList.add('hidden');
        }

        function showAddReplicaModal() {
            document.getElementById('addReplicaModal').classList.remove('hidden');
        }

        function hideAddReplicaModal() {
            document.getElementById('addReplicaModal').classList.add('hidden');
        }

        function showClusterExpandModal() {
            document.getElementById('clusterExpandModal').classList.remove('hidden');
        }

        function hideClusterExpandModal() {
            document.getElementById('clusterExpandModal').classList.add('hidden');
        }

        function showBackupModal() {
            document.getElementById('backupModal').classList.remove('hidden');
        }

        function hideBackupModal() {
            document.getElementById('backupModal').classList.add('hidden');
        }

        function showRestoreModal() {
            document.getElementById('restoreModal').classList.remove('hidden');
        }

        function hideRestoreModal() {
            document.getElementById('restoreModal').classList.add('hidden');
        }

        function showMaintenanceModal() {
            document.getElementById('maintenanceModal').classList.remove('hidden');
        }

        function hideMaintenanceModal() {
            document.getElementById('maintenanceModal').classList.add('hidden');
        }

        function showUpgradeModal() {
            document.getElementById('upgradeModal').classList.remove('hidden');
        }

        function hideUpgradeModal() {
            document.getElementById('upgradeModal').classList.add('hidden');
        }

        // Load switchover candidates
        async function loadSwitchoverCandidates() {
            try {
                const response = await fetch('/api/v1/nodes');
                const result = await response.json();
                
                if (result.success && result.data) {
                    const candidates = result.data.filter(node => !node.is_primary && node.status === 'Online');
                    renderSwitchoverCandidates(candidates);
                }
            } catch (error) {
                console.error('Error loading candidates:', error);
            }
        }

        function renderSwitchoverCandidates(candidates) {
            const container = document.getElementById('switchoverCandidates');
            
            if (candidates.length === 0) {
                container.innerHTML = '<p class="text-gray-500 text-center">No hay candidatos disponibles para switchover</p>';
                return;
            }

            container.innerHTML = candidates.map(node => `
                <label class="flex items-center p-3 border border-gray-200 rounded-lg cursor-pointer hover:bg-gray-50">
                    <input type="radio" name="switchoverCandidate" value="${node.node_id}" onchange="selectSwitchoverCandidate('${node.node_id}')" class="mr-3">
                    <div>
                        <div class="font-medium">${node.node_id}</div>
                        <div class="text-sm text-gray-600">Health: ${node.health_score}% | Lag: ${node.replication_lag_seconds || 0}s</div>
                    </div>
                </label>
            `).join('');
        }

        function selectSwitchoverCandidate(nodeId) {
            selectedSwitchoverCandidate = nodeId;
            document.getElementById('executeSwitchoverBtn').disabled = false;
        }

        // Execute functions
        async function executeSwitchover() {
            if (!selectedSwitchoverCandidate) {
                showToast('Por favor selecciona un candidato', 'error');
                return;
            }

            try {
                const response = await fetch(`/api/v1/nodes/${selectedSwitchoverCandidate}/promote`, {
                    method: 'POST'
                });
                
                if (response.ok) {
                    showToast('Switchover ejecutado exitosamente', 'success');
                    hideSwitchoverModal();
                    loadClusterInfo();
                } else {
                    const error = await response.json();
                    showToast('Error en switchover: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        async function executeFailover() {
            try {
                const response = await fetch('/api/v1/cluster/failover', {
                    method: 'POST'
                });
                
                if (response.ok) {
                    showToast('Failover ejecutado exitosamente', 'success');
                    hideFailoverModal();
                    loadClusterInfo();
                } else {
                    const error = await response.json();
                    showToast('Error en failover: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        async function executeAddReplica() {
            const host = document.getElementById('newReplicaHost').value;
            const port = document.getElementById('newReplicaPort').value;
            const sshUser = document.getElementById('newReplicaSshUser').value;
            const sshKey = document.getElementById('newReplicaSshKey').value;

            if (!host || !port || !sshUser || !sshKey) {
                showToast('Por favor completa todos los campos', 'error');
                return;
            }

            try {
                const response = await fetch('/api/v1/cluster/add-replica', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        host,
                        port: parseInt(port),
                        ssh_user: sshUser,
                        ssh_key_path: sshKey
                    })
                });
                
                if (response.ok) {
                    showToast('Réplica agregada exitosamente', 'success');
                    hideAddReplicaModal();
                    loadClusterInfo();
                } else {
                    const error = await response.json();
                    showToast('Error al agregar réplica: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        async function executeClusterExpand() {
            const nodesText = document.getElementById('clusterExpandNodes').value;
            
            if (!nodesText.trim()) {
                showToast('Por favor ingresa al menos un nodo', 'error');
                return;
            }

            const nodes = nodesText.split('\n').filter(line => line.trim()).map(line => {
                const [host, port, user, sshKey] = line.split(':');
                return { host, port: parseInt(port), ssh_user: user, ssh_key_path: sshKey };
            });

            try {
                const response = await fetch('/api/v1/cluster/expand', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ nodes })
                });
                
                if (response.ok) {
                    showToast('Cluster expandido exitosamente', 'success');
                    hideClusterExpandModal();
                    loadClusterInfo();
                } else {
                    const error = await response.json();
                    showToast('Error al expandir cluster: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        async function executeBackup() {
            const type = document.getElementById('backupType').value;
            const compression = document.getElementById('backupCompression').value;
            const path = document.getElementById('backupPath').value;
            const retention = document.getElementById('backupRetention').value;

            try {
                const response = await fetch('/api/v1/cluster/backup', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        backup_type: type,
                        compression,
                        destination_path: path,
                        retention_days: parseInt(retention)
                    })
                });
                
                if (response.ok) {
                    showToast('Backup iniciado exitosamente', 'success');
                    hideBackupModal();
                } else {
                    const error = await response.json();
                    showToast('Error al crear backup: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        // Restore functions
        async function executeRestore() {
            const file = document.getElementById('restoreFile').value;
            const type = document.getElementById('restoreType').value;
            const target = document.getElementById('restoreTarget').value;
            const timestamp = document.getElementById('restoreTimestamp').value;

            if (!file) {
                showToast('Por favor especifica el archivo de backup', 'error');
                return;
            }

            try {
                const response = await fetch('/api/v1/cluster/restore', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        backup_file: file,
                        restore_type: type,
                        target_node: target,
                        point_in_time: timestamp || null
                    })
                });
                
                if (response.ok) {
                    showToast('Restore iniciado exitosamente', 'success');
                    hideRestoreModal();
                } else {
                    const error = await response.json();
                    showToast('Error al ejecutar restore: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        // Maintenance functions
        async function executeMaintenance() {
            const type = document.getElementById('maintenanceType').value;
            const level = document.getElementById('maintenanceLevel').value;
            const target = document.getElementById('maintenanceTarget').value;
            const schedule = document.getElementById('maintenanceSchedule').value;

            try {
                const response = await fetch('/api/v1/cluster/maintenance', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        maintenance_type: type,
                        maintenance_level: level,
                        target_nodes: target,
                        schedule: schedule
                    })
                });
                
                if (response.ok) {
                    showToast('Mantenimiento iniciado exitosamente', 'success');
                    hideMaintenanceModal();
                } else {
                    const error = await response.json();
                    showToast('Error al ejecutar mantenimiento: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        // Upgrade functions
        async function executeUpgrade() {
            const targetVersion = document.getElementById('targetVersion').value;
            const strategy = document.getElementById('upgradeStrategy').value;
            const validateConnections = document.getElementById('validateConnections').checked;
            const validateReplication = document.getElementById('validateReplication').checked;
            const validatePerformance = document.getElementById('validatePerformance').checked;

            try {
                const response = await fetch('/api/v1/cluster/upgrade', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        target_version: targetVersion,
                        upgrade_strategy: strategy,
                        validation: {
                            connections: validateConnections,
                            replication: validateReplication,
                            performance: validatePerformance
                        }
                    })
                });
                
                if (response.ok) {
                    showToast('Actualización iniciada exitosamente', 'success');
                    hideUpgradeModal();
                } else {
                    const error = await response.json();
                    showToast('Error al iniciar actualización: ' + (error.error || 'Unknown error'), 'error');
                }
            } catch (error) {
                showToast('Error de red: ' + error.message, 'error');
            }
        }

        // Show/hide point-in-time div based on restore type
        document.addEventListener('DOMContentLoaded', function() {
            const restoreTypeSelect = document.getElementById('restoreType');
            const pointInTimeDiv = document.getElementById('pointInTimeDiv');
            
            if (restoreTypeSelect && pointInTimeDiv) {
                restoreTypeSelect.addEventListener('change', function() {
                    if (this.value === 'point-in-time') {
                        pointInTimeDiv.classList.remove('hidden');
                    } else {
                        pointInTimeDiv.classList.add('hidden');
                    }
                });
            }
        });

        // Toast notification
        function showToast(message, type = 'info') {
            const toast = document.getElementById('toast');
            const toastMessage = document.getElementById('toastMessage');
            const toastIcon = document.getElementById('toastIcon');

            toastMessage.textContent = message;

            if (type === 'success') {
                toastIcon.innerHTML = '<svg class="w-5 h-5 text-green-400" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path></svg>';
                toast.classList.add('border-green-200');
            } else if (type === 'error') {
                toastIcon.innerHTML = '<svg class="w-5 h-5 text-red-400" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path></svg>';
                toast.classList.add('border-red-200');
            } else {
                toastIcon.innerHTML = '<svg class="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path></svg>';
                toast.classList.add('border-blue-200');
            }

            toast.classList.remove('hidden');
            setTimeout(() => {
                toast.classList.add('hidden');
            }, 5000);
        }

        function showLoading() {
            document.getElementById('loadingState').classList.remove('hidden');
            document.getElementById('clusterContent').classList.add('hidden');
            document.getElementById('errorState').classList.add('hidden');
        }

        function showClusterContent() {
            document.getElementById('loadingState').classList.add('hidden');
            document.getElementById('clusterContent').classList.remove('hidden');
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