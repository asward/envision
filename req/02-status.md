## Functions

Display of current session status.

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
