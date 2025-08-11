roadmap.md – BLC PostgreSQL HA
Filosofía
Queremos ser el referente mundial en simplicidad, performance e innovación para HA en PostgreSQL.
Este roadmap prioriza:
[ ] Instalación en minutos
[ ] Failover rápido y predecible
[ ] Observabilidad total
[ ] Innovación con AI y autocuración

Fases y entregables
Fase 0 – PoC y agente mínimo (Semana 1)
[x] blcpg-ha inicial con CLI mínima (run, init)
[x] Configuración en config.toml con parámetros básicos
[x] Healthcheck con pg_isready y verificación de conexión
[x] Failover básico con pg_ctl promote
[x] Logging básico en stdout y archivo

Objetivo: Tener un nodo que pueda promoverse manualmente y monitorearse.

Fase 1 – Backend flock y healthcheck robusto (Semana 2)
[ ] Implementar TryLock() con flock para elección de líder
[x] Loop de healthcheck con retries y timeout configurable
[x] Cálculo de replication lag con funciones internas PostgreSQL
[x] Endpoint /status con JSON básico
[x] Exportar métricas iniciales a Prometheus (pg_status, replication_lag)

Objetivo: Failover automático en entornos simples sin dependencias externas.

Fase 2 – Backend Redis (Semana 3)
[x] Configuración avanzada en config.toml
[ ] Soporte para Redis standalone, Sentinel y Cluster
[ ] TTL auto-renew para mantener liderazgo
[ ] Fallback automático a modo flock si Redis no está disponible

Objetivo: Failover distribuido rápido con backend externo.

Fase 3 – Bus de cluster y elección avanzada (Semana 4-5)
[x] Endpoints /peers/status y /cluster/status
[x] Comunicación HTTP REST entre nodos para intercambio de estado
[x] Broadcast de lag, timeline, rol y salud
[x] Algoritmo de mejor réplica (min lag + healthy)

Objetivo: Tener consenso distribuido robusto y elección optimizada de líder. ✅ COMPLETADO

Fase 4 – Integraciones VIP y balanceadores (Semana 6)
[x] Soporte para HAProxy (Data Plane API)
[x] Soporte para ProxySQL API v2
[x] Soporte para Keepalived (VIP local)
[x] Soporte para AWS Elastic IP
[x] VIP Manager unificado con fallback automático
[x] Soporte amazon ELB

Objetivo: Redirigir tráfico de forma automática tras un failover. ✅ COMPLETADO

Fase 5 – blcpg-cli y blcpg-gui Core (Semana 7-8)
[x] APIs REST para control remoto desde GUI
[x] blcpg-cli: Comandos para control manual (promote, demote, status, cluster-info)
[x] blcpg-cli: Comandos inteligentes de promoción/democión con auto-selección de candidatos
[x] blcpg-cli: Análisis de impacto y confirmación del usuario para operaciones críticas
[x] blcpg-cli: Comandos de monitoreo (health, metrics, vip, nodes)
[x] blcpg-cli: Configuración remota y gestión de nodos
[ ] blcpg-gui: Dashboard en tiempo real (nodos, roles, lag, estado)
[ ] blcpg-gui: Controles manuales: promote, demote, disable, enable
[ ] Paquete instalable (.deb, .rpm) y Helm chart inicial

Objetivo: CLI y GUI funcionales para visualizar y controlar el cluster. ✅ MVP COMPLETADO (CLI funcional al 90%)

NOTA: El MVP principal (failover automático) está COMPLETO desde la Fase 3. 
La Fase 5 CLI está funcional al 90% y puede considerarse MVP-complete para operaciones manuales.
Se recomienda avanzar a Fase 10 para funcionalidades avanzadas de gestión de réplicas.

Fase 6 – Observabilidad avanzada (Semana 9)
[x] Métricas extendidas: failovers, VIP ops, lock owner
[ ] Circuit breaker para evitar failover loops
[ ] Notificaciones Slack/Webhook/Email
[ ] Simulador de caos para pruebas de resiliencia

Objetivo: Visibilidad total y pruebas de robustez.

