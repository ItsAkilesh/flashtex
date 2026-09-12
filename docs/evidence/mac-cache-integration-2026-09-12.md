# Native preview cache integration

Reviewed exact Mac candidate6b43a3a and its ancestor typing/revision changes.
GH26 fix uses exact Double size bits, constructs CTFont at requested size, and
bounds both font and line dictionaries. Tests include10.000 versus10.004 points,
one-ULP distinction, font size equality and independent font/line eviction.

Authenticated Mac lead reported185/185 native tests with real binaries and a
successful Xcode build for this exact candidate in issue2 at08:27:33Z. Linux
Commander independently inspected source/tests and clean merge; native tests
cannot be rerun on this Linux host. Request Mac validation again on integrated main.

The accompanying typing benchmark measures programmatic text insertion through
CoreAnimation commit, not presentation or display scanout. At60KB the reported
p95 remains329ms; exact PDF bytes, pixel parity and global latency goals are not met.
The new cache does not resolve producer font substitution or original-GID shaping.
