#!/usr/bin/env python3
"""Generate GUIDE.md's board from PLAN.md.

PLAN.md's step heads are the only place a step's title, class and status are
written; the board in GUIDE.md is generated from them, so the two cannot
disagree. The design is Rarog's `tools/diag/guide_board.py`, adapted to this
repository's numbering and markers.

What PLAN must look like (the grammar this reads):

- a phase is a level-3 heading `### Phase N — <name>`. Any other level-2 or
  level-3 heading ends it. A finished phase's heading ends `— CLOSED <date>`
  and its first paragraph is the summary the board shows instead of steps;
- a step head is a list item, at any indent, whose bold run reads
  `**<ID> <title> — <field>.**`, possibly wrapped over several lines. The ID
  is `N.M` or `N.M.K`, with `N` its phase's number. The title is what the
  board shows. The field names the step's capability classes in backticks
  (`R3`, `R2`, `I2`, `I1`, `M`, `V`; "`R2`, then `I2`" for a step taken in
  two parts) and may say EXIT, IN PROGRESS, or one of DONE, REJECTED,
  DEFERRED, PARKED, FIXED, CLOSED with an optional date, which resolves it;
- the `### Order of work` section's table lists open steps in the order they
  are taken, one `| <ID> | <waits for> |` row each. Its first row is the
  board's Now; every row names an open step.

The board sits in GUIDE.md between BEGIN and END below; everything outside
the markers is hand-kept.

Usage:
  python tools/guide_board.py            # rewrite GUIDE's board in place
  python tools/guide_board.py --check    # exit 1 if GUIDE's board is stale
  python tools/guide_board.py --self-test
"""

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
GUIDE = ROOT / "GUIDE.md"
PLAN = ROOT / "PLAN.md"

BEGIN = ("<!-- board: generated from PLAN.md by `python tools/guide_board.py`; "
         "edit PLAN, never this block -->")
END = "<!-- end of generated board -->"

PHASE = re.compile(r"^### Phase (\d+) — (.+)$")
HEADING = re.compile(r"^#{2,3} ")
STEP = re.compile(r"^ *- \*\*(\d+\.\d+(?:\.\d+)?) ")
CLASS = re.compile(r"`(R3|R2|I2|I1|M|V)`")
RESOLVED = re.compile(
    r"\b(DONE|REJECTED|DEFERRED|PARKED|FIXED|CLOSED)\b(?: (\d{4}-\d{2}-\d{2}))?")
IN_PROGRESS = re.compile(r"\bIN PROGRESS\b")
EXIT = re.compile(r"\bEXIT\b")
ORDER_HEADING = "### Order of work"
ORDER_ROW = re.compile(r"^\|\s*(\d+\.\d+(?:\.\d+)?)\s*\|\s*(.*?)\s*\|\s*$")

MARK_OPEN, MARK_PROGRESS, MARK_DONE = "☐", "◐", "☑"


class PlanError(Exception):
    """PLAN does not follow the grammar the board is generated from."""


def _bold_run(lines, i):
    """The text of the bold run that opens on line i, joined across wraps."""
    joined = lines[i]
    j = i
    while joined.count("**") < 2:
        j += 1
        if j >= len(lines) or not lines[j].strip():
            raise PlanError("PLAN.md:%d: step head's bold run never closes" % (i + 1))
        joined += " " + lines[j].strip()
    start = joined.index("**") + 2
    return joined[start:joined.index("**", start)]


def _summary(lines, i):
    para = []
    for line in lines[i + 1:]:
        if not line.strip():
            if para:
                break
            continue
        if line.startswith("#"):
            break
        para.append(line.rstrip())
    if not para:
        raise PlanError("%s has no summary paragraph" % lines[i])
    return "\n".join(para)