Fase 7 – Documentación y migración (Semana 10)
[ ] Guía "3 nodos en 10 minutos"
[ ] Script de migración desde Patroni
[ ] Casos de uso (on-prem, multi-cloud, Kubernetes)
[ ] Troubleshooting guide

Objetivo: Cualquier DBA pueda instalarlo y migrar en minutos.

Fase 8 – Funciones enterprise (Semana 11-12)
[ ] Autenticación avanzada (RBAC, LDAP, OIDC)
[ ] Audit logging detallado y firmado
[ ] Multi-tenancy
[ ] Integración con sistemas de backup y monitoreo externos

Objetivo: Listo para entornos corporativos exigentes.

Fase 9 – Innovación y AI (Semana 13-14)
[ ] Predicción de fallos con ML/AI
[ ] Failover proactivo en base a métricas históricas
[ ] Ajuste automático de parámetros PostgreSQL (autotuning)
[ ] Panel de recomendaciones automáticas en blcpg-gui

Objetivo: HA que no solo reacciona, sino que se anticipa.

NUEVA FASE 10 – Gestión avanzada de réplicas y switchover (Semana 15-16)
[x] blcpg-cli: Comando `add-replica` inteligente con auto-selección de fuente
  [x] Detección automática de nodos sin PostgreSQL instalado
  [x] Selección inteligente de fuente para pg_basebackup (primario vs réplica)
  [x] Configuración automática de replicación (postgresql.auto.conf, standby.signal)
  [x] Validación de replicación post-instalación
[x] blcpg-cli: Comando `switchover` planificado (más seguro que demote)
  [x] Bloqueo de escrituras antes del switchover
  [x] Verificación de lag = 0 en réplica candidata
  [x] Switchover síncrono con confirmación
  [x] Reconfiguración automática de otras réplicas
[x] blcpg-cli: Comando `failover` automático
  [x] Detección automática de caída del primario
  [x] Auto-selección de mejor candidato (menor lag + mejor health)
  [x] Failover automático con notificaciones
  [x] Rollback automático en caso de fallo
[x] blcpg-cli: Comando `replication-status` detallado
  [x] Estado de replicación por nodo (sync_state, replay_lag, write_lag)
  [x] Información de WAL (pg_last_wal_receive_lsn, pg_last_wal_replay_lsn)
  [x] Alertas de replicación lenta o caída
  [x] Recomendaciones de optimización
[x] blcpg-cli: Comando `cluster-expand` para agregar nodos
  [x] Detección de nodos vacíos en el cluster
  [x] Instalación automática de PostgreSQL
  [x] Configuración automática como réplica
  [x] Integración automática al cluster

Objetivo: Gestión completa de réplicas y operaciones avanzadas de failover/switchover.

NUEVA FASE 11 – Operaciones avanzadas de PostgreSQL (Semana 17-18)
[ ] blcpg-cli: Comando `backup` inteligente
  [ ] Backup desde primario o réplica (auto-selección)
  [ ] Backup incremental y diferencial
  [ ] Compresión y encriptación
  [ ] Integración con sistemas de backup externos
[ ] blcpg-cli: Comando `restore` inteligente
  [ ] Restore a punto en el tiempo específico
  [ ] Restore a réplica específica
  [ ] Validación post-restore
  [ ] Reintegración automática al cluster
[ ] blcpg-cli: Comando `maintenance` para operaciones de mantenimiento
  [ ] Modo mantenimiento (read-only)
  [ ] VACUUM y ANALYZE automático
  [ ] Reindex automático
  [ ] Limpieza de logs y archivos temporales
[ ] blcpg-cli: Comando `upgrade` para actualizaciones de PostgreSQL
  [ ] Detección de versiones compatibles
  [ ] Plan de actualización automático
  [ ] Rollback automático en caso de fallo
  [ ] Actualización rolling sin downtime

Objetivo: Operaciones avanzadas de PostgreSQL automatizadas y seguras.

