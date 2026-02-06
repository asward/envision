## Functions

00-F01: **session**
Create a running session of env-var tracking. Store env vars and allows diffs
and saves of current sessions to file.

00-F{}: **status**
Display active profiles, snapshot, change counts, and dirty/clean state.

00-F{}: **set <var> <value>**
Set an environment variable.

00-F{}: **unset <var>**
Unset an environment variable.

00-F{}: **clear**
Remove all tracked changes, and profile create envvar.

00-F{}: **profile**
Loads profile files and clearly indicates the active profile.

## Requirements

### 00 - Baseline

**00-R1:** When any command except session init is invoked, the system shall verify that session init has been run first.

**00-R2:** When any command except session init is invoked and init has not been run, the system shall return an error.

**00-R3:** When the system creates snapshots, the system shall name them with timestamp or sequence number.

**00-R4:** When snapshot count exceeds the configured threshold, the system shall prune old auto-snapshots.

**00-R5:** When storing tracking data, the system shall use per-user, per-session storage.

**00-R6:** When a command completes successfully, the system shall exit with code 0.

**00-R7:** When a command fails, the system shall exit with a non-zero code.

**00-R8:** When an error occurs, the system shall display a clear error message with actionable guidance.

**00-R9:** When tracking data is corrupted, the system shall degrade gracefully.

**00-R10:** When the system detects a session state the system should indicate
that state visually within the current session.
