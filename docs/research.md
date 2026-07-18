# Somatic Topology Engine: Feasibility and Research Synthesis

**Status:** Concept-stage research report
**Date:** 2026-07-17
**Project brief:** [`.plans/description.md`](../.plans/description.md)

## Executive summary

The Somatic Topology Engine (STE) is technically plausible as a local, wearable-free system for detecting occupancy, gross motion, respiration candidates, and—with substantially tighter acquisition conditions—cardiac-related periodicity. The current evidence does **not** support the stronger claim that commodity Wi-Fi CSI alone can directly or uniquely infer cognitive load, emotional valence, or decision-making phase. Those states are latent constructs with overlapping physiological correlates, strong person and context dependence, and time scales that conflict with the proposed 500 ms label cadence.

The defensible path is to build STE as a personalized, uncertainty-aware psychophysiological research instrument. It should first validate its RF-derived observables against reference sensors, then validate carefully operationalized state estimates against synchronized task labels and self-report. Until those studies succeed, UI outputs should say what the system observed or estimated—such as `respiration rising`, `motion contamination`, or `possible elevated arousal`—rather than asserting `deep focus` or `decision threshold`.

## Research question and goal state

**Question:** Can the named ruvnet components and CrowPi/Raspberry Pi hardware support a scientifically defensible, entirely local prototype that estimates personalized cognitive-affective state from ambient Wi-Fi CSI?

The research goal is complete when the project has:

1. a verified hardware and software acquisition path;
2. a distinction between directly sensed variables, derived physiology, and inferred latent state;
3. an evidence-graded feasibility assessment;
4. an architecture aligned with actual component capabilities;
5. a validation protocol with falsifiable success criteria;
6. privacy, safety, and human-subject safeguards; and
7. an implementation sequence that prevents unsupported claims from entering the UI.

## GOAP research plan

**Current state:** The repository contains a one-line README, the project brief, and agent tooling, but no STE source, tests, datasets, model card, or hardware captures. The external components exist, but their readiness and fit vary.

**Estimated plan cost:** Medium for a replay-based demonstrator; high for a valid single-person physiological prototype; very high for generalizable cognitive-affective inference.

| Step | Preconditions | Effect | Cost |
|---|---|---|---|
| Verify claims and interfaces | Project brief and upstream sources available | Capability matrix grounded in current sources | Low |
| Prove CSI acquisition | Supported Pi, firmware, AP, packet stream | Reproducible raw CSI capture and replay | Medium |
| Validate observables | CSI plus synchronized reference sensors | Error bounds for motion, respiration, and cardiac estimates | High |
| Define latent-state labels | Ethics review, task protocol, self-report instruments | Trainable and falsifiable targets | High |
| Train personalized models | Clean labeled sessions across days | Calibrated per-user estimates | High |
| Test generalization | Held-out sessions, rooms, positions, and people | Honest operating envelope | Very high |
| Integrate edge/UI pipeline | Valid model, latency and memory budgets | Local demonstrator with uncertainty gates | Medium |

**Replanning triggers:** Pi CSI capture proves unstable; cardiac estimates fail reference agreement; motion dominates the signal; labels are unreliable; held-out-session performance approaches baseline; 500 ms end-to-end latency is incompatible with valid feature windows; or upstream APIs do not match the brief.

**Fallback:** Ship a replayable ambient physiology and focus-session assistant that reports presence, motion quality, respiration trend, user anchors, and conservative personalized pattern similarity. Treat cognitive labels as user-authored annotations, not sensor ground truth.

## Methodology

This synthesis checked:

- the project brief and current repository contents;
- current upstream READMEs for RuView, rvCSI, ruv-FANN, RuVector, MidStream, DSPy.ts, AgentDB, and Ruflo;
- the Nexmon CSI extraction project and literature using Raspberry Pi/Nexmon;
- peer-reviewed or primary research on Wi-Fi CSI respiration, vital signs, activity recognition, dataset leakage, physiological emotion recognition, cognitive workload, and ultra-short HRV; and
- privacy implications of passive RF sensing.

Evidence grades used here are:

- **High:** directly documented in current source code/project documentation and/or supported by multiple independent studies;
- **Medium:** credible single-source evidence or a plausible integration not yet demonstrated for STE;
- **Low:** extrapolation beyond demonstrated inputs, environment, labels, or validation conditions.

Upstream documentation is evidence of software capability, not independent evidence that a physiological or psychological claim is valid.

## Evidence map

### 1. Wi-Fi CSI acquisition on Raspberry Pi 4 is feasible, but operationally fragile — Evidence: High

