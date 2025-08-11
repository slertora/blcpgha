# Backends de Consenso - BLC PostgreSQL HA

## Resumen Ejecutivo

BLC PostgreSQL HA soporta múltiples backends de consenso para adaptarse a diferentes entornos y necesidades. Cada backend tiene sus ventajas y desventajas, permitiendo elegir la solución más apropiada según el contexto.

## Backends Disponibles

### 1. Raft (Prioritario)

**Descripción**: Implementación del algoritmo de consenso Raft para elección distribuida de líder.

#### ✅ Pros
- **Consenso distribuido robusto**: Garantiza consistencia en entornos distribuidos
- **Tolerancia a fallos**: Funciona correctamente con hasta N/2-1 nodos fallidos
- **Elección de líder automática**: No requiere intervención manual
- **Log de cambios**: Mantiene historial de cambios de configuración
- **Escalabilidad**: Soporta clusters de cualquier tamaño
- **Sin dependencias externas**: Funciona completamente en memoria
- **Recuperación automática**: Se recupera automáticamente de fallos de red

#### ❌ Contras
- **Complejidad**: Algoritmo más complejo de implementar y debuggear
- **Overhead de red**: Requiere comunicación constante entre nodos
- **Latencia**: Puede introducir latencia en la elección de líder
- **Recursos**: Consume más CPU y memoria que alternativas simples
- **Configuración**: Requiere configuración de timeouts y parámetros

#### 🎯 Casos de Uso Ideales
- Clusters de 3+ nodos
- Entornos de alta disponibilidad crítica
- Infraestructuras distribuidas
- Cuando se requiere consistencia fuerte
- Entornos cloud multi-zona

---

### 2. Redis

**Descripción**: Utiliza Redis como backend de coordinación con TTL y auto-renewal.

#### ✅ Pros
- **Simplicidad**: Fácil de implementar y entender
- **Performance**: Muy rápido para operaciones de lock
- **TTL automático**: Evita locks huérfanos
- **Auto-renewal**: Mantiene el liderazgo activamente
- **Fallback**: Puede fallback a flock si Redis no está disponible
- **Monitoreo**: Fácil de monitorear con herramientas existentes
- **Persistencia**: Redis puede persistir el estado si es necesario

#### ❌ Contras
- **Dependencia externa**: Requiere Redis funcionando
- **Single point of failure**: Si Redis falla, el sistema puede quedar sin líder
- **Configuración**: Requiere configurar Redis (standalone/Sentinel/Cluster)
- **Red**: Depende de conectividad de red a Redis
- **Costos**: Agrega infraestructura adicional

#### 🎯 Casos de Uso Ideales
- Entornos con Redis ya desplegado
- Clusters pequeños (2-3 nodos)
- Cuando se requiere velocidad máxima
- Entornos donde Redis es parte de la infraestructura
- Migraciones desde soluciones que ya usan Redis

---

### 3. flock

**Descripción**: Utiliza file locking del sistema operativo para elección de líder.

#### ✅ Pros
- **Simplicidad extrema**: Muy fácil de implementar
- **Sin dependencias**: No requiere servicios externos
- **Performance**: Extremadamente rápido
- **Confiable**: Utiliza mecanismos del kernel
- **Recursos mínimos**: Consume muy pocos recursos
- **Debugging**: Fácil de debuggear y monitorear
- **Ubicuo**: Funciona en cualquier sistema Unix/Linux

#### ❌ Contras
- **Limitado a un nodo**: Solo funciona en un servidor
- **Sin distribución**: No soporta clusters distribuidos
- **Single point of failure**: Si el servidor falla, se pierde el lock
- **Sin persistencia**: El lock se pierde si el proceso termina
- **Escalabilidad limitada**: No escala más allá de un servidor

#### 🎯 Casos de Uso Ideales
- Entornos de desarrollo y testing
- Instalaciones simples de un nodo
- Cuando se requiere simplicidad máxima
- Entornos con recursos muy limitados
- Prototipos y PoCs

