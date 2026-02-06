### Requirements

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