rvCSI documents Nexmon ingestion for the BCM43455c0 used with Raspberry Pi 4/5, validation of malformed frames, normalized `CsiFrame`/`CsiWindow`/`CsiEvent` types, DSP, deterministic replay, and a TypeScript boundary. Nexmon CSI independently supports CSI extraction on selected Broadcom chips. Published Raspberry Pi 4 experiments also use Nexmon for respiration sensing.

This is not a normal Wi-Fi application interface. It requires compatible firmware/kernel combinations, monitor-mode capture, a transmitter or controlled traffic source, channel configuration, and repeatable placement. The brief's phrase “captured by the Crow Pi's onboard WiFi” omits these dependencies. The Wi-Fi NIC is the receiver; a useful CSI link also needs a transmitter/AP and sufficient packet rate.

**Implication:** freeze a known-good OS/kernel/firmware image and prove capture before building any inference or UI layer.

### 2. Presence, movement, and respiration are realistic first targets — Evidence: High

Wi-Fi CSI human sensing has strong literature for presence and activity. Respiration is physically coupled to periodic chest displacement, and systems such as FarSense demonstrate CSI-based respiration under controlled arrangements. rvCSI already exposes motion energy, presence scoring, signal-quality events, and a heuristic breathing-band candidate.

Real-world robustness remains a concern. A 550-hour study found Wi-Fi vital sensing plausible but emphasized that many techniques degrade outside experimental conditions. Range, multipath geometry, subject orientation, other people, fans, pets, AP traffic, and furniture changes can all alter CSI.

**Implication:** quality gating and “unable to estimate” states are core product behavior, not optional polish.

### 3. Cardiac periodicity may be detectable, but “cardiac coherence” is not yet an established input — Evidence: Medium/Low

RuView documents a vital-sign pipeline and claims heart-rate support, while broader RF literature contains contactless cardiac sensing results. Cardiac chest displacement is much smaller than respiratory motion, however, and the rvCSI runtime itself currently promises only a heuristic breathing candidate, not validated beat-to-beat intervals.

“Cardiac coherence” is also underspecified. If it means HRV or respiratory sinus arrhythmia, it requires accurate inter-beat intervals and feature windows much longer than 500 ms. Reviews of ultra-short HRV find that some time-domain measures may be usable in tens of seconds under specific conditions, while frequency-domain measures generally require longer windows and careful validation.

**Implication:** validate beat detection and interval error against ECG/PPG before calculating HRV-like features. Do not use a 500 ms window for them; a 500 ms UI refresh may display a slowly updated estimate with its window age.

### 4. Cognitive load is correlated with physiology, not uniquely encoded by it — Evidence: Medium

Heart rate, HRV, respiration, electrodermal activity, pupil response, and performance measures have all been studied as workload correlates. NASA's workload literature notes that heart rate may rise and HRV may fall with load, while also warning that heart rate alone is neither highly sensitive nor diagnostically specific. Multimodal workload studies normally combine several physiological channels and use task performance and NASA-TLX-like self-report as labels.

The same autonomic changes can arise from physical movement, caffeine, temperature, social stress, fatigue, illness, or excitement. CSI adds an extra confound because body motion changes the RF signal directly as well as changing physiology.

**Implication:** define workload as a task-specific estimate, train within person, include motion and environmental covariates, and report calibration and uncertainty. Avoid the term “cognitive load” for unvalidated unsupervised clusters.

### 5. Valence cannot be reliably derived from respiration and heart signals alone at present — Evidence: Low

Physiological emotion-recognition research supports autonomic signals as correlates of affect, especially arousal. It also documents large inter-subject variance, inconsistent elicitation and annotation, dataset limitations, and a substantial gap between subject-dependent laboratory accuracy and subject-independent real-world performance. Reviews explicitly conclude that realistic subject-independent recognition remains unresolved.

Valence is particularly difficult because positive excitement and negative stress can share elevated arousal. The brief supplies no facial, vocal, electrodermal, contextual, or self-report channel that could disambiguate them.

**Implication:** estimate arousal-like change before valence. Valence should remain user-supplied through anchors or brief self-report unless a dedicated validation study demonstrates otherwise.

### 6. “Decision-making phase” is currently an ungrounded construct — Evidence: Low

No reviewed component or cited CSI literature directly measures a decision threshold. A model might learn temporal patterns aligned with a tightly designed decision task, but the target must be operationalized—for example stimulus onset, evidence accumulation period, response time, confidence, and post-decision interval. Calling a generic physiological transition a decision threshold would overinterpret the data.

