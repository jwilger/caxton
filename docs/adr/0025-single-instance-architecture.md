## Status

Accepted

## Context

The original Caxton design assumed distributed multi-instance deployment
capabilities, with clustering coordination, distributed state management,
and complex orchestration between multiple server instances.

However, analysis of production requirements and operational
complexity
revealed:

- No demonstrated need for distributed deployment in target use cases
- Significant architectural complexity without proven benefits
- Single-instance servers successfully handle many production workloads
- Administrative console as optional feature creates
  unnecessary
  configuration overhead
- Development and testing complexity increases dramatically with
  distributed
  systems
- Operational burden of cluster management outweighs benefits for
  target
  scale

Many successful production systems operate effectively as
single-instance
servers (PostgreSQL primary, Redis standalone, many application
servers)
and only add distribution when scale demands it.

## Decision

Caxton will operate as a single-instance server architecture with
the
following characteristics:

1. **Single Server Instance**: Caxton runs as one server process
   per
   deployment
2. **Integrated Admin Console**: Administrative interface is
   built-in, not
   optional
3. **No Clustering**: Multi-instance clustering is not a
   current
   architectural goal
4. **Vertical Scaling Focus**: Scale up resources on single
   instance rather
   than horizontal distribution
5. **Future Migration Path**: Architecture allows future
   multi-instance
   capability if requirements emerge

## Consequences

**Positive:**

- Dramatically simplified architecture and implementation
- Reduced operational complexity and deployment requirements
- Faster path to production readiness and feature delivery
- No distributed state management or consensus protocols needed
- No cluster coordination overhead or split-brain scenarios
- Easier debugging, monitoring, and troubleshooting
- Lower resource requirements for small to medium deployments
- Standard single-process deployment patterns familiar to operations teams

**Negative:**

- Single point of failure for the service (mitigated by standard HA practices)
- Vertical scaling limits based on hardware constraints
- Cannot distribute load across multiple instances natively
- May require future architectural changes if horizontal scaling becomes necessary

**Neutral:**

- Standard availability practices still apply (process monitoring,
  health checks, restarts)
- Load balancing can still be achieved through multiple independent
  deployments
- Backup and recovery patterns remain straightforward

## Implementation Impact

- Removes distributed coordination logic from codebase
- Simplifies state management to single-process patterns
- Admin console becomes integral component, not optional flag
- Configuration complexity reduced significantly
- Testing focuses on single-instance behavior patterns

## Supersedes

- ADR-0006: Application Server Architecture
- ADR-0014: Coordination-First Architecture
- ADR-0015: Distributed Protocol Architecture

## Related Decisions

- ADR-0022: Web-based Admin Console (admin console integral to server)
- ADR-0014: Coordination-First Architecture (simplified to single-instance coordination)
