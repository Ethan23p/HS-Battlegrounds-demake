---
status: accepted
---

# Own the random number generator

A stored Seed must reproduce the same Match indefinitely — the generator is observable
game behaviour, not an implementation detail. `rand`'s `StdRng` is documented as free to
change algorithm, so a routine dependency bump could invalidate every recorded Match. We
implement the generator in-tree instead (`xoshiro256**`, seeded via `SplitMix64`, both
fixed public-domain designs) and take no RNG dependency.

## Consequences

- Replays survive dependency upgrades — the entire point.
- Randomness is drawn from **substreams** keyed by Seed + domain tag, not one global
  sequence, so a Shop-draw change can't shift Combat's draws.
- We own the generator's correctness, mitigated by tests and published reference output.
- Output is fine for a game, unsuitable for cryptography.
- Changing the `Domain` tags or mixing function breaks old Seeds — those constants are
  frozen.
