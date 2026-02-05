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

---

### 03 - set

**03-R1:** When set is invoked and the session is not initialized, the system shall return an error.

**03-R2:** When set is invoked, the system shall validate the variable name follows POSIX naming rules.

**03-R3:** When the variable name is invalid, the system shall reject the operation with an error.

**03-R4:** When set is invoked with a valid variable name, the system shall set the variable in the current shell environment.

**03-R5:** When set is invoked, the system shall export the variable to make it available to child processes.

**03-R6:** When setting a variable, the system shall record the change in tracking data.

**03-R7:** When setting a variable that already exists, the system shall store the previous value.

**03-R8:** When setting a variable, the system shall mark the variable as "tracked".

**03-R9:** When auto-snapshot is enabled, the system shall create a snapshot before modifying the variable.

**03-R10:** When set completes, the system shall confirm the variable was set.

**03-R11:** When setting a variable that already existed, the system shall display the previous value.

**03-R12:** When setting a variable, the system shall indicate whether it overwrites a tracked or untracked variable.

**03-R13:** When setting a system-critical variable (PATH, HOME, etc.), the system shall display a warning.

**03-R14:** When the variable already exists with the same value, the system shall not create a snapshot.

**03-R15:** When the variable name conflicts with a shell built-in, the system shall warn but allow the operation.

**03-R16:** When the value contains special characters or quotes, the system shall handle escaping properly.

---

### 04 - unset

**04-R1:** When unset is invoked and the session is not initialized, the system shall return an error.

**04-R2:** When unset is invoked, the system shall validate that the variable name exists.

**04-R3:** When unset is invoked with a valid variable name, the system shall unset the variable from the current shell environment.

**04-R4:** When unsetting a variable, the system shall record the removal in tracking data.

**04-R5:** When unsetting a variable, the system shall store the removed value for potential restoration.

**04-R6:** When unsetting a variable, the system shall mark the removal as "tracked".

**04-R7:** When auto-snapshot is enabled, the system shall create a snapshot before removing the variable.

**04-R8:** When unset completes, the system shall confirm the variable was unset.

**04-R9:** When unsetting a variable, the system shall display the value that was removed.

**04-R10:** When unsetting a variable, the system shall indicate whether it was tracked, untracked, or original.

**04-R11:** When unsetting a system-critical variable, the system shall display a strong warning.

**04-R12:** When unsetting a non-existent variable, the system shall warn but succeed.

**04-R13:** When unsetting a readonly variable, the system shall fail with a clear error.

---

### 05 - clear

**05-R1:** When clear is invoked and the session is not initialized, the system shall return an error.

**05-R2:** When clear is invoked without --force flag, the system shall require interactive confirmation.

**05-R3:** When clear is invoked, the system shall display a preview of changes before applying.

**05-R4:** When auto-snapshot is enabled, the system shall create a snapshot before clearing.

**05-R5:** When clear is invoked, the system shall remove all variables that were set through the tool.

**05-R6:** When clear is invoked, the system shall restore any variables that were unset through the tool.

**05-R7:** When clear is invoked, the system shall not modify untracked variables.

**05-R8:** When clear is invoked, the system shall not modify original baseline variables.

**05-R9:** When clear completes, the system shall display all variables removed.

**05-R10:** When clear completes, the system shall display all variables restored.

**05-R11:** When clear completes, the system shall display the final state.

**05-R12:** When no tracked changes exist, the system shall succeed with message "nothing to clear".

**05-R13:** When the baseline snapshot is missing, the system shall error and refuse to clear.

**05-R14:** When clearing encounters readonly variables, the system shall clear what is possible and report failures.

---