NUEVA FASE 12 – Enterprise Cloud Services (Semana 19-22)
[ ] Soporte completo para Kubernetes
  [ ] Helm charts para despliegue en K8s
    [ ] Chart para blcpg-ha (StatefulSet con PersistentVolumes)
    [ ] Chart para blcpg-cli (Job/CronJob para operaciones)
    [ ] Chart para blcpg-gui (Deployment con Ingress)
    [ ] Chart para PostgreSQL (StatefulSet con replicación)
  [ ] Operadores de Kubernetes (K8s Operators)
    [ ] Custom Resource Definitions (CRDs) para clusters PostgreSQL
    [ ] Controller para gestión automática de clusters
    [ ] Webhook para validación de recursos
    [ ] Admission controller para políticas de seguridad
  [ ] Integración con servicios K8s nativos
    [ ] Service mesh (Istio/Linkerd) para comunicación entre nodos
    [ ] Ingress controllers para acceso externo
    [ ] Cert-manager para certificados TLS automáticos
    [ ] External-dns para DNS automático
[ ] Contenedores Docker enterprise
  [ ] Multi-stage builds optimizados
  [ ] Imágenes multi-arch (amd64, arm64)
  [ ] Security scanning y vulnerabilidades
  [ ] Base images minimalistas (distroless/alpine)
  [ ] Health checks y readiness probes
  [ ] Configuración via environment variables
  [ ] Secrets management (Docker secrets, K8s secrets)
[ ] API REST enterprise para servicios cloud
  [ ] OpenAPI/Swagger documentation completa
  [ ] Rate limiting y throttling
  [ ] API versioning (v1, v2, etc.)
  [ ] Pagination para listas grandes
  [ ] Filtering y sorting avanzado
  [ ] Bulk operations (create/update/delete múltiple)
  [ ] Webhooks para eventos (cluster state changes)
[ ] Multi-tenancy y isolation
  [ ] Namespace isolation en K8s
  [ ] Resource quotas y limits
  [ ] Network policies para isolation
  [ ] Storage classes por tenant
  [ ] RBAC granular por tenant
  [ ] Audit logging por tenant
[ ] Auto-scaling y auto-healing
  [ ] Horizontal Pod Autoscaler (HPA) para réplicas
  [ ] Vertical Pod Autoscaler (VPA) para recursos
  [ ] Cluster autoscaler para nodos
  [ ] Auto-healing de pods fallidos
  [ ] Auto-recovery de clusters corruptos
  [ ] Predictive scaling basado en métricas
[ ] Observabilidad enterprise
  [ ] Integración con Prometheus/Grafana
  [ ] Distributed tracing (Jaeger/Zipkin)
  [ ] Centralized logging (ELK stack/Fluentd)
  [ ] Alerting y notification (AlertManager)
  [ ] Custom dashboards por tenant
  [ ] SLA monitoring y reporting
[ ] Security y compliance
  [ ] Encryption at rest (LUKS, KMS)
  [ ] Encryption in transit (TLS 1.3)
  [ ] Network policies y firewalls
  [ ] Pod security policies
  [ ] Compliance reporting (SOC2, GDPR, HIPAA)
  [ ] Vulnerability scanning automático
  [ ] Secret rotation automático
[ ] Backup y disaster recovery
  [ ] Backup automático a S3/GCS/Azure
  [ ] Point-in-time recovery (PITR)
  [ ] Cross-region replication
  [ ] Disaster recovery automation
  [ ] Backup encryption y compression
  [ ] Retention policies automáticas
[ ] Billing y metering
  [ ] Usage tracking por tenant
  [ ] Resource consumption metrics
  [ ] Billing integration (Stripe, AWS Marketplace)
  [ ] Cost optimization recommendations
  [ ] Quota management y enforcement
  [ ] Usage alerts y notifications
[ ] Self-service portal
  [ ] Web UI para gestión de clusters
  [ ] Cluster provisioning wizard
  [ ] Monitoring dashboards
  [ ] Backup management
  [ ] User management y roles
  [ ] API key management
[ ] Integración con cloud providers
  [ ] AWS RDS-compatible API
  [ ] Azure Database-compatible API
  [ ] GCP Cloud SQL-compatible API
  [ ] Terraform providers
  [ ] CloudFormation templates
  [ ] ARM templates (Azure)
[ ] Performance y optimización
  [ ] Connection pooling (PgBouncer)
  [ ] Query optimization y tuning
  [ ] Resource optimization automático
  [ ] Performance benchmarking
  [ ] Capacity planning tools
  [ ] Performance alerts y recommendations