## Comparación Técnica

| Aspecto | Raft | Redis | flock |
|---------|------|-------|-------|
| **Complejidad** | Alta | Media | Baja |
| **Performance** | Media | Alta | Muy Alta |
| **Escalabilidad** | Alta | Media | Baja |
| **Tolerancia a fallos** | Alta | Media | Baja |
| **Dependencias** | Ninguna | Redis | Ninguna |
| **Recursos** | Altos | Medios | Mínimos |
| **Configuración** | Compleja | Media | Simple |
| **Debugging** | Complejo | Medio | Simple |

## Estrategia de Implementación

### Fase 1: flock (Semana 2)
- Implementar TryLock() con flock
- Failover automático básico
- Ideal para entornos simples

### Fase 2: Redis (Semana 3)
- Soporte para Redis standalone/Sentinel/Cluster
- TTL auto-renew para liderazgo
- Fallback automático a flock

### Fase 3: Raft (Semana 4-5)
- Comunicación gRPC entre nodos
- Consenso distribuido robusto
- Elección optimizada de líder

## Recomendaciones por Entorno

### 🏢 **Enterprise/Producción**
**Recomendado**: Raft
- Consistencia fuerte requerida
- Alta disponibilidad crítica
- Recursos disponibles

### ☁️ **Cloud/DevOps**
**Recomendado**: Redis o Raft
- Redis si ya está en la infraestructura
- Raft para máxima robustez

### 🏠 **On-Premise Simple**
**Recomendado**: flock o Redis
- flock para instalaciones simples
- Redis para múltiples nodos

### 🧪 **Desarrollo/Testing**
**Recomendado**: flock
- Simplicidad máxima
- Sin dependencias externas

## Configuración

```toml
[locker]
backend = "raft"  # "raft", "redis", "flock"
ttl = 30

[raft]
bind_addr = "127.0.0.1:9701"
node_id = "node-1"
election_timeout_ms = 1000
heartbeat_interval_ms = 100

[redis]
addrs = ["127.0.0.1:6379"]
username = ""
password = ""
db = 0
sentinel = false
master_name = "mymaster"
cluster = false
tls_enable = false
timeout = 5000
```

## Migración entre Backends

BLC PostgreSQL HA permite migrar entre backends sin downtime:

1. **flock → Redis**: Agregar configuración Redis y cambiar backend
2. **Redis → Raft**: Configurar Raft y cambiar backend
3. **Raft → Redis**: Para simplificar en entornos pequeños

## Monitoreo y Observabilidad

### Métricas por Backend

#### Raft
- `raft_is_leader`: Estado de liderazgo
- `raft_current_term`: Término actual
- `raft_commit_index`: Índice de commit
- `raft_last_applied`: Último índice aplicado

#### Redis
- `redis_connection_status`: Estado de conexión
- `redis_lock_ttl`: TTL del lock actual
- `redis_operations_total`: Operaciones totales

#### flock
- `flock_held`: Si el lock está activo
- `flock_operations_total`: Operaciones de lock

## Troubleshooting

### Problemas Comunes

#### Raft
- **Split brain**: Verificar configuración de timeouts
- **Lentitud**: Ajustar heartbeat_interval_ms
- **No elección**: Verificar conectividad entre nodos

#### Redis
- **Lock perdido**: Verificar TTL y auto-renewal
- **Conexión**: Verificar conectividad y configuración
- **Sentinel**: Verificar configuración de Sentinel

#### flock
- **Lock no liberado**: Verificar que el proceso termine correctamente
- **Permisos**: Verificar permisos en el directorio de locks

## Conclusión

La elección del backend de consenso debe basarse en:

1. **Requisitos de disponibilidad**
2. **Complejidad aceptable**
3. **Infraestructura existente**
4. **Recursos disponibles**
5. **Escalabilidad futura**

BLC PostgreSQL HA proporciona flexibilidad para adaptarse a cualquier entorno, desde instalaciones simples hasta clusters enterprise distribuidos. 