**Implication:** treat decision phase as a separate research experiment, not an MVP output. Compare against trivial baselines such as elapsed time since prompt and recent motor preparation.

### 7. Personalization and replay are more defensible than universal classification — Evidence: High

CSI models commonly suffer from domain shift across rooms, positions, people, and hardware. A critical re-evaluation of Wi-Fi action-recognition methods found substantial performance drops when leakage was removed through proper subject-based partitioning. rvCSI's deterministic replay and versioned calibration, plus RuVector/AgentDB similarity and feedback mechanisms, are useful for personalized baselines and longitudinal drift.

Memory does not automatically improve measurement accuracy. Feedback must be grounded in a valid target, protected against accidental or biased reinforcement, and evaluated on future held-out sessions.

**Implication:** store provenance with every anchor: user, session, room, link geometry, firmware, calibration version, feature/model version, task, label source, and confidence. Never train on test-session feedback.

## Component fit and integration corrections

| Component | Defensible STE role | Readiness/constraint |
|---|---|---|
| **rvCSI** | Pi/Nexmon ingest, validation, DSP, quality events, replay, TS bridge | Strongest direct fit. It explicitly emits evidence rather than medical decisions. Live ESP32 support is a follow-up, but Pi/Nexmon is documented. |
| **RuView** | Reference architecture and optional vital-sign modules | Relevant, but many README claims are project claims rather than independent validation. Prefer its tested signal crates over duplicating them. |
| **ruv-FANN** | Small local classifier/regressor on ARM | Plausible. The project is a general Rust neural-network library; STE must supply trained weights, preprocessing parity, calibration, and Pi benchmarks. |
| **MidStream** | Incremental event/window processing or change-point logic after an adapter | The upstream project primarily analyzes streaming AI output. “Streaming cognitive-state inference” is not an off-the-shelf documented capability. |
| **DSPy.ts** | Optimize a constrained text rendering/program against an explicit metric | It cannot establish physiological truth. A fixed template is safer and faster for 500 ms status text; prompt optimization belongs outside the real-time safety path. |
| **RuVector** | Local embeddings, similarity, temporal memory, drift experiments | Strong capability, but rvCSI says its production RuVector binding was still a follow-up; verify exact integration versions. |
| **AgentDB** | Persistent personalized episodes/anchors and explicit feedback | Available offline and on edge runtimes. “Gets more accurate” requires valid rewards and prospective evaluation. Avoid running two overlapping memory stores without a clear boundary. |
| **Ruflo** | Development-time orchestration, calibration workflows, experiment management | A full agent swarm is unnecessary in the hard real-time loop. Keep deterministic sensing and inference supervised by a small local service. |
| **CrowPi peripherals** | Touch-to-anchor, OLED status, RGB quality/arousal display, DHT11 covariate | Useful interaction layer. Verify the exact CrowPi revision, pin mapping, OLED interface, and touch hardware before implementation. |

### DHT11 correction claim

Ambient humidity and temperature can be logged as nuisance covariates because environment changes can affect propagation and comfort. The brief's phrase “controlling inference temperature with ambient humidity correction” has no defined physical or statistical model. A DHT11 should not directly alter neural-network softmax temperature without empirical calibration; doing so can make confidence less calibrated.

## Recommended architecture

```text
Wi-Fi AP / traffic source
          |
Pi 4 Broadcom NIC + pinned Nexmon image
          |
rvCSI validation -> signal-quality and motion gates -> versioned window store
          |                                              |
          |                                     deterministic replay
          v
observable estimators
(presence, motion, respiration; cardiac only after validation)
          |
personalized temporal model + calibrated abstention
          |
state estimate API -------------------------> anchor/event memory
          |                                      (AgentDB or RuVector)
          v
deterministic UI renderer
(OLED label + confidence/quality; RGB; touch anchor)
```

Keep three schemas separate:

1. **Observation:** CSI quality, amplitude/phase features, motion, periodicity, environment.
2. **Physiology estimate:** respiration rate, beat candidate, confidence, window length, reference-validation status.
3. **Latent-state estimate:** target definition, probability, calibration, model scope, abstention reason, and provenance.

This separation makes it difficult for an unvalidated feature to be silently presented as a mental fact.

## Validation program

### Phase 0 — Acquisition and replay

- Pin the exact CrowPi/Pi model, OS, kernel, Nexmon firmware, AP, band, channel, bandwidth, packet source, and antenna geometry.
- Capture empty-room, stationary-person, breathing, gross-motion, and interference sessions.
- Require reproducible `.pcap` to `.rvcsi` conversion and replay-identical features.
- Record packet rate, rejected/degraded frames, missingness, CPU, memory, temperature, and latency.