Objetivo: Producto enterprise listo para servicios cloud PostgreSQL (estilo Aurora, RDS, Cloud SQL).

Hitos clave
✅ Semana 1: PoC operativo
✅ Semana 4: Failover distribuido robusto
✅ Semana 6: Integraciones VIP completas
✅ Semana 8: CLI inteligente con promoción/democión avanzada

Semana 10: GUI funcional y cluster gestionable
Semana 12: Migraciones desde Patroni
Semana 14: Funciones AI activas
Semana 16: Gestión avanzada de réplicas
Semana 18: Operaciones avanzadas de PostgreSQL
Semana 22: Enterprise cloud services completos

Desarrollo de Componentes
blcpg-ha: Fases 0-4 (Core del sistema) ✅ COMPLETADO
blcpg-cli: Fases 5-6, 10-11 (Herramienta de línea de comandos) ⏳ EN PROGRESO
blcpg-gui: Fases 5-9, 12 (Interfaz web) [ ] PENDIENTE
blcpg-operator: Fase 12 (Kubernetes operator) [ ] PENDIENTE
blcpg-cloud: Fase 12 (Cloud services) [ ] PENDIENTE

Tiempos totales
Duración estimada: 22 semanas (extendido de 18 semanas)
Estrategia: Desarrollo paralelo de blcpg-ha y blcpg-cli desde el inicio, blcpg-gui a partir de la fase 5, enterprise features en fase 12.

Estado actual (Agosto 2024)
✅ Fase 0: Completada - PoC operativo
✅ Fase 1: Completada - Healthcheck robusto y métricas
✅ Fase 2: Completada - Configuración avanzada
✅ Fase 3: Completada - Comunicación HTTP REST entre nodos (MVP PRINCIPAL COMPLETO)
✅ Fase 4: Completada - Integraciones VIP y balanceadores
⏳ Fase 5: En progreso - CLI inteligente (90% completado, pendiente configuración remota y GUI)
[ ] Fase 6: Pendiente - Observabilidad avanzada
[ ] Fase 7: Pendiente - Documentación y migración
[ ] Fase 8: Pendiente - Funciones enterprise
[ ] Fase 9: Pendiente - Innovación y AI
[ ] Fase 10: Pendiente - Gestión avanzada de réplicas (PRÓXIMA PRIORIDAD)
[ ] Fase 11: Pendiente - Operaciones avanzadas de PostgreSQL (SEGUNDA PRIORIDAD)
[ ] Fase 12: Pendiente - Enterprise cloud services

Progreso: 5/12 fases completadas (42%)
MVP Status: ✅ COMPLETO (Fases 0-3 + CLI funcional en Fase 5)
Próximo objetivo: Comenzar Fase 10 (Gestión avanzada de réplicas) - RECOMENDADO

## 🎯 ESTRATEGIA DE DESARROLLO ACTUALIZADA

### 📋 ORDEN DE PRIORIDADES:
1. **🆕 Fase 10** - Gestión avanzada de réplicas (PRÓXIMA)
   - `add-replica` - Agregar réplicas automáticamente
   - `switchover` - Switchover planificado más seguro
   - `failover` - Failover automático con notificaciones
   - `replication-status` - Estado detallado de replicación
   - `cluster-expand` - Expandir cluster automáticamente

2. **🆕 Fase 11** - Operaciones avanzadas de PostgreSQL (SEGUNDA)
   - `backup` - Backup inteligente
   - `restore` - Restore inteligente
   - `maintenance` - Operaciones de mantenimiento
   - `upgrade` - Actualizaciones de PostgreSQL

3. **⏳ Fase 5** - Completar CLI y GUI (TERCERA)
   - Configuración remota y gestión de nodos
   - Dashboard en tiempo real (GUI)
   - Controles manuales en GUI
   - Paquete instalable (.deb, .rpm) y Helm chart inicial

### 🎯 RACIONAL:
- **Fase 10-11** son funcionalidades core que completan el producto
- **Fase 5** es principalmente GUI y empaquetado (menos crítico)
- **MVP ya está completo** - podemos enfocarnos en funcionalidades avanzadas