def parse_plan(plan_text):
    """The phases in PLAN order, each with its steps.

    A step is a dict: id, title, classes (list), resolved (e.g. 'DONE
    2026-11-14' or None), in_progress, exit.
    """
    lines = plan_text.split("\n")
    phases = []
    current = None
    seen = set()
    for i, line in enumerate(lines):
        phase = PHASE.match(line)
        if phase:
            closed = "— CLOSED" in line
            current = {"number": phase.group(1), "heading": line, "steps": [],
                       "closed": closed, "summary": _summary(lines, i) if closed else None}
            phases.append(current)
            continue
        if HEADING.match(line):
            current = None
            continue
        if current is None or current["closed"]:
            continue
        m = STEP.match(line)
        if not m:
            continue
        sid = m.group(1)
        if sid.split(".")[0] != current["number"]:
            raise PlanError("PLAN.md:%d: step %s under Phase %s"
                            % (i + 1, sid, current["number"]))
        if sid in seen:
            raise PlanError("PLAN.md:%d: step %s defined twice" % (i + 1, sid))
        seen.add(sid)
        rest = _bold_run(lines, i)[len(sid) + 1:]
        if " — " not in rest:
            raise PlanError("PLAN.md:%d: step %s head lacks ' — <class>'" % (i + 1, sid))
        title, field = rest.split(" — ", 1)
        classes = CLASS.findall(field)
        if not classes:
            raise PlanError("PLAN.md:%d: step %s head names no capability class"
                            % (i + 1, sid))
        resolved = RESOLVED.search(field)
        current["steps"].append({
            "id": sid,
            "title": title.strip().rstrip("."),
            "classes": classes,
            "resolved": " ".join(g for g in resolved.groups() if g) if resolved else None,
            "in_progress": bool(IN_PROGRESS.search(field)),
            "exit": bool(EXIT.search(field)),
        })
    return phases


def parse_order(plan_text):
    """The Order of work rows, in order: (id, waits for or None)."""
    lines = plan_text.split("\n")
    if lines.count(ORDER_HEADING) != 1:
        raise PlanError("PLAN.md must hold exactly one '%s' section" % ORDER_HEADING)
    rows = []
    for line in lines[lines.index(ORDER_HEADING) + 1:]:
        if HEADING.match(line):
            break
        m = ORDER_ROW.match(line)
        if m and m.group(1) not in [sid for sid, _ in rows]:
            waits = m.group(2)
            rows.append((m.group(1), None if waits in ("", "—", "-") else waits))
    return rows


def _class_text(step):
    return ", then ".join("`%s`" % cls for cls in step["classes"])


def build_board(plan_text):
    """The board text that belongs between BEGIN and END ('\\n' newlines)."""
    phases = parse_plan(plan_text)
    steps = {s["id"]: s for p in phases for s in p["steps"]}
    order = parse_order(plan_text)
    for sid, _ in order:
        if sid not in steps:
            raise PlanError("Order of work names %s, which PLAN has no head for" % sid)
        if steps[sid]["resolved"]:
            raise PlanError("Order of work still lists %s, which is %s: remove its row"
                            % (sid, steps[sid]["resolved"]))
    out = []
    if order:
        sid, waits = order[0]
        now = steps[sid]
        line = "**Now: %s** (%s): %s." % (sid, _class_text(now), now["title"])
        if waits:
            line += " Waits for %s." % waits
        out.append(line)
        if len(order) > 1:
            out.append("")
            out.append("**Then:** %s." % ", ".join(sid for sid, _ in order[1:4]))
    else:
        out.append("**Now:** no step in the order of work.")
    for p in phases:
        out += ["", p["heading"], ""]
        if p["closed"]:
            out.append(p["summary"])
            continue
        for s in p["steps"]:
            indent = "  " * (s["id"].count(".") - 1)
            notes = [_class_text(s)]
            if s["exit"]:
                notes.append("EXIT")
            if s["resolved"]:
                mark = MARK_DONE
                notes.append(s["resolved"])
            elif s["in_progress"]:
                mark = MARK_PROGRESS
                notes.append("IN PROGRESS")
            else:
                mark = MARK_OPEN
            out.append("%s- %s **%s** %s — %s"
                       % (indent, mark, s["id"], s["title"], ", ".join(notes)))
    return "\n".join(out) + "\n"


def _split(guide_text):
    if guide_text.count(BEGIN) != 1 or guide_text.count(END) != 1:
        raise PlanError("GUIDE.md must hold exactly one board block between "
                        "the generated-board markers")
    a = guide_text.index(BEGIN) + len(BEGIN)
    b = guide_text.index(END)
    if a > b:
        raise PlanError("GUIDE.md's board markers are out of order")
    return guide_text[:a], guide_text[a:b], guide_text[b:]


def board_problems(guide_text, plan_text):
    """[] when GUIDE's board equals the one PLAN generates, else problems."""
    try:
        _, block, _ = _split(guide_text.replace("\r\n", "\n"))
        want = "\n" + build_board(plan_text.replace("\r\n", "\n"))
    except PlanError as error:
        return [str(error)]
    if block != want:
        return ["GUIDE.md's board is stale against PLAN.md: run "
                "`python tools/guide_board.py`"]
    return []


