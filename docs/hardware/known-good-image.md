# Known-good radio image procedure

There is currently no qualified production image; this document is the build
and acceptance procedure, not an image claim.

1. Start from the approved 64-bit Raspberry Pi OS source image and verify its
   published signature and SHA-256.
2. Rebuild the pinned kernel/Nexmon patch and privileged rvCSI helper in the
   hermetic release environment. Preserve source revisions, toolchains, SBOM,
   licenses, compiler output, and artifact digests.
3. Install the helper at its fixed root-owned path with no shell wrapper. Run it
   under the minimum capture capability, bounded resources, read-only system
   paths, and a private IPC/stdout channel to the unprivileged Rust runtime.
4. Record board revision, chipset, OS digest, kernel, firmware digest, Nexmon,
   rvCSI, AP, band/channel/width, packet source, geometry, calibration, power,
   voltage, temperature, and throttling state.
5. Run malformed-input, policy-denial, revocation, AP-loss, packet-loss,
   backpressure, reboot, and replay qualification. Confirm capture is disabled
   before authorization and after revocation/restart.
6. Reproduce the image independently, compare artifacts, sign the compatibility
   manifest, and retain rollback media and recovery instructions.

Do not replace pending digests in documentation by hand. The release pipeline
must write them from verified artifacts and bind them into signed acceptance
evidence.
