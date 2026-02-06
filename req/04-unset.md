## Requirements

**04-R2:** When unset is invoked, the system shall validate that the variable name exists.

**04-R3:** When unset is invoked with a valid variable name, the system shall unset the variable from the current shell environment.

**04-R4:** If a valid session exists, when unsetting a variable, the system shall record the removal in tracking data.

**04-R5:** If a valid session exists, when unsetting a variable, the system shall store the removed value for potential restoration.

**04-R6:** If a valid session exists, when unsetting a variable, the system shall mark the removal as "tracked".

**04-R7:** If a valid session exists, when auto-snapshot is enabled, the system shall create a snapshot before removing the variable.

**04-R8:** When unset completes, the system shall confirm the variable was unset.

**04-R9:** When unsetting a variable, the system shall display the value that was removed.

**04-R10:** When unsetting a variable, the system shall indicate whether it was tracked, untracked, or original.

**04-R11:** When unsetting a system-critical variable, the system shall display a strong warning.

**04-R12:** When unsetting a non-existent variable, the system shall warn but succeed.

**04-R13:** When unsetting a readonly variable, the system shall fail with a clear error.

---
