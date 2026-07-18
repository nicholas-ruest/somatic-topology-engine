# Validation study preregistration

Complete and freeze this record before inspecting validation/test outcomes.
Material changes require a new protocol version and supersession record; never
edit a completed result.

## Identity and authority

- Study/protocol version:
- Accountable owner and independent reviewer:
- Synthetic, retrospective, pilot, or human prospective cohort:
- Ethics authority identifier, scope, approval/expiry:
- Participant consent authority/version/purpose/expiry:
- Jurisdiction and prohibited uses:

Human collection cannot start while either authority is absent, expired, or
out of scope. Administrative approval is not a substitute.

## Question and operating envelope

- Exact intended construct or signal-only endpoint:
- Null hypothesis and negative-result interpretation:
- Population/cohort, inclusion/exclusion, rooms, devices, days:
- Hardware/firmware/radio/calibration/reference-sensor profiles:
- Motion, interference, missingness, OOD, and quality exclusions:
- Intended use and excluded claims:

## Dataset and split freeze

- Dataset manifest/card digest and license/consent/retention:
- Participant/session/room/day group keys:
- Development/validation/test assignment manifest digest:
- Leakage checks and expected failure fixture:
- Transform/preprocessing/feature digests:
- Missingness and reference-quality handling:

No participant, session, room, or day group may cross partition roles. Test data
cannot select models, thresholds, calibration, preprocessing, or stopping.

## Analysis

- Primary metric, unit, direction, threshold, and confidence interval method:
- Secondary metrics and multiplicity control:
- Baselines and minimum improvement:
- Agreement/calibration/selective-risk/coverage endpoints:
- Minimum sample size and stopping rule:
- Failure-rate and subgroup analyses:
- Resource and latency limits:

## Promotion decision

- Exact gate expression and required evidence artifacts:
- Promotion scope and expiration:
- Rejection/abstention behavior:
- Independent reviewer and decision authority:
- Negative-result retention location:

Frozen protocol digest:
