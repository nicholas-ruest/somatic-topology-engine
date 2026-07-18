# Phase 18 production-readiness decision

Decision: **NOT APPROVED FOR PRODUCTION OR COMMERCIAL SALE**.

Engineering implementation and current-host tests do not establish commercial viability or production readiness. Mandatory missing evidence includes exact Raspberry Pi/CrowPi HIL, power/thermal and multi-day soak; complete fuzz and penetration campaigns; authorized human validation for proposed scientific claims; physical site qualification; jurisdiction-specific legal/regulatory/privacy approval; commercial pilot; manufacturing/supplier controls; warranty/RMA and staffed support evidence; incident/recall exercises; post-market/CAPA operation; and an exact signed ARM release review.

All unsupported capabilities and claims remain disabled. The readiness decision engine treats missing evidence as blocking and cannot be overridden by feature flags, configuration, documentation language, or schedule. Run `scripts/validate-phase18-readiness.sh` to verify documentation, automated controls, and machine-readable blocked status.
