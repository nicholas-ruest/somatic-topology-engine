# Radio calibration profile draft

Calibration binds observations to one compatibility manifest and installation.
The signed profile must contain a profile/version identifier, device and link
identity, board/chipset/firmware/kernel, AP and packet source, center frequency,
bandwidth, antenna/subcarrier counts, geometry measurements, room identifier,
calibration times, operator, environmental covariates, fixture digest, algorithm
version, acceptance statistics, and expiration/requalification triggers.

Calibration cannot authorize sensing, widen purpose, suppress rejected/missing
counts, or convert degraded capture into accepted evidence. Recalibrate after
device/AP movement, antenna/firmware/kernel/channel change, repair, reset,
material interference drift, or profile expiry. Raw calibration captures retain
the same consent, encryption, access, and lifecycle policy as other CSI.

Draft acceptance requires deterministic replay, finite/physically plausible
frames, complete provenance, no unexplained sequence reordering, >=95% accepted
frames, zero hidden critical loss, stable power/thermal state, and a separately
approved scientific calibration gate. Numeric geometry tolerances remain
pending measurement on the reference installation.
