---
status: accepted
---

# Own the random number generator

A stored Seed must reproduce the same Match indefinitely, making the generator part of
observable game behaviour, not an implementation detail. The `rand` ecosystem doesn't
promise value-stability across major versions — `StdRng` is documented as free to change
algorithm — so a routine dependency bump could silently invalidate every recorded Match.
We implement the generator in-tree instead (`xoshiro256**`, seeded via `SplitMix64`, both
fixed public-domain designs), taking no RNG dependency at all.

## Consequences

- Replays survive dependency upgrades — the entire point.
- Randomness is drawn from **substreams** derived from the Seed plus a domain tag, not
  one global sequence, so a change to how the Shop draws can't shift what Combat draws.
  Seeds stay meaningful across engine changes, not just across time.
- We own the generator's correctness, mitigated by tests and by choosing designs whose
  reference output is published.
- The output is statistically fine for a game and unsuitable for cryptography — nothing
  here should use it for anything else.
- Changing the `Domain` tag values or the mixing function breaks old Seeds. Those
  constants are frozen; treat them as a file format.
