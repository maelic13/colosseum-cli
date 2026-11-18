# `spsa read` definitions (12.11, `R2`)

The definitions `spsa read` computes, each pinned against the reference the
engine projects use today, before any code. Signed off by the maintainer on
2026-11-18; PLAN 12.11 holds the step and its decisions of 2026-11-14.

## The reference

Basilisk `tools/spsa_read.py` at commit
`5f61c94644467f43daba1c31f472dc6d39cb0678` (blob SHA-256
`7367d75af1b7f14d07fe4b85f43caff78ebd62261fca66b6207a8f96d7005ede`) and its
`tools/diag/test_spsa_read.py` (blob SHA-256
`2731448add9b854557cf7ae671327c9df6e74a6228f08977b9a4144a4e9e80de`). Rarog's
copy at `638b56d` differs only in its schema names. The suite passes at that
revision, 29 tests, including the five recorded blocks it reproduces from
Basilisk's run directories (`b33`, `b46`, `b54` and `b23` blocks 1 and 2:
2,000 iterations each, 8 to 49 coordinates, 30 games an iteration).

The reference reads a run from outside: it regenerates the runner's ChaCha12
sign stream in Python and re-derives each move from a closed form of the
schedule. `spsa read` is the runner reading itself: the signs are
`perturbations_for_iteration`, the gains are `SpsaSchedule::coefficients`,
and each iteration's score is the one the journal replay gives. Every
definition below is the reference's unless it says otherwise.

## Inputs

- **Iterations** are the run's completed iterations, `0 ≤ k < K` with `K`
  the completed count, as `spsa history` replays them from the journal; a
  read is safe while the tune runs. Nothing else in the run directory is
  written or changed.
- **`δ[k][i]`** is the sign the runner drew for coordinate `i` at iteration
  `k`: the `spsa-perturbations` stream from the run's master seed,
  iteration-major, then tune-file knob order, one little-endian `u64` a
  draw, `+1` when odd.
- **`D[k]`** is the plus arm's wins minus its losses over the iteration's
  official games. Engine `a` is always the plus arm.
- **`c_k` and `r_k`** are the runner's: `c_k = c0 / (k + 1)^γ`,
  `a_k = a0 / (A + k + 1)^α`, `r_k = a_k / c_k²`, with `α = 0.601`,
  `γ = 0.102`, `A = iterations / 10` and `a0` derived from `r_end` (the
  reference's closed form `r_end ((k+1)/n)^(2γ) ((A+n)/(A+k+1))^α` is the
  same number).
- **`G`** is games per iteration. **`n`** is the number of coordinates.

## The window

`--from K0 --to K1` reads iterations `K0 ≤ k < K1`. `K1` defaults to `K` and
`K0` to `max(0, K1 − 1,000)`, so a run with fewer than 1,000 completed
iterations is read whole (the reference refuses a window shorter than 1,000;
the request asks for all). `K1 > K`, `K0 ≥ K1` or an empty run is a usage
error. `W = K1 − K0`.

## The reads

- **z.** `z[i] = Σ δ[k][i] D[k] / sqrt(Σ D[k]²)` over the window; `0` when
  every `D` is `0`. With no gradient it has mean 0 and variance 1, because
  the runner drew the signs.
- **Elo across one perturbation.** `E = 695 / (2G)` Elo per unit of mean
  `D`. A coordinate's figure is `E × Σ δ[k][i] D[k] / W`, and its standard
  error, the same for every coordinate, is `E × sqrt(Σ D[k]²) / W`. 695
  stands for `1600 / ln 10` (694.87) and is kept as written.
- **The familywise line.** `Φ⁻¹(1 − (1 − (1 − α_f)^(1/n)) / 2)` with family
  alpha `α_f`, 0.05 by default (`--family-alpha`). `Φ⁻¹` is Wichura's
  AS 241, the algorithm of Python's `NormalDist.inv_cdf`: the core's
  existing quantile, a bisection on an approximate `erf`, is off by about
  5e-5 at these tails, which moves the fourth digit.
- **The excess.** `Σ z² − n`, and the same in units of its null SD
  `sqrt(2n)`.
- **Live or quiet.** Live when some `|z|` reaches the familywise line or the
  excess reaches its line, 2 null SD by default (`--excess-sd`); otherwise
  quiet. Both lines are reported with the read. A quiet read also reports
  what it rules out: no coordinate is worth more than the line times the
  standard error, and the forecast's upper reading (twice its SD).
