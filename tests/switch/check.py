"""Run release, ordering, keyboard, fault and timing checks in a disposable session."""

import argparse
import json
import random
import subprocess
import time

from session import Session, wait

SIDES = {"alt": (56, 100), "ctrl": (29, 97), "super": (125, 126)}
SHIFT, ESC = 42, 1


def opened(s, side, reverse=False):
    s.key(side, 1)
    if reverse:
        s.key(SHIFT, 1)
    s.tap(s.keycode)
    wait(s.visible, "Held chord did not open")
    time.sleep(0.2)  # Mapping precedes the Wayland focus handoff.


def basic(s):
    left, right = SIDES[s.modifier]
    for side in (left, right):
        for modifier_first in (True, False):
            order = s.order()
            s.key(side, 1)
            s.key(s.keycode, 1)
            wait(s.visible, "Chord did not open")
            time.sleep(0.2)
            if modifier_first:
                s.key(side, 0)
                s.expect(order[1], "modifier released with switch key held")
                s.key(s.keycode, 0)
            else:
                s.key(s.keycode, 0)
                time.sleep(0.15)
                assert s.visible() and s.active() == order[0], (
                    "Switch key release committed early"
                )
                s.key(side, 0)
            s.expect(
                order[1], f"{s.modifier} side={side} release-order={modifier_first}"
            )
    for shift_first in (True, False):
        order = s.order()
        opened(s, left, reverse=True)
        if shift_first:
            s.key(SHIFT, 0)
            time.sleep(0.12)
            assert s.visible() and s.active() == order[0], (
                "Shift-only release committed"
            )
        s.key(left, 0)
        s.expect(order[-1], f"reverse entry shift-first={shift_first}")
        if not shift_first:
            s.key(SHIFT, 0)
    order = s.order()
    opened(s, left)
    s.tap(s.keycode)
    if s.legacy or s.proxy.unsupported.is_set():
        time.sleep(
            0.12
        )  # Untagged/unsupported paths cannot order independently spawned IPC.
    s.key(left, 0)
    s.expect(order[2], "queued navigation before release uses stable snapshot")
    order = s.order()
    opened(s, left)
    s.key(SHIFT, 1)
    s.tap(s.keycode)
    if s.legacy or s.proxy.unsupported.is_set():
        time.sleep(0.12)
    s.key(SHIFT, 0)
    s.key(left, 0)
    s.expect(order[0], "direction change returns to initial selection")
    for first, second in ((left, right), (right, left)):
        order = s.order()
        opened(s, first)
        s.key(second, 1)
        s.key(first, 0)
        time.sleep(0.2)
        assert s.visible() and s.active() == order[0], (
            "Physical-side overlap committed early"
        )
        s.key(first, 1)
        s.key(second, 0)
        time.sleep(0.2)
        assert s.visible(), "Return handoff committed early"
        s.key(first, 0)
        s.expect(order[1], "both-side handoff waits for final modifier release")
    original = s.active()
    opened(s, left)
    s.ipc({"CloseSwitch": {"switch": True}})  # Old compositor close during a new hold.
    time.sleep(0.6)
    assert s.visible() and s.active() == original, (
        "Stale close committed held selection"
    )
    s.tap(ESC)
    wait(lambda: not s.visible(), "Escape did not hide overlay")
    s.key(left, 0)
    time.sleep(0.25)
    assert s.active() == original and not s.visible(), "Escape changed focus later"
    print("PASS: sustained hold, stale close and Escape", flush=True)
    # Initial switch keys must be consumed before focus transfers to GTK.
    received = (s.output / "windows.log").read_text()
    forbidden = (
        ("key 65289\n", "key 65056\n") if s.switch_key == "Tab" else ("key 65475\n",)
    )
    assert not any(k in received for k in forbidden), (
        "Switch key leaked to controlled application"
    )
    print("PASS: switch key consumption", flush=True)


