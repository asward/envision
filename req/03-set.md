### Requirements

**03-R2:** When set is invoked, the system shall validate the variable name follows POSIX naming rules.

**03-R3:** When the variable name is invalid, the system shall reject the operation with an error.

**03-R4:** When set is invoked with a valid variable name, the system shall set the variable in the current shell environment.

**03-R5:** When set is invoked, the system shall export the variable to make it available to child processes.

**03-R6:** If a valid session is active, when setting a variable, the system shall record the change in tracking data.

**03-R7:** If a valid session is active, when setting a variable that already exists, the system shall store the previous value.

**03-R8:** If a valid session is active, when setting a variable, the system shall mark the variable as "tracked".

**03-R9:** If a valid session is active, when auto-snapshot is enabled, the system shall create a snapshot before modifying the variable.

**03-R10:** When set completes, the system shall confirm the variable was set.

**03-R11:** When setting a variable that already existed, the system shall display the previous value.

**03-R12:** If a valid session is active, when setting a variable, the system shall indicate whether it overwrites a tracked or untracked variable.

**03-R13:** When setting a system-critical variable (PATH, HOME, etc.), the system shall display a warning, unless the --no-warn flag is set.

**03-R14:** If a valid session is active, when the variable already exists with the same value, the system shall not create a snapshot.

**03-R16:** When the value contains special characters or quotes, the system shall handle escaping properly.
