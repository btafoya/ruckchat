# 000 - Vision

## Purpose

RuckChat is a self-hostable team chat platform built for operators who want full control over their communication data without operating a distributed systems graduate program.

## Core Beliefs

- **Deployment simplicity wins.** A single binary, one PostgreSQL database, and a reverse proxy should be enough for most teams.
- **Avoid infrastructure tourism.** Redis, Kafka, Elasticsearch, Kubernetes, and microservices are excluded from the v1 design.
- **Batteries included.** Authentication, organizations, channels, direct messages, file uploads, search, notifications, and a plugin SDK are part of the product, not aftermarket add-ons.
- **Own your data.** The server runs where the operator chooses; there is no required SaaS dependency.

## Target Users

- Small-to-medium teams (tens to a few thousand users).
- Teams regulated by data-residency, compliance, or internal IT policies.
- Developers and system administrators who prefer configuration files over vendor consoles.

## Success Criteria

1. A new operator can install and run RuckChat in under ten minutes on commodity hardware.
2. A developer can build and run the full stack locally in under five minutes.
3. End users can send and receive messages, upload files, and search history without training.
4. The server can serve a single-organization deployment from a single process.

## What RuckChat Is Not

- Not a public social network.
- Not a marketplace for third-party integrations (plugins are supported, but the core does not depend on them).
- Not a replacement for specialized compliance archiving or e-discovery suites, although it exports data in standard formats.

## Long-Term Direction

RuckChat v1 focuses on a solid single-server chat experience. Future versions may add federation, advanced compliance features, or managed hosting, but only after the core is stable, tested, and documented.