**Gate:** at least 95% valid windows in the intended stationary operating geometry, with explicit recovery after link or calibration failure. This is an engineering target, not a scientific standard.

### Phase 1 — Observable validity

- Synchronize CSI with a respiratory belt and ECG or validated PPG; use timestamps and a documented alignment error.
- Evaluate respiration and heart estimates separately during stillness, posture changes, speech, typing, and deliberate motion.
- Report MAE, bias and limits of agreement, coverage/abstention, and failure rate—not correlation alone.
- Hold out entire sessions and days.

**Gate:** pre-register acceptable errors for the intended non-medical use. If cardiac interval accuracy fails, remove HRV/coherence features rather than substituting a noisy proxy.

### Phase 2 — Personalized workload study

- Obtain ethics/IRB-equivalent review as applicable and informed consent.
- Use repeated, counterbalanced workload tasks with task performance, response time, and NASA-TLX or an appropriate validated instrument.
- Capture sleep/fatigue, caffeine, posture, speech, activity, room, and environmental covariates.
- Train only on prior sessions; test chronologically on unseen days.
- Compare against majority, elapsed-time, task-condition, respiration-only, heart-only, and motion-only baselines.

**Gate:** pre-registered improvement over baselines with calibrated probabilities and useful coverage on held-out days. Report per-user results and confidence intervals.

### Phase 3 — Affect and decision experiments

- Treat arousal, valence, and decision phase as distinct targets with distinct protocols.
- Use repeated in-the-moment self-report for affect; do not infer valence from task category alone.
- For decisions, timestamp stimuli, evidence, responses, and confidence; test whether physiology adds value beyond task timing and movement.
- Do not pool windows from the same session across train and test splits.

**Gate:** no product-style label until replicated across sessions and compared with strong nuisance-variable baselines.

## Real-time behavior and latency

The UI may refresh every 500 ms, but the underlying estimators operate on different horizons:

| Output | Likely evidence horizon | Recommended update behavior |
|---|---:|---|
| Signal quality / gross motion | sub-second to seconds | 2 Hz is plausible |
| Respiration phase/rate | multiple respiratory cycles | update display at 2 Hz but expose window age and stability |
| Heart rate | multiple beats | update only when confidence and stillness gates pass |
| HRV-like features | tens of seconds to minutes, metric-dependent | slow update; never imply 500 ms resolution |
| Cognitive-affective estimate | protocol- and model-dependent | smooth, debounce, abstain, and disclose evidence horizon |

Natural-language generation every 500 ms adds latency, nondeterminism, flicker, and hallucination risk. Use enumerated labels and templates in the live loop. DSPy.ts can optimize wording offline against human-rated clarity without changing the underlying state.

## Privacy, safety, and governance

“No camera” does not mean “no privacy risk.” Device-free RF sensing can reveal occupancy, movement, routines, physiology, and potentially identity without requiring a person to carry a device. STE should therefore provide:

- opt-in consent for every person in the sensing area and a visible sensing indicator;
- local-only raw data by default, encryption at rest, retention limits, and a physical delete control;
- no identity inference, clinical diagnosis, employment scoring, deception detection, or covert use;
- signed model/calibration provenance and an audit log of anchors and feedback;
- an always-available off state and clear RF sensing boundaries;
- user access to, correction of, and deletion of their data; and
- language stating that outputs are experimental estimates, not readings of thoughts or medical advice.

## Contradictions resolved

1. **“Commodity onboard Wi-Fi is sufficient” vs specialized CSI extraction.** Resolved in favor of the narrower claim: the Pi 4 chipset is supported through Nexmon, but only with patched firmware, compatible software, configured traffic, and a transmitter.
2. **“Cognitive state every 500 ms” vs physiological feature duration.** Resolved by separating display cadence from evidence-window duration. A 2 Hz UI cannot create 2 Hz physiological validity.
3. **“Cardiac coherence” vs current rvCSI output.** Unresolved scientifically and unsupported by rvCSI's documented event set. It remains a research target behind reference-sensor validation.
4. **“Valence from autonomic signals” vs non-specific arousal.** Resolved by removing valence from the MVP and using user annotation unless prospective validation supports it.
5. **“Memory gets more accurate over time” vs feedback contamination.** Resolved conditionally: memory enables personalization, but improvement must be measured on future held-out sessions using valid rewards.
6. **“MidStream performs cognitive-state inference” vs its documented purpose.** Resolved as an integration adaptation, not a native feature.

## Recommended delivery sequence

