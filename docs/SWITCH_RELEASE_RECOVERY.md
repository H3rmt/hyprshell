# Switch modifier release recovery

Switch mode commits the current selection when neither physical side of its
configured modifier is held. GTK key-release events and generated compositor
release bindings trigger a check. A bounded asynchronous check also runs every
80 ms while a selection is open, to recover a release lost between opening and
GTK acquiring keyboard focus. There is no idle polling. Each request has a
150 ms deadline; there is at most one in flight. New navigation or release
evidence cancels an older request, and request identities reject already queued
responses after navigation, close, cancellation, or configuration replacement.

Both physical keys are read in a single compositor `repl` operation. Sequential
left/right queries can observe a false release during an overlapping handoff.
The response is strictly `pressed;pending_opens`, with two boolean values. An
`eval` acknowledgement is not a state response. Malformed responses and timeouts
leave the selection open and suppress repeated warnings until a successful
state query. The frequency bounds normal missed-event recovery to roughly one
80 ms interval plus compositor/GTK scheduling, without continuously polling.

The first open collects the recent-window snapshot. Later opens and GTK
navigation update that selection directly. Lua bindings register every issued
open in a compositor table and attach its receipt ID and original keyboard event
time to the IPC command. The atomic query acknowledges received IDs and waits
until all issued opens have arrived before committing. This matters because
separate command processes can arrive out of order: tracking only the latest
issued timestamp can lose an older navigation command. Cancellation discards
pending entries at or before its timestamp. A failed command delivery can leave
an entry pending; Escape cancels it. We intentionally do not expire a pending
navigation into a potentially wrong focus change.

Escape records its original keyboard timestamp and cancels without focusing a
selection. A timestamped open at or before that cancellation is stale, using
wrapping 32-bit event-time comparison. A newer timestamped open is accepted,
including the next quick chord. An untagged explicit `OpenSwitch` represents a
new request, even just after cancellation. `CloseSwitch { switch: true }` is
release evidence, rather than an unconditional force-commit command. It cannot
commit a newly held chord. `switch: false` remains explicit cancellation.

Unsupported state queries are detected separately from transient errors. They
stop polling and use real configured-modifier release events plus GTK's current
seat modifier mask. The mask accounts for both physical sides; lack of focus or
seat state is unknown, never proof of release. The callback waits for queued
Wayland modifier updates before reading that mask. Releasing Shift alone is not
release evidence, and no generated Shift-only release binding remains. Canonical
modifier keysyms from the existing upstream helpers are retained, as are binding
consumption flags.

Both Lua and legacy binding generation remain supported. Legacy bindings cannot
attach the original compositor timestamp or register Lua receipt IDs. Therefore
this change does not solve delayed-open cancellation ordering, arbitrarily late
navigation delivery, or every pre-focus missed-release gap on legacy
compositors. The event-and-seat-mask mechanism is separately tested with an
unsupported-query injection, including physical-side overlap and Shift release;
that does not establish equivalent recovery on an old compositor. There is no
minimum-version increase.

Timers, request futures, compatibility callbacks, and deferred focus changes
have explicit ownership. Close and cancellation remove release work; reopening
cancels a pending old focus dispatch. Reload replaces the retained GTK controller
and invalidates the previous configuration's requests. Destruction removes the
sources and the existing thumbnail timer. Thumbnail capture cleanup otherwise
retains development-branch behavior. PR 519 changes that cleanup independently
and needs a fresh conflict review if it lands.

See [the disposable reproduction instructions](../tests/switch/README.md) for
runtime checks. Test evidence for a particular candidate belongs with its exact
base and binary; previous downstream results do not validate a new build.
