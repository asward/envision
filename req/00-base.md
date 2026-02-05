## Functions

00-F01: **init**
Create baseline snapshot of current environment state. Required before other commands.

00-F{}: **status current**
Display active snapshot, change counts, and dirty/clean state.

00-F{}: **manipulate set <var> <value>**
Set and track an environment variable.

00-F{}: **manipulate unset <var>**
Unset and track removal of a variable.

00-F{}: **manipulate clear**
Remove all tracked changes, restore to baseline.

00-F{}: **diff show**
Compare current state to baseline, categorize as original/tracked/untracked.

## Requirements

### 00 - Baseline

**00-R1:** When any command except init is invoked, the system shall verify that init has been run first.

**00-R2:** When any command except init is invoked and init has not been run, the system shall return an error.

**00-R3:** When the system creates snapshots, the system shall name them with timestamp or sequence number.

**00-R4:** When snapshot count exceeds the configured threshold, the system shall prune old auto-snapshots.

**00-R5:** When storing tracking data, the system shall use per-user, per-session storage.

**00-R6:** When a command completes successfully, the system shall exit with code 0.

**00-R7:** When a command fails, the system shall exit with a non-zero code.

**00-R8:** When an error occurs, the system shall display a clear error message with actionable guidance.

**00-R9:** When tracking data is corrupted, the system shall degrade gracefully.

---

### 02 - status current

**02-R1:** When status current is invoked and the session is not initialized, the system shall return an error message "run 'init' first".

**02-R2:** When status current is invoked, the system shall display the name of the currently loaded snapshot or "baseline" if none is loaded.

**02-R3:** When status current is invoked, the system shall display the timestamp of when the current snapshot was created or loaded.

**02-R4:** When status current is invoked, the system shall display the count of tracked variables.

**02-R5:** When status current is invoked, the system shall display the count of untracked changes.

**02-R6:** When status current is invoked, the system shall display the total count of variables that differ from baseline.

**02-R7:** When untracked changes exist, the system shall mark the state as "dirty".

**02-R8:** When the current state exactly matches the loaded snapshot, the system shall mark the state as "clean".

**02-R9:** When the baseline snapshot is missing or corrupted, the system shall flag this condition.

**02-R10:** When status current completes and the state is clean, the system shall exit with code 0.

**02-R11:** When status current completes and the state is dirty, the system shall exit with code 1.

---

### 03 - manipulate set

**03-R1:** When manipulate set is invoked and the session is not initialized, the system shall return an error.

**03-R2:** When manipulate set is invoked, the system shall validate the variable name follows POSIX naming rules.

**03-R3:** When the variable name is invalid, the system shall reject the operation with an error.

**03-R4:** When manipulate set is invoked with a valid variable name, the system shall set the variable in the current shell environment.

**03-R5:** When manipulate set is invoked, the system shall export the variable to make it available to child processes.

**03-R6:** When setting a variable, the system shall record the change in tracking data.

**03-R7:** When setting a variable that already exists, the system shall store the previous value.

**03-R8:** When setting a variable, the system shall mark the variable as "tracked".

**03-R9:** When auto-snapshot is enabled, the system shall create a snapshot before modifying the variable.

**03-R10:** When manipulate set completes, the system shall confirm the variable was set.

**03-R11:** When setting a variable that already existed, the system shall display the previous value.

**03-R12:** When setting a variable, the system shall indicate whether it overwrites a tracked or untracked variable.

**03-R13:** When setting a system-critical variable (PATH, HOME, etc.), the system shall display a warning.

**03-R14:** When the variable already exists with the same value, the system shall not create a snapshot.

**03-R15:** When the variable name conflicts with a shell built-in, the system shall warn but allow the operation.

**03-R16:** When the value contains special characters or quotes, the system shall handle escaping properly.

---

### 04 - manipulate unset

**04-R1:** When manipulate unset is invoked and the session is not initialized, the system shall return an error.

**04-R2:** When manipulate unset is invoked, the system shall validate that the variable name exists.

**04-R3:** When manipulate unset is invoked with a valid variable name, the system shall unset the variable from the current shell environment.

**04-R4:** When unsetting a variable, the system shall record the removal in tracking data.

**04-R5:** When unsetting a variable, the system shall store the removed value for potential restoration.

**04-R6:** When unsetting a variable, the system shall mark the removal as "tracked".

**04-R7:** When auto-snapshot is enabled, the system shall create a snapshot before removing the variable.

**04-R8:** When manipulate unset completes, the system shall confirm the variable was unset.

**04-R9:** When unsetting a variable, the system shall display the value that was removed.

**04-R10:** When unsetting a variable, the system shall indicate whether it was tracked, untracked, or original.

**04-R11:** When unsetting a system-critical variable, the system shall display a strong warning.

**04-R12:** When unsetting a non-existent variable, the system shall warn but succeed.

**04-R13:** When unsetting a readonly variable, the system shall fail with a clear error.

---

### 05 - manipulate clear

**05-R1:** When manipulate clear is invoked and the session is not initialized, the system shall return an error.

**05-R2:** When manipulate clear is invoked without --force flag, the system shall require interactive confirmation.

**05-R3:** When manipulate clear is invoked, the system shall display a preview of changes before applying.

**05-R4:** When auto-snapshot is enabled, the system shall create a snapshot before clearing.

**05-R5:** When manipulate clear is invoked, the system shall remove all variables that were set through the tool.

**05-R6:** When manipulate clear is invoked, the system shall restore any variables that were unset through the tool.

**05-R7:** When manipulate clear is invoked, the system shall not modify untracked variables.

**05-R8:** When manipulate clear is invoked, the system shall not modify original baseline variables.

**05-R9:** When manipulate clear completes, the system shall display all variables removed.

**05-R10:** When manipulate clear completes, the system shall display all variables restored.

**05-R11:** When manipulate clear completes, the system shall display the final state.

**05-R12:** When no tracked changes exist, the system shall succeed with message "nothing to clear".

**05-R13:** When the baseline snapshot is missing, the system shall error and refuse to clear.

**05-R14:** When clearing encounters readonly variables, the system shall clear what is possible and report failures.

---

### 06 - diff show

**06-R1:** When diff show is invoked and the session is not initialized, the system shall return an error.

**06-R2:** When diff show is invoked, the system shall compare the current environment to the baseline snapshot.

**06-R3:** When diff show is invoked, the system shall categorize each variable as original, tracked, or untracked.

**06-R4:** When diff show is invoked, the system shall detect additions, removals, and modifications.

**06-R5:** When displaying added variables, the system shall show: "+ VAR=value (tracked|untracked)".

**06-R6:** When displaying removed variables, the system shall show: "- VAR=value".

**06-R7:** When displaying modified variables, the system shall show: "~ VAR: old_value → new_value (tracked|untracked)".

**06-R8:** When displaying unchanged variables, the system shall show only a summary count.

**06-R9:** When a filter option is provided, the system shall show only tracked changes if requested.

**06-R10:** When a filter option is provided, the system shall show only untracked changes if requested.

**06-R11:** When a pattern filter is provided, the system shall show only variables matching the pattern.

**06-R12:** When diff show is invoked, the system shall use human-readable diff format by default.

**06-R13:** When machine-parseable format is requested, the system shall output in JSON or CSV format.

**06-R14:** When displaying output, the system shall use color coding: green for tracked, yellow for untracked, red for removed.

**06-R15:** When the current state matches baseline exactly, the system shall display "No changes".

**06-R16:** When variable values are very long, the system shall truncate with ellipsis.

**06-R17:** When variable values contain binary or non-printable characters, the system shall show escaped or hex representation.

---
