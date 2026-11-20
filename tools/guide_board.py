#!/usr/bin/env python3
"""Generate each guide's board from its plan: GUIDE.md from PLAN.md, and
GUIDE_GUI.md from PLAN_GUI.md.

A plan's step heads are the only place a step's title, class and status are
written; the board in its guide is generated from them, so the two cannot
disagree. The design and the board's format are Rarog's
`tools/diag/guide_board.py`, adapted to this repository.

What a plan must look like (the grammar this reads):

- its steps live in one level-2 section, the one holding the level-3
  `### Order of work`. Every other level-3 heading in that section is a
  group. A group whose heading ends `— CLOSED` (with an optional date) shows
  its first paragraph on the board instead of steps; a group holding step
  heads shows them; any other group is not shown;
- a step head is a list item, at any indent, whose bold run reads
  `**<ID> <title> — <field>.**`, possibly wrapped over several lines. PLAN.md
  numbers its steps (`7`, a sub-step `7.1`); PLAN_GUI.md letters them (`C`,
  a sub-step `C.1`). A sub-step follows its parent. The title is what the
  board shows. The field names the step's capability classes in backticks
  (`R3`, `R2`, `I2`, `I1`, `M`, `V`; "`R2`, then `I2`" for a step taken in
  two parts) and may say EXIT, IN PROGRESS, or one of DONE, REJECTED,
  DEFERRED, PARKED, FIXED, CLOSED with an optional date, which resolves it;
- the `### Order of work` table lists open steps in the order they are
  taken, one `| <ID> | <waits for> |` row each. Its first row is the
  board's Now; every row names an open step.

The board sits first in the guide, between its BEGIN and END markers. Each
group is a level-2 heading and each step a task-list item, `- [x]` with its
outcome or `- [ ]` with its classes in bold; a sub-step is indented four
spaces. Everything outside the markers is hand-kept.

Usage:
  python tools/guide_board.py            # rewrite both boards in place
  python tools/guide_board.py --check    # exit 1 if either board is stale
  python tools/guide_board.py --self-test
"""

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

NUMBERED = r"\d+(?:\.\d+)?"
LETTERED = r"[A-Z](?:\.\d+)?"
# (plan, guide, step-ID pattern)
PAIRS = (
    ("PLAN.md", "GUIDE.md", NUMBERED),
    ("PLAN_GUI.md", "GUIDE_GUI.md", LETTERED),
)

END = "<!-- end of generated board -->"
LEVEL2 = re.compile(r"^## ")
LEVEL3 = re.compile(r"^### (.+)$")
CLOSED = re.compile(r"— CLOSED(?: \d{4}-\d{2}-\d{2})?$")
CLASS = re.compile(r"`(R3|R2|I2|I1|M|V)`")
RESOLVED = re.compile(
    r"\b(DONE|REJECTED|DEFERRED|PARKED|FIXED|CLOSED)\b(?: (\d{4}-\d{2}-\d{2}))?")
IN_PROGRESS = re.compile(r"\bIN PROGRESS\b")
EXIT = re.compile(r"\bEXIT\b")
ORDER_HEADING = "### Order of work"


class PlanError(Exception):
    """A plan does not follow the grammar its board is generated from."""


def begin_marker(plan_name):
    return ("<!-- board: generated from %s by `python tools/guide_board.py`; "
            "edit %s, never this block -->" % (plan_name, plan_name))


def _bold_run(lines, i, plan_name):
    """The text of the bold run that opens on line i, joined across wraps."""
    joined = lines[i]
    j = i
    while joined.count("**") < 2:
        j += 1
        if j >= len(lines) or not lines[j].strip():
            raise PlanError("%s:%d: step head's bold run never closes" % (plan_name, i + 1))
        joined += " " + lines[j].strip()
    start = joined.index("**") + 2
    return joined[start:joined.index("**", start)]


def _summary(lines, i, plan_name):
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
        raise PlanError("%s: %s has no summary paragraph" % (plan_name, lines[i]))
    return "\n".join(para)