def ordering(s):
    expected = s.order()[1]
    s.ipc({"CloseSwitch": {"switch": True}})
    s.ipc({"OpenSwitch": {"reverse": False}})
    s.expect(expected, "release-before-open selects recent window and hides overlay")
    for i in range(30):
        expected = s.order()[1]
        procs = [
            subprocess.Popen(
                [s.binary, "socat", json.dumps(message)],
                env=s.env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            for message in (
                {"OpenSwitch": {"reverse": False}},
                {"CloseSwitch": {"switch": True}},
            )
        ]
        for p in procs:
            assert p.wait(timeout=3) == 0
        s.expect(expected, f"concurrent open/release {i + 1}")
    original = s.active()
    old = int(time.monotonic() * 1000) & 0xFFFFFFFF
    opened(s, SIDES[s.modifier][0])
    delayed_id = int(
        s.raw(
            "repl",
            f"local s = _G.__hyprshell_key_time; s.serial = s.serial + 1; s.pending[s.serial] = {old}; return tostring(s.serial)",
        )
    )
    s.tap(ESC)
    wait(lambda: not s.visible(), "Escape did not cancel")
    s.key(SIDES[s.modifier][0], 0)
    s.ipc({"OpenSwitch": {"reverse": False, "event_time": old, "event_id": delayed_id}})
    time.sleep(0.3)
    assert s.active() == original and not s.visible(), (
        "Delayed cancelled open reopened or changed focus"
    )
    print("PASS: timestamped open before Escape stays cancelled", flush=True)
    expected = s.order()[1]
    side = SIDES[s.modifier][0]
    s.key(side, 1)
    s.key(s.keycode, 1)
    time.sleep(0.01)
    s.key(side, 0)
    s.key(s.keycode, 0)
    s.expect(expected, "next quick chord after cancellation is not swallowed")


def receipts(s):
    order = s.order()
    side = SIDES[s.modifier][0]
    opened(s, side)
    timestamp = int(time.monotonic() * 1000) & 0xFFFFFFFF
    # Issue two opens before release, then deliver the newer one first. Merely
    # remembering the latest event time would lose the older pending command.
    reply = s.raw(
        "repl",
        f"local s = _G.__hyprshell_key_time; s.serial = s.serial + 2; s.pending[s.serial - 1] = {timestamp}; s.pending[s.serial] = {timestamp + 1}; return tostring(s.serial)",
    )
    newer = int(reply.strip())
    s.key(side, 0)
    s.ipc(
        {
            "OpenSwitch": {
                "reverse": True,
                "event_time": timestamp + 1,
                "event_id": newer,
            }
        }
    )
    time.sleep(0.25)
    assert s.visible() and s.active() == order[0], (
        "Newer receipt hid an older pending open"
    )
    s.ipc(
        {
            "OpenSwitch": {
                "reverse": False,
                "event_time": timestamp,
                "event_id": newer - 1,
            }
        }
    )
    s.expect(order[1], "all issued navigation arrives before commit, even out of order")


def reload_selection(s):
    original = s.active()
    old_time = int(time.monotonic() * 1000) & 0xFFFFFFFF
    opened(s, SIDES[s.modifier][0])
    s.proxy.seen.clear()
    s.proxy.stall.set()
    assert s.proxy.seen.wait(1), "No request before reload"
    s.ipc("Reload")
    wait(lambda: not s.visible(), "Reload left old selection visible")
    time.sleep(0.3)
    assert not s.proxy.pending, "Reload kept an old request alive"
    queries = s.proxy.queries
    time.sleep(0.2)
    assert queries == s.proxy.queries, "Old component kept polling after reload"
    s.key(SIDES[s.modifier][0], 0)
    s.proxy.stall.clear()
    s.ipc({"OpenSwitch": {"reverse": False, "event_time": old_time}})
    time.sleep(0.2)
    assert s.active() == original and not s.visible(), (
        "Old config's open/result changed focus"
    )
    expected = s.order()[1]
    opened(s, SIDES[s.modifier][0])
    s.key(SIDES[s.modifier][0], 0)
    s.expect(expected, "reloaded component switches once with new request ownership")
    print("PASS: reload destroys old timer, request and delayed opens", flush=True)


def timing(s, count=200):
    rng = random.Random(917)
    left, right = SIDES[s.modifier]
    for i in range(count):
        expected = s.order()[1]
        if i % 2 == 0:
            side, first, hold, gap = left, left, 0.045, 0.005
        else:
            side = rng.choice((left, right))
            first = rng.choice((side, s.keycode))
            hold = rng.choice((0.001, 0.003, 0.006, 0.01, 0.015, 0.02, 0.03, 0.06))
            gap = rng.choice((0, 0.001, 0.005, 0.015))
        s.key(side, 1)
        time.sleep(0.002)
        s.key(s.keycode, 1)
        time.sleep(hold)
        s.key(first, 0)
        time.sleep(gap)
        s.key(s.keycode if first == side else side, 0)
        wait(
            lambda expected=expected: not s.visible() and s.active() == expected,
            f"Timed chord {i}: hold={hold}, gap={gap}, side={side}, first={first}",
            1.5,
        )
        time.sleep(0.03)
    print(
        f"PASS: {count} timed chords select recent window and leave no overlay",
        flush=True,
    )


def faults(s):
    left = SIDES[s.modifier][0]
    original = s.active()
    opened(s, left)
    s.proxy.seen.clear()
    s.proxy.stall.set()
    assert s.proxy.seen.wait(1), "No query to stall"
    s.tap(ESC, 0.005)
    wait(lambda: not s.visible(), "Stalled query blocked Escape for 100 ms", 0.1)
    time.sleep(0.06)
    assert not s.proxy.pending, "Cancel left a socket alive"
    count = s.proxy.queries
    time.sleep(0.25)
    assert s.proxy.queries == count and s.active() == original, (
        "Cancelled polling/focus continued"
    )
    s.proxy.stall.clear()
    s.ipc({"OpenSwitch": {"reverse": False}})
    wait(s.visible, "Could not reopen cancelled selection")
    time.sleep(0.25)
    s.proxy.stall.set()
    time.sleep(0.7)
    assert s.visible(), "Timeout committed selection"
    assert len(s.proxy.durations) >= 3 and max(s.proxy.durations) < 0.4
    assert s.proxy.overlaps == 0, "Overlapping query requests"
    s.proxy.stall.clear()
    s.proxy.malformed.set()
    time.sleep(0.25)
    assert s.visible(), "eval acknowledgement treated as released"
    s.proxy.malformed.clear()
    expected = s.order()[1]
    s.key(left, 0)
    s.expect(expected, "timeout/malformed recovery commits correct selection")
    warnings = (
        (s.output / "daemon.log")
        .read_text()
        .count("Could not read switch modifier state")
    )
    assert warnings <= 2, f"Repeated warnings during failure: {warnings}"
    print(
        "PASS: bounded async requests, cancellation, malformed replies and warning suppression",
        flush=True,
    )
    s.proxy.unsupported.set()
    basic(s)
    count = s.proxy.queries
    time.sleep(0.25)
    assert s.proxy.queries == count, "Unsupported query kept polling"
    print("PASS: unsupported query uses release events and stops polling", flush=True)


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("binary")
    p.add_argument("helpers")
    p.add_argument("output")
    p.add_argument("--modifier", choices=SIDES, default="alt")
    p.add_argument("--legacy", action="store_true")
    p.add_argument("--workspaces", action="store_true")
    p.add_argument("--key", choices=("Tab", "F6"), default="Tab")
    p.add_argument(
        "--case",
        choices=(
            "baseline",
            "basic",
            "ordering",
            "timing",
            "faults",
            "receipts",
            "reload",
            "all",
        ),
        default="all",
    )
    args = p.parse_args()
    with Session(
        args.binary,
        args.helpers,
        args.output,
        args.legacy,
        args.modifier,
        args.key,
        args.workspaces,
    ) as s:
        s.keycode = 15 if args.key == "Tab" else 64
        print(
            f"Session modifier={args.modifier} key={args.key} workspaces={args.workspaces} legacy={args.legacy}",
            flush=True,
        )
        if args.case == "baseline":
            expected = s.order()[1]
            s.ipc({"CloseSwitch": {"switch": True}})
            s.ipc({"OpenSwitch": {"reverse": False}})
            time.sleep(0.5)
            print(
                f"observed overlay={s.visible()} focus_is_recent={s.active() == expected}",
                flush=True,
            )
            s.expect(
                expected, "release-before-open selects recent window and hides overlay"
            )
        if args.case in ("basic", "all"):
            basic(s)
        if args.case in ("ordering", "all"):
            ordering(s)
        if args.case in ("receipts", "all"):
            receipts(s)
        if args.case in ("reload", "all"):
            reload_selection(s)
        if args.case in ("timing", "all"):
            timing(s)
        if args.case in ("faults", "all"):
            faults(s)
