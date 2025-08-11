Eres el desarrollador principal de BINLOGICPG-HA y BLCGUI, un sistema en Rust para alta disponibilidad de PostgreSQL que debe superar a Patroni, Stolon y repmgr, con un solo binario por nodo y consumo ultraeficiente.

Reglas:
1. Lenguaje: Rust, sin usar `mod.rs`. Cada módulo en su propio archivo y rutas explícitas.
2. Cada hito del PRD es un commit independiente.
3. Cada commit debe:
   - Compilar sin errores.
   - Incluir tests unitarios y de integración.
   - Actualizar `roadmap.md` marcando el hito como completado.
4. API HTTP: `/status`, `/cluster`, `/metrics`, `/promote`.
5. CLI (`blcpg-cli`): comandos `status`, `promote`, `failover`, `metrics`, `nodes`.
6. Consenso múltiple: Raft (prioritario), Redis, flock.
7. Health checks con scoring y failover automático/manual.
8. Integración Prometheus y soporte HAProxy, Consul y ProxySQL.
9. Documentación completa (`README.md`) y configuración en TOML.
10. Optimización:
    - <5s failover.
    - <50MB RAM.
    - Config simple <20 líneas.
11. Distribución: Binario cross-compilado para Linux usando `cross`.
12. Configuración TOML con secciones: postgresql, api, cluster, raft, locker, logging, redis, health, critical_failover, metrics, vip.

Objetivo inicial: entregar `blcpg-ha` y `blcpg-cli` funcionales para la Fase 1 con Raft como backend de consenso principal.

Formato de configuración TOML requerido:
```toml
[postgresql]
host = "localhost"
port = 5432
user = "postgres"
password = "santus"
database = "postgres"
data_dir = "/var/lib/postgresql/data"
bin_dir = "/usr/bin"

[api]
host = "0.0.0.0"
port = 8009
auth_token = "webgui-secret-token-2025"

[cluster]
enabled = true
node_id = "local-node"
cluster_port = 9601
peers = []
heartbeat_interval = "5s"
timeout = "10s"

[raft]
bind_addr = "127.0.0.1:9701"
peers = []

[locker]
backend = "redis"
ttl = 30

[logging]
level = "debug"
format = "text"
output = "stdout"

[redis]
addrs = ["localhost:6379"]
username = ""
password = ""
db = 0
sentinel = false
master_name = ""
cluster = false
tls_enable = false
timeout = 5

[health]
enabled = true
interval = "2s"
timeout = "5s"
auto_promote = true

[critical_failover]
enabled = true
health_check_interval = "2s"
failover_timeout = "30s"
verification_attempts = 3
min_health_score = 80.0
max_lag_bytes = 104857600
fencing_timeout = "10s"
rollback_timeout = "15s"

[metrics]
enabled = true
prometheus = ":9090"

[vip]
type = "none"
```
13. Estructura de proyecto:
    - Sin `mod.rs`, cada módulo en su propio archivo con rutas explícitas.
    - Todos los comentarios en inglés.
    - Licencia MIT, autor: Santiago Lertora (santiagolertora@gmail.com).

14. Distribución y versionado:
    - Generar binarios para Linux y macOS en cada compilación.
    - Incrementar versión automáticamente en cada build.
    - Binarios de Linux en directorio `release/`.
    - Incluir configuración de testeo en `release/` para reproducibilidad.

15. CRÍTICO - Misión Crítica:
    - NADA hardcodeado. Todo debe ser configurable y funcional.
    - Manejo robusto de errores en todas las operaciones.
    - Timeouts configurables para todas las operaciones de red.
    - Circuit breakers para prevenir cascadas de fallos.
    - Logs estructurados con niveles apropiados.
    - Métricas exhaustivas para monitoreo.
    - Graceful shutdown en todos los servicios.
    - Validación de configuración al inicio.
    - Fallbacks automáticos para servicios críticos.
    - Tests de integración para todos los flujos críticos. 


    no puede haber nada harcodeado!! nada todo debe ser funcional real y de mision critica