def _steps_section(lines, plan_name):
    """The line range [a, b) of the level-2 section holding the order of work."""
    if lines.count(ORDER_HEADING) != 1:
        raise PlanError("%s must hold exactly one '%s' section" % (plan_name, ORDER_HEADING))
    order = lines.index(ORDER_HEADING)
    a = max((i for i in range(order) if LEVEL2.match(lines[i])), default=0)
    b = next((i for i in range(order, len(lines)) if LEVEL2.match(lines[i])), len(lines))
    return a, b


def parse_plan(plan_text, id_pattern, plan_name="PLAN"):
    """The groups in plan order, each with its steps.

    A step is a dict: id, title, classes (list), resolved (e.g. 'DONE
    2026-11-14' or None), in_progress, exit.
    """
    lines = plan_text.split("\n")
    a, b = _steps_section(lines, plan_name)
    step = re.compile(r"^ *- \*\*(%s) " % id_pattern)
    groups = []
    current = None
    seen = set()
    for i in range(a, b):
        line = lines[i]
        heading = LEVEL3.match(line)
        if heading:
            current = None
            if line == ORDER_HEADING:
                continue
            closed = bool(CLOSED.search(line))
            current = {"heading": heading.group(1), "steps": [], "closed": closed,
                       "summary": _summary(lines, i, plan_name) if closed else None}
            groups.append(current)
            continue
        if current is None or current["closed"]:
            continue
        m = step.match(line)
        if not m:
            continue
        sid = m.group(1)
        if sid in seen:
            raise PlanError("%s:%d: step %s defined twice" % (plan_name, i + 1, sid))
        if "." in sid and sid.split(".")[0] not in seen:
            raise PlanError("%s:%d: sub-step %s has no step %s before it"
                            % (plan_name, i + 1, sid, sid.split(".")[0]))
        seen.add(sid)
        rest = _bold_run(lines, i, plan_name)[len(sid) + 1:]
        if " — " not in rest:
            raise PlanError("%s:%d: step %s head lacks ' — <class>'" % (plan_name, i + 1, sid))
        title, field = rest.split(" — ", 1)
        classes = CLASS.findall(field)
        if not classes:
            raise PlanError("%s:%d: step %s head names no capability class"
                            % (plan_name, i + 1, sid))
        resolved = RESOLVED.search(field)
        current["steps"].append({
            "id": sid,
            "title": title.strip().rstrip("."),
            "classes": classes,
            "resolved": " ".join(g for g in resolved.groups() if g) if resolved else None,
            "in_progress": bool(IN_PROGRESS.search(field)),
            "exit": bool(EXIT.search(field)),
        })
    return [g for g in groups if g["closed"] or g["steps"]]


def parse_order(plan_text, id_pattern, plan_name="PLAN"):
    """The Order of work rows, in order: (id, waits for or None)."""
    lines = plan_text.split("\n")
    _steps_section(lines, plan_name)
    row = re.compile(r"^\|\s*(%s)\s*\|\s*(.*?)\s*\|\s*$" % id_pattern)
    rows = []
    for line in lines[lines.index(ORDER_HEADING) + 1:]:
        if line.startswith("#"):
            break
        m = row.match(line)
        if m and m.group(1) not in [sid for sid, _ in rows]:
            waits = m.group(2)
            rows.append((m.group(1), None if waits in ("", "—", "-") else waits))
    return rows


def _classes(step):
    return ", then ".join(step["classes"])


