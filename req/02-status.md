## Functions

Display of current session status.

### 02 - status

**02-R1:** When status is invoked and the session is not initialized, the system shall return an error message indicating that the session is not initialized and exit with code 1.

**02-R3:** When status is invoked, the system shall display the
timestamp of the last session baseline.

**02-R4:** When status is invoked, the system shall display the count of tracked variables.

**02-R5:** When status is invoked, the system shall display the count of untracked changes.

**02-R6:** When status is invoked, the system shall display the total count of variables that differ from baseline.

**02-R7:** When untracked changes exist, the system shall mark the state as "dirty".

**02-R8:** When the current state exactly matches the loaded snapshot, the system shall mark the state as "clean".

**02-R9:** When the baseline snapshot is missing or corrupted, the system shall flag this condition.

**02-R10:** When status completes and the state is clean, the system shall exit with code 0.

**02-R11:** When status completes and the state is dirty, the system shall exit with code 1.

---
