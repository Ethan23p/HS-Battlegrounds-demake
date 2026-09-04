---
status: accepted
---

# Own the random number generator

A stored Seed must reproduce the same Match indefinitely, which makes the generator part
of the game's observable behaviour rather than an implementation detail. The `rand`
ecosystem does not promise value-stability across major versions — `StdRng` is explicitly
documented as free to change its algorithm — so a routine dependency bump could silently
invalidate every recorded Match. We therefore implement the generator in-tree
(`xoshiro256**`, seeded through `SplitMix64`, both fixed public-domain reference designs)
and take no RNG dependency at all.

## Consequences

- Replays survive dependency upgrades. This is the entire point.
- Randomness is drawn from **substreams** derived from the Seed plus a domain tag, not one
  global sequence, so a change to how the Shop draws cannot shift what Combat draws. Seeds
  stay meaningful across engine changes, not just across time.
- We own the generator's correctness. Mitigated by tests, and by choosing designs whose
  reference output is published rather than inventing our own.
- The output is statistically fine for a game and unsuitable for cryptography. Nothing in
  this project should ever use it for anything else.
- Changing the `Domain` tag values, or the mixing function, breaks old Seeds. Those
  constants are frozen; treat them as a file format.