def build_board(plan_text, id_pattern, plan_name="PLAN"):
    """The board text that belongs between BEGIN and END ('\\n' newlines)."""
    groups = parse_plan(plan_text, id_pattern, plan_name)
    steps = {s["id"]: s for g in groups for s in g["steps"]}
    order = parse_order(plan_text, id_pattern, plan_name)
    for sid, _ in order:
        if sid not in steps:
            raise PlanError("%s: Order of work names %s, which has no step head"
                            % (plan_name, sid))
        if steps[sid]["resolved"]:
            raise PlanError("%s: Order of work still lists %s, which is %s: remove its row"
                            % (plan_name, sid, steps[sid]["resolved"]))
    out = []
    if order:
        sid, waits = order[0]
        now = steps[sid]
        line = "**Now: %s** (`%s`): %s." % (sid, "`, then `".join(now["classes"]),
                                             now["title"])
        if waits:
            line += " Waits for %s." % waits
        out.append(line)
    else:
        out.append("**Now:** no step in the order of work.")
    for g in groups:
        out += ["", "## " + g["heading"], ""]
        if g["closed"]:
            out.append(g["summary"])
            continue
        for s in g["steps"]:
            indent = " " * (4 * s["id"].count("."))
            if s["resolved"]:
                box, suffix = "x", s["resolved"]
            elif s["in_progress"]:
                box, suffix = " ", "**IN PROGRESS / %s**" % _classes(s)
            else:
                box, suffix = " ", "**%s**" % _classes(s)
            if s["exit"] and not s["resolved"]:
                suffix += ", EXIT"
            out.append("%s- [%s] **%s** %s — %s" % (indent, box, s["id"], s["title"], suffix))
    return "\n".join(out) + "\n"


def _split(guide_text, plan_name, guide_name):
    begin = begin_marker(plan_name)
    if guide_text.count(begin) != 1 or guide_text.count(END) != 1:
        raise PlanError("%s must hold exactly one board block between the "
                        "generated-board markers for %s" % (guide_name, plan_name))
    a = guide_text.index(begin) + len(begin)
    b = guide_text.index(END)
    if a > b:
        raise PlanError("%s's board markers are out of order" % guide_name)
    return guide_text[:a], guide_text[a:b], guide_text[b:]


def board_problems(guide_text, plan_text, id_pattern, plan_name="PLAN", guide_name="GUIDE"):
    """[] when the guide's board equals the one its plan generates, else problems."""
    try:
        _, block, _ = _split(guide_text.replace("\r\n", "\n"), plan_name, guide_name)
        want = "\n" + build_board(plan_text.replace("\r\n", "\n"), id_pattern, plan_name)
    except PlanError as error:
        return [str(error)]
    if block != want:
        return ["%s's board is stale against %s: run `python tools/guide_board.py`"
                % (guide_name, plan_name)]
    return []


def write_board(plan_name, guide_name, id_pattern):
    guide = ROOT / guide_name
    raw = guide.read_bytes().decode("utf-8")
    crlf = "\r\n" in raw
    if crlf and raw.count("\r\n") != raw.count("\n"):
        raise PlanError("%s has mixed line endings" % guide_name)
    head, _, tail = _split(raw.replace("\r\n", "\n"), plan_name, guide_name)
    plan = (ROOT / plan_name).read_text(encoding="utf-8").replace("\r\n", "\n")
    text = head + "\n" + build_board(plan, id_pattern, plan_name) + tail
    guide.write_bytes((text.replace("\n", "\r\n") if crlf else text).encode("utf-8"))