def write_board():
    raw = GUIDE.read_bytes().decode("utf-8")
    crlf = "\r\n" in raw
    if crlf and raw.count("\r\n") != raw.count("\n"):
        raise PlanError("GUIDE.md has mixed line endings")
    head, _, tail = _split(raw.replace("\r\n", "\n"))
    plan = PLAN.read_text(encoding="utf-8").replace("\r\n", "\n")
    text = head + "\n" + build_board(plan) + tail
    GUIDE.write_bytes((text.replace("\n", "\r\n") if crlf else text).encode("utf-8"))


def self_test():
    plan = "\n".join([
        "## S8. Implementation plan",
        "",
        "### Order of work",
        "",
        "| Step | Waits for |",
        "|---|---|",
        "| 2.2 | a decision |",
        "| 2.3.1 | — |",
        "",
        "### Phase 1 — Old work — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "Pointer paragraph, not shown.",
        "",
        "### Phase 2 — New work",
        "",
        "- **(a) A group, not a step** (2.1–2.3).",
        "  - **2.1 First step — `I1`, DONE 2026-02-02.** Body.",
        "  - **2.2 Second step, wrapped",
        "    over two lines — `R2`, then `I2`.** Body.",
        "- **2.3 Parent — `M`, EXIT.** Body.",
        "    - **2.3.1 Child — `V`, IN PROGRESS.** Body.",
        "- Not a step: **2.9** is only cited here.",
        "",
        "### Post-release research",
        "- **2.7 Outside any phase — `V`.** Ignored.",
    ])
    want = "\n".join([
        "**Now: 2.2** (`R2`, then `I2`): Second step, wrapped over two lines."
        " Waits for a decision.",
        "",
        "**Then:** 2.3.1.",
        "",
        "### Phase 1 — Old work — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "### Phase 2 — New work",
        "",
        "- ☑ **2.1** First step — `I1`, DONE 2026-02-02",
        "- ☐ **2.2** Second step, wrapped over two lines — `R2`, then `I2`",
        "- ☐ **2.3** Parent — `M`, EXIT",
        "  - ◐ **2.3.1** Child — `V`, IN PROGRESS",
    ]) + "\n"
    got = build_board(plan)
    if got != want:
        sys.stdout.write("FAIL: board self-test\n--- want\n%s--- got\n%s" % (want, got))
        return 1
    guide = "# G\n\n%s\n%s%s\n\nrest\n" % (BEGIN, want, END)
    if board_problems(guide, plan):
        sys.stdout.write("FAIL: board self-test: a current board reads stale\n")
        return 1
    if not board_problems(guide.replace("Child", "Kid"), plan):
        sys.stdout.write("FAIL: board self-test: an edited board reads current\n")
        return 1
    if not board_problems(guide.replace("\n", "\r\n"), plan.replace("Child", "Kid")):
        sys.stdout.write("FAIL: board self-test: a PLAN edit does not stale the board\n")
        return 1
    malformed = (
        (plan.replace(" — `V`, IN PROGRESS.** Body.", ".** Body."), "a head without ' — '"),
        (plan.replace("- **2.3 Parent", "- **3.3 Parent"), "a step in the wrong phase"),
        (plan.replace("— `M`, EXIT.**", "— EXIT.**"), "a head without a class"),
        (plan.replace("| 2.3.1 | — |", "| 2.3.1 | — |\n| 2.1 | — |"), "a finished step in the order"),
        (plan.replace("| 2.3.1 | — |", "| 2.8 | — |"), "an unknown step in the order"),
        (plan.replace("  - **2.1 First", "  - **2.2 First"), "a step defined twice"),
    )
    for bad, why in malformed:
        try:
            build_board(bad)
        except PlanError:
            continue
        sys.stdout.write("FAIL: board self-test accepted %s\n" % why)
        return 1
    sys.stdout.write("board generator self-test: PASS (output, staleness, "
                     "%d malformed plans)\n" % len(malformed))
    return 0


def main():
    parser = argparse.ArgumentParser(description="Generate GUIDE.md's board from PLAN.md.")
    parser.add_argument("--check", action="store_true", help="exit 1 if the board is stale")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    if args.self_test:
        return self_test()
    try:
        if args.check:
            problems = board_problems(GUIDE.read_text(encoding="utf-8"),
                                      PLAN.read_text(encoding="utf-8"))
            for problem in problems:
                sys.stdout.write("  %s\n" % problem)
            return 1 if problems else 0
        write_board()
    except PlanError as error:
        sys.stdout.write("FAIL: %s\n" % error)
        return 1
    sys.stdout.write("GUIDE.md board regenerated from PLAN.md\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
