PRD.md – BLC PostgreSQL HA
1. Visión y objetivos
El proyecto BLC PostgreSQL HA busca redefinir el estándar de alta disponibilidad para PostgreSQL, ofreciendo una solución que:

Sea radicalmente más simple de instalar, configurar y operar que Patroni, PgAutoFailover o Stolon.

Mantenga latencia mínima en failover (objetivo < 5 segundos de detección, < 20 segundos de takeover).

Garantice 99.999% de disponibilidad en entornos de misión crítica.

Proporcione observabilidad completa y control total desde CLI o GUI.

Sea portable: On-Prem, Cloud, Kubernetes, bare metal.

Innovación: failover predictivo con AI, diagnósticos proactivos, y mecanismos de autocuración.

Filosofía de diseño:
"Zero friction, maximum resilience."
Un administrador debería poder pasar de cero a cluster HA operativo en minutos, con mínima curva de aprendizaje.

2. Componentes principales
2.1. blcpg-ha – Servidor / Agente HA
Corre en cada nodo del cluster.

Funciones:

Monitoreo continuo de PostgreSQL.

Elección de líder y failover automático/manual.

Integración con balanceadores y VIP.

APIs HTTP/gRPC para control y monitoreo.

Modos de coordinación:

flock (simple, sin dependencias externas).

Redis (alta velocidad, distribuido).

Raft embebido (consenso nativo, sin servicios externos).

2.2. blcpg-cli – Cliente CLI
Administración del cluster desde cualquier nodo o workstation.

Funciones:

Estado en tiempo real de nodos y cluster.

Promover/demover nodos.

Consultar métricas.

Ejecutar diagnósticos (doctor, debug).

Interactuar con APIs de balanceadores y VIP.

Ideal para integración con scripts y CI/CD.

2.3. blcpg-gui – Interfaz Web UI
Panel centralizado para monitorear y administrar el cluster.

Funciones:

Visualización en tiempo real del estado de nodos, roles, lag y salud.

Controles manuales de failover y mantenimiento.

Integración con sistemas de alerta (Slack, Webhook, Email).

Historial de eventos y logs.

Visualización de métricas Prometheus con gráficos.

Diseño responsivo y soporte dark/light mode.

3. Arquitectura técnica
Topología mínima: 3 nodos PostgreSQL (primario + réplicas).

Comunicación entre nodos: gRPC (estado, heartbeats, métricas).

APIs externas: HTTP REST para control, Prometheus para métricas.

Backend de coordinación:

flock → Entornos simples, on-prem.

Redis → Multi-nodo con baja latencia.

Raft → Alta resiliencia sin servicios externos.

Seguridad:

TLS opcional entre nodos.

Autenticación por token o mTLS para CLI y GUI.

Logs firmados y auditables.

4. Principios clave de desarrollo
Failover rápido y predecible.

Instalación "one-command" en cualquier entorno.

Logs y métricas exhaustivas por defecto.

Comportamiento seguro ante fallos de red (split brain prevention).

Extensible para balanceadores, clouds y storage.

5. Fases de desarrollo
Fase 0 – PoC y agente mínimo
CLI mínima (run, init).

Healthcheck con pg_isready.

Failover básico con pg_ctl promote.

Configuración en TOML.

Logs básicos.

Fase 1 – Backend flock y healthcheck robusto
TryLock() con flock.

Loop de healthcheck estable.

Cálculo de lag con funciones internas de PostgreSQL.

Endpoint /status.

Métricas Prometheus iniciales.

Fase 2 – Backend Redis
Soporte para Redis Standalone, Sentinel, Cluster.

TTL auto-renew.

Fallback automático a modo mock.

Configuración avanzada en TOML.

Fase 3 – Bus de cluster y elección avanzada
Comunicación gRPC entre nodos.

Intercambio de lag, timeline, rol y salud.

Algoritmo de mejor réplica (min lag + healthy).

Endpoints /peers/status, /cluster/status.

Fase 4 – Integraciones VIP y balanceadores
Balanceadores soportados:

HAProxy (Data Plane API).

ProxySQL API v2.

Keepalived (VIP local).

AWS Elastic IP.

VIP Manager unificado.

Fallback automático entre balanceadores.

Fase 5 – blcpg-gui Core y control manual
Dashboard en tiempo real.

Controles: promote, demote, disable, enable.

APIs REST para control remoto.

Paquete instalable y Helm chart.

Fase 6 – Observabilidad avanzada
Métricas extendidas: failovers, VIP ops, lock owner.

Circuit breaker para flaps de salud.

Notificaciones por Slack/Webhook.

Simulador de caos.

Fase 7 – Documentación y migración
Guía "3 nodos en 10 minutos".

Script de migración desde Patroni.

Casos de uso (on-prem, multi-cloud, K8s).

Troubleshooting guide.

Fase 8 – Funciones enterprise
Autenticación avanzada (RBAC, LDAP, OIDC).

Audit logging detallado.

Multi-tenancy.

Integración con backup y monitoreo externo.

Fase 9 – Innovación y AI
Predicción de fallos con ML/AI.

Failover proactivo.

Ajuste automático de parámetros PostgreSQL.

Panel de recomendaciones automáticas.

6. Criterios de aceptación por fase
Cada fase será aprobada si:

Pasa 100% de tests unitarios e integración.

Cumple objetivos de RTO y RPO definidos.

Documentación y ejemplos listos.

Instalación reproducible en entornos de prueba.

7. Plan de despliegue
Binarios: .deb y .rpm.

Contenedores: Docker + Helm charts.

Automatización: Ansible role oficial.

Instalador interactivo: install.sh.

8. Plan de observabilidad
Exporter Prometheus nativo en blcpg-ha.

Dashboards Grafana listos para importar.

Logging estructurado con rotación.

Alertas configurables.

9. Compatibilidad y extensibilidad
PostgreSQL >= 12.

Linux (Debian, Ubuntu, RHEL, Rocky, Alma).

Cloud: AWS, GCP, Azure.

APIs abiertas para integrar otros balanceadores o VIP managers.