1. **Rename the first milestone:** “Ambient Somatic Signal Explorer,” not BCI or thought reader.
2. **Build the capture spine:** pinned Nexmon image, rvCSI capture/validation, replay fixtures, and quality dashboard.
3. **Implement peripheral I/O:** touch anchors, OLED, RGB, and DHT logging behind hardware interfaces with a simulator.
4. **Validate respiration first:** it offers the strongest physiology-to-CSI path and exposes motion/geometry failures early.
5. **Attempt cardiac validation:** proceed to HRV-like features only if inter-beat timing agrees with reference data.
6. **Choose one memory layer:** start with a simple append-only, versioned event store; add AgentDB/RuVector retrieval only when a concrete query and evaluation exist.
7. **Run the personalized workload protocol:** preregister splits and metrics; publish negative results and abstention coverage.
8. **Add conservative state rendering:** templates, evidence horizon, confidence calibration, and explicit unknown/contaminated states.
9. **Investigate affect and decision phase separately:** neither should block the useful somatic-sensing prototype.

## Decision

**Proceed, with a narrowed claim and stage gates.** The project has a credible hardware/software path for ambient RF sensing and a potentially valuable personalized biofeedback interface. It is not presently justified as a BCI, an exocortex that reads mental state, or a system that has achieved real-time cognitive/valence/decision inference. The scientifically strong version of STE is compelling precisely because it exposes uncertainty, validates each inference layer, and allows failure to falsify the ambitious claims.

## Limitations

- No physical CrowPi/Pi, CSI capture, or upstream repository checkout was available in this workspace for execution testing.
- Upstream projects are fast-moving; interfaces and documented capabilities should be pinned to commit hashes during implementation.
- This was a targeted feasibility synthesis, not a formal systematic review or meta-analysis.
- Product-specific CrowPi revision and peripheral pin/interface details remain to be verified from the purchased unit's documentation.
- Regulations and human-subject requirements depend on jurisdiction and deployment context; obtain qualified review before collecting participant data.

## References

### Project and component sources

- [STE project brief](../.plans/description.md)
- [RuView repository and hardware guidance](https://github.com/ruvnet/RuView)
- [RuView vital-sign pipeline decision record](https://github.com/ruvnet/RuView/blob/main/docs/adr/ADR-021-vital-sign-detection-rvdna-pipeline.md)
- [rvCSI repository](https://github.com/ruvnet/rvcsi)
- [Nexmon CSI repository](https://github.com/seemoo-lab/nexmon_csi)
- [ruv-FANN repository](https://github.com/ruvnet/ruv-FANN)
- [RuVector repository](https://github.com/ruvnet/RuVector)
- [MidStream repository](https://github.com/ruvnet/midstream)
- [DSPy.ts repository](https://github.com/ruvnet/dspy.ts)
- [AgentDB repository](https://github.com/ruvnet/agentdb)
- [Ruflo repository](https://github.com/ruvnet/ruflo)

### Scientific evidence

- Zeng et al., [FarSense: Pushing the Range Limit of WiFi-Based Respiration Sensing](https://arxiv.org/abs/1907.03994)
- Hillyard et al., [On the Goodness of WiFi Based Monitoring of Vital Signs in the Wild](https://arxiv.org/abs/2003.09386)
- Yang et al., [Self-Supervised Learning for WiFi CSI-Based Human Activity Recognition: A Systematic Study](https://arxiv.org/abs/2308.02412)
- Hernandez and Bulut, [Exposing Data Leakage in Wi-Fi CSI-Based Human Action Recognition](https://doi.org/10.3390/inventions9040090)
- Barradas et al., [Emotion Recognition from Peripheral Physiological Signals: A Systematic Review](https://doi.org/10.1145/3771719)
- Shu et al., [A Review of Emotion Recognition Using Physiological Signals](https://pmc.ncbi.nlm.nih.gov/articles/PMC6069143/)
- Prabhakar et al., [Measuring Cognitive Workload Using Multimodal Sensors](https://arxiv.org/abs/2205.04235)
- NASA, [Workload Transition: Implications for Individual and Team Performance](https://ntrs.nasa.gov/citations/20030086428)
- Pecchia et al., [Are Ultra-Short Heart Rate Variability Features Good Surrogates of Short-Term Ones?](https://pmc.ncbi.nlm.nih.gov/articles/PMC5998753/)
- Shaffer et al., [A Critical Review of Ultra-Short-Term Heart Rate Variability Norms Research](https://doi.org/10.3389/fnins.2020.594880)
- Domingues et al., [IEEE 802.11 CSI Randomization to Preserve Location Privacy](https://doi.org/10.1016/j.comnet.2021.108257)
