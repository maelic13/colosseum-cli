# Opening-book utilities

`colosseum-cli book` works on EPD or PGN files without launching an engine.
The format follows the file extension; use `--format epd|pgn` to override it.
For PGN, `--plies` selects how many half-moves form each opening.

```powershell
colosseum-cli book verify openings.epd
colosseum-cli book stats openings.pgn --plies 12 --json
colosseum-cli book hash openings.epd
colosseum-cli book slice openings.pgn subset.epd --plies 12 --count 200 --order random --seed 42
```

An EPD line is a position's four FEN fields (board, side to move, castling,
en passant) followed by optional opcodes. Its move counters — the halfmove
clock and the fullmove number — are the two integers right after those
fields when the line has them, as a FEN's last two fields; otherwise the
`hmvc` and `fmvn` opcodes; otherwise `0 1`. The engines are sent the position
with those counters, so a book's 50-move clocks and move numbers hold in its
games.

`verify` accounts for every non-comment EPD line or PGN game. It exits with 1
and lists one-based rejected candidate indices when any candidate is malformed,
or when none is usable. Normal parsing never turns rejection into startpos.

`hash` reports SHA-256 over the exact input bytes. `stats` reports that identity
alongside byte size, candidate/usable/rejected counts, unique and duplicate
resolved openings, min/mean/max retained plies, and an evaluation band when EPD
`ce` (centipawns) or PGN `[%eval ...]` (pawns) annotations are present. Units
remain explicit and are never mixed.

`slice` first requires a clean verification, then applies the requested start,
count and sequential or versioned named-stream random order. The output is
one whole FEN per line, move counters included, so a slice plays as its book
does: a PGN opening is replayed and materialized as its resulting position.
JSON records input/output hashes, seed, order, requested range and actual
count. Existing output is refused unless `--force` is explicit; input
and output must differ.


## How a run consumes a book

Every game-playing command takes book entries sequentially from `--book-start`
in the resolved order. A schedule that needs more entries than remain is
refused when the configuration is resolved, naming how many it needs, how many
remain and the shortfall, so two segments of one book cannot silently replay
the same openings. `--book-wrap` opts into modular reuse and is recorded with
the run. `--dry-run` reports the exact index range a run will consume before it
starts. Reuse is reported either way, because it narrows error bars
misleadingly and should never be a surprise.

`slice` is the usual answer to a shortfall: cut a larger book into shards, one
per run, instead of restarting inside the same range.