- **The gradient**, coordinate `i` over iterations `S`, in Elo per unit of
  the coordinate: `E × Σ δ[k][i] D[k] c_k,i / (2 Σ c_k,i²)`, every iteration
  of `S` in the denominator, `D = 0` included (leaving those out reads the
  gradient high by their share; the reference's tests pin this).
- **The gain read** over a span `[s, t)`: cut at `s, s + g, s + 2g, …` with
  `g` the gain segment (500, `--gain-segment`); a last segment shorter than
  `g` joins the one before it unless it is the only one. In each segment,
  for each coordinate, the gradient over its even iterations (by absolute
  `k`) times the coordinate's actual move summed over its odd iterations,
  plus the reverse; summed over segments and coordinates, in Elo. The
  actual move at `k` is `centre[k+1][i] − centre[k][i]`, clamping included.
  Read twice: since iteration 0, `[0, K1)`, as the reference does, and over
  the window, `[K0, K1)`, as the request adds.
- **The null SD of a gain read** is the population SD of the same read
  over `N` re-draws (40, `--null-reps`). In a re-draw every coordinate's
  sign at an iteration with `D ≠ 0` is drawn afresh (an iteration with
  `D = 0` moves nothing either way and draws nothing), and the moves are
  the unclamped `δ r_k c_k,i D[k]` of those signs. **Decided 2026-11-18:**
  the draws come from the CLI's own generator, a new named stream
  `spsa-read-null` from the null seed `S` (1, `--null-seed`), one Rademacher
  draw per sign, re-draw-major, then iteration, then coordinate, a fresh
  stream for each of the two spans. The reference draws them with Python's
  `random.Random(S)`; every other figure agrees with it to the digit, and
  the null SD agrees within sampling error (its relative SE is about
  `1 / sqrt(2(N − 1))`, 11% at 40), so the fixtures pin the CLI's own null
  values beside the reference's.
- **The forecast**, labelled an upper bound, over the next
  `[K1, min(K1 + F, iterations))` with `F` the span (2,000,
  `--forecast-span`; 0 turns it off): `factor × excess`, SD
  `factor × sqrt(2n)`, with
  `factor = (Σ_{segment} r_k f_k²) × mean_window(D²) × E / (2 × mean_window(f_k²) × W)`
  and `f_k = (k + 1)^−γ`. There is none at the ceiling, and then no forecast
  floor either.
- **Rails.** A coordinate is near a bound when its centre at `K1` is within
  `c` of it, `c` its perturbation at the last read iteration `K1 − 1`.
  Its outward `z` is `z` near the maximum, `−z` near the minimum, `|z|`
  near both, and none otherwise. A rail is an outward `z` at or past the
  familywise line. Reported per coordinate: the distance to the nearer
  bound, the outward `z` and the rail flag. **Decided 2026-11-18:** the
  reference's rail candidate (an outward `z` of 2.0 or more under the line)
  is not a CLI figure; the 2.0 is a project threshold, and a project
  applies it to the outward `z`.
- **Clamp share.** The share of iterations `[0, K1)` on which either arm of
  the coordinate left its range: `centre[k] − c_k < min` or
  `centre[k] + c_k > max`, the runner's `arm_clamp`.
- **Faults.** The run's own counters from its checkpoint (engine faults
  and time losses per engine, infrastructure). They are reported, and
  decide nothing.

## Integrity

The reference checks each move of `spsa history` against its closed form,
`move = r_k c_k D δ`, clamped where it was. Inside the CLI the centres are
that replay: the runner's own update, `clamp(centre + c_k r_k D δ)`, applied
to the journal's scores. So the read compares the replay with every record
of centres the run keeps on its own: the checkpoint's centres at its
completed count, and, for a finished run, each iteration's centres in
`result.json`. The worst deviation, in units of that iteration's unit move
`c_k r_k`, is reported; anything over `1e-6` is an error, exit 3, never a
warning. A sound run reads `0`, the runner's arithmetic being the one that
wrote them; the reference's check reads under `1e-9` on the five recorded
blocks. A journal or seed that does not match the run is refused before any
read: the journal's anchor and the verified schedule already check them.

## Output

- **Text**: a summary block (window, `n`, both lines, max `|z|` and the
  coordinates over the line, the excess in SD, live or quiet with what it
  rules out, both gain reads with their null SDs, the forecast, rails,
  clamps over 1%, faults, the integrity figure), then one row per
  coordinate ordered by `|z|`: name, `z`, Elo across a perturbation,
  centre, start, rounded value (the runner's round half away from zero),
  clamp share, rail flag.
- **`--json`**: the same under schema `colosseum-spsa-read-v1`, stable field
  names, values unrounded (the reference rounds; agreement is checked at its
  rounding).
- **Exit status**: 0 for a read; 2 for usage, or a directory that is not an
  SPSA tune; 1 for a run that cannot be read (no record, a journal that does
  not replay); 3 for an inconsistent one (integrity).

## Not in the CLI

As decided on 2026-11-14: no checkpoint grid, verdict, `faults-accepted`
workflow, `--record` log or theta writing. A project keeps those in its
wrapper, reads `spsa read --json` and pauses a tune with
`--stop-after-iteration`.

## Fixtures

- **Golden vectors** from the reference at the pinned revision: the sign
  stream, the familywise lines for 8, 15, 29 and 49 coordinates, the
  forecast floors, the gradient with zero scores, the rounding.
- **Agreement**: the five recorded blocks exported compactly by a
  repository Python tool from Basilisk's run directories (each block's
  schedule, seed, starting centres, games per iteration, every iteration's
  `D`, and a SHA-256 of its recorded centres, which the CLI's re-derived
  centres must reproduce), with the reference's figures at the pinned
  revision. A test reads them and matches every deterministic figure to the
  reference's rounding, and the null SDs within the stated sampling error
  as well as to the CLI's own committed values.
- **Synthetic runs**: a planted gradient reading live and over the line,
  none reading quiet, `z`'s variance about 1 over many seeds, the gain
  read's mean about 0 over sign-randomised runs, and a tampered journal or a
  wrong seed failing.