def self_test():
    numbered = "\n".join([
        "## S1. Specification",
        "",
        "- **5 Not a step: outside the steps section — `V`.** Ignored.",
        "",
        "## S8. Requests",
        "",
        "### Capability classes",
        "",
        "Not a group: it holds no step.",
        "",
        "### Order of work",
        "",
        "| Step | Waits for |",
        "|---|---|",
        "| 2 | a decision |",
        "| 3.1 | — |",
        "",
        "### History — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "Pointer paragraph, not shown.",
        "",
        "### Open requests",
        "",
        "- **(a) A group, not a step** (1–3).",
        "  - **1 First step — `I1`, DONE 2026-02-02.** Body.",
        "  - **2 Second step, wrapped",
        "    over two lines — `R2`, then `I2`.** Body.",
        "- **3 Parent — `M`, EXIT.** Body.",
        "    - **3.1 Child — `V`, IN PROGRESS.** Body.",
        "- Not a step: **9** is only cited here.",
        "",
        "## S9. Risks",
        "- **7 Outside the steps section — `V`.** Ignored.",
    ])
    want = "\n".join([
        "**Now: 2** (`R2`, then `I2`): Second step, wrapped over two lines."
        " Waits for a decision.",
        "",
        "## History — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "## Open requests",
        "",
        "- [x] **1** First step — DONE 2026-02-02",
        "- [ ] **2** Second step, wrapped over two lines — **R2, then I2**",
        "- [ ] **3** Parent — **M**, EXIT",
        "    - [ ] **3.1** Child — **IN PROGRESS / V**",
    ]) + "\n"
    got = build_board(numbered, NUMBERED)
    if got != want:
        sys.stdout.write("FAIL: board self-test\n--- want\n%s--- got\n%s" % (want, got))
        return 1
    lettered = (numbered.replace("**1 ", "**A ").replace("**2 ", "**B ")
                .replace("**3 ", "**C ").replace("**3.1 ", "**C.1 ")
                .replace("| 2 |", "| B |").replace("| 3.1 |", "| C.1 |"))
    want_lettered = (want.replace("**1**", "**A**").replace("**2**", "**B**")
                     .replace("**3**", "**C**").replace("**3.1**", "**C.1**")
                     .replace("**Now: 2**", "**Now: B**"))
    got = build_board(lettered, LETTERED)
    if got != want_lettered:
        sys.stdout.write("FAIL: lettered board self-test\n--- want\n%s--- got\n%s"
                         % (want_lettered, got))
        return 1
    guide = "# G\n\n%s\n%s%s\n\nrest\n" % (begin_marker("PLAN"), want, END)
    if board_problems(guide, numbered, NUMBERED):
        sys.stdout.write("FAIL: board self-test: a current board reads stale\n")
        return 1
    if not board_problems(guide.replace("Child", "Kid"), numbered, NUMBERED):
        sys.stdout.write("FAIL: board self-test: an edited board reads current\n")
        return 1
    if not board_problems(guide.replace("\n", "\r\n"), numbered.replace("Child", "Kid"),
                          NUMBERED):
        sys.stdout.write("FAIL: board self-test: a plan edit does not stale the board\n")
        return 1
    if not board_problems(guide, numbered, NUMBERED, plan_name="PLAN_GUI"):
        sys.stdout.write("FAIL: board self-test: another plan's markers read current\n")
        return 1
    malformed = (
        (numbered.replace(" — `V`, IN PROGRESS.** Body.", ".** Body."), "a head without ' — '"),
        (numbered.replace("— `M`, EXIT.**", "— EXIT.**"), "a head without a class"),
        (numbered.replace("| 3.1 | — |", "| 3.1 | — |\n| 1 | — |"), "a finished step in the order"),
        (numbered.replace("| 3.1 | — |", "| 8 | — |"), "an unknown step in the order"),
        (numbered.replace("  - **1 First", "  - **2 First"), "a step defined twice"),
        (numbered.replace("    - **3.1 Child", "    - **4.1 Child")
         .replace("| 3.1 |", "| 4.1 |"), "a sub-step without its step"),
        (numbered.replace("### Order of work", "### Order"), "a plan without an order of work"),
    )
    for bad, why in malformed:
        try:
            build_board(bad, NUMBERED)
        except PlanError:
            continue
        sys.stdout.write("FAIL: board self-test accepted %s\n" % why)
        return 1
    sys.stdout.write("board generator self-test: PASS (numbered and lettered output, "
                     "staleness, %d malformed plans)\n" % len(malformed))
    return 0


def main():
    parser = argparse.ArgumentParser(description="Generate each guide's board from its plan.")
    parser.add_argument("--check", action="store_true", help="exit 1 if a board is stale")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    if args.self_test:
        return self_test()
    try:
        if args.check:
            problems = []
            for plan_name, guide_name, id_pattern in PAIRS:
                problems += board_problems(
                    (ROOT / guide_name).read_text(encoding="utf-8"),
                    (ROOT / plan_name).read_text(encoding="utf-8"),
                    id_pattern, plan_name, guide_name)
            for problem in problems:
                sys.stdout.write("  %s\n" % problem)
            return 1 if problems else 0
        for plan_name, guide_name, id_pattern in PAIRS:
            write_board(plan_name, guide_name, id_pattern)
            sys.stdout.write("%s board regenerated from %s\n" % (guide_name, plan_name))
    except PlanError as error:
        sys.stdout.write("FAIL: %s\n" % error)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
