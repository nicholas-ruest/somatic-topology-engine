# Deterministic replay fixture format

`RVCSIv1\0` files contain a sequence of little-endian `u32`-length-prefixed
records. Each record contains sequence `u64`, event time `u64`, center Hz `u64`,
bandwidth Hz `u32`, antenna count `u8`, subcarrier count `u16`, and that many
little-endian `(f64, f64)` I/Q pairs. Classic PCAP fixtures carry the same record
body in each captured packet. Tests generate exact bytes to avoid opaque binary
fixtures; fuzzing mutates both containers.
