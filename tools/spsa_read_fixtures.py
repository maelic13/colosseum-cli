#!/usr/bin/env python3
"""Export finished SPSA tunes as `spsa read` agreement fixtures.

    python tools/spsa_read_fixtures.py --reference <spsa_read.py> \
        --results <dir of run directories> --out tests/fixtures/spsa-read \
        <block> [<block> ...]

Each block is a finished tune's run directory under --results. Its fixture is
the run reduced to what a read needs: the schedule artifact as the run wrote
it, games per iteration, the starting centres, every iteration's score D, and
the recorded centres every RECORDED_EVERY iterations and at the end, which the
CLI's re-derived centres must reproduce within the reference's tolerance (a
runner of another version may round a move differently in its last bit). Beside them are the reference
reader's figures, from `--reference` (the engine projects' spsa_read.py at
the revision PLAN 12.11 pins), at two reads: the last 1,000 iterations, and
1,000 iterations ending at the three-quarter point, which has a forecast.

The reference is imported and run, so it reads each block's games.jsonl to
rebuild D; the export checks those against result.json's own scores.
Standard library only.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
import tempfile
from pathlib import Path

NULL_REPS = 40
RECORDED_EVERY = 100


def load_reference(path: Path):
    spec = importlib.util.spec_from_file_location("spsa_read_reference", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader
    sys.modules["spsa_read_reference"] = module  # dataclasses resolve by name
    spec.loader.exec_module(module)
    return module


def reference_read(sr, run, at: int) -> dict:
    result = sr.read(run, at, window_size=1000, null_reps=NULL_REPS)
    window = range(at - 1000, at)
    window_gain = sr.crossfit_gain(run, run.delta, sr.actual_moves(run), window)
    return {
        "window": result["window"],
        "familywise_line": result["familywise_line"],
        "max_abs_z": result["max_abs_z"],
        "over_line": result["over_line"],
        "excess": result["excess"],
        "excess_sd": result["excess_sd"],
        "live": result["live"],
        "coordinate_elo": result["floor"]["coordinate_elo"],
        "coordinate_se_elo": result["floor"]["coordinate_se_elo"],
        "forecast_two_sd_elo": result["floor"]["forecast_two_sd_elo"],
        "gain_since_start": result["gain"]["elo"],
        "gain_since_start_null_sd": result["gain"]["null_sd"],
        "gain_window": round(window_gain, 2),
        "forecast": result["forecast"] if result["forecast"]["segment"][0] < result["forecast"]["segment"][1] else None,
        "rails": result["rails"],
        "coordinates": [
            {key: coordinate[key] for key in
             ("name", "z", "elo_across_perturbation", "theta", "arm_clamped_share", "rail")}
            for coordinate in result["coordinates"]
        ],
        "move_check_worst_error": result["move_check_worst_error"],
    }


def export(sr, directory: Path, scratch: Path) -> dict:
    schedule = json.loads((directory / "spsa-schedule.json").read_text(encoding="utf-8"))
    resolved = json.loads((directory / "resolved-config.json").read_text(encoding="utf-8"))
    result = json.loads((directory / "result.json").read_text(encoding="utf-8"))
    games = int(resolved["settings"]["games_per_iteration"])
    iterations = result["driver"]["completed_iterations"]
    start = [float(parameter["original"]) for parameter in result["tuned_result"]["parameters"]]
    rows = [start] + [[float(x) for x in iteration["centers_after"]] for iteration in iterations]
    scores = [int(iteration["score"]["difference"]) for iteration in iterations]
    names = [knob["name"] for knob in schedule["knobs"]]
    history = scratch / f"{directory.name}-history.json"
    history.write_text(json.dumps({"report": {
        "run_status": result["driver"]["status"], "completed_iterations": len(iterations), "knobs": names,
        "rows": [{"iteration": k, "centers": row} for k, row in enumerate(rows)]}}), encoding="utf-8")
    run = sr.load_run(directory, cli=None, history_json=history)
    if run.d != scores:
        raise SystemExit(f"{directory.name}: games.jsonl and result.json disagree on the scores")
    completed = len(scores)
    return {
        "block": directory.name,
        "source": "Basilisk tools/results, exported by tools/spsa_read_fixtures.py",
        "schedule": schedule,
        "games_per_iteration": games,
        "start": start,
        "scores": scores,
        "recorded_centres": {str(k): rows[k] for k in sorted({*range(0, completed + 1, RECORDED_EVERY), completed})},
        "reference": {
            "null_reps": NULL_REPS,
            "reads": [reference_read(sr, run, completed), reference_read(sr, run, completed * 3 // 4)],
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("blocks", nargs="+")
    args = parser.parse_args()
    sr = load_reference(args.reference)
    args.out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        for block in args.blocks:
            fixture = export(sr, args.results / block, Path(scratch))
            path = args.out / f"{block}.json"
            path.write_text(json.dumps(fixture, indent=1) + "\n", encoding="utf-8")
            print(f"{path}: {len(fixture['scores'])} iterations, {len(fixture['start'])} coordinates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
