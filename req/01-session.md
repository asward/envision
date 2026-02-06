## Functions

Initialize environment tracking for the current shell session by creating a baseline snapshot.

### 01 - session

**01-R1:** When session init is invoked, the system shall capture all current environment variables and their values as the baseline snapshot.

**01-R2:** When creating a baseline, the system shall mark all captured variables with status "original".

**01-R3:** When session init is invoked, the system shall generate a unique session identifier.

**01-R4:** When session init is invoked, the system shall create storage for tracking metadata and snapshots.

**01-R5:** When session init is invoked, the system shall initialize an empty tracking state with no tracked changes.

**01-R6:** When session init is invoked, the system shall record the current timestamp and shell PID.

**01-R7:** When session init is invoked and a session already exists, the system shall return an error.

**01-R8:** When session init is invoked with --force flag and a session already exists, the system shall warn about losing tracking history and reinitialize.

**01-R9:** When session init is invoked with --resume flag and a session exists, the system shall continue the existing session.

**01-R10:** When session init completes successfully, the system shall display the count of variables captured, session identifier, and storage location.

**01-R11:** When the storage location is unavailable, the system shall fail with a clear error message.

**01-R12:** When session init detects stale session data from a crashed session, the system shall offer cleanup options.

**01-R13:** When a session is activated the system shall display a banner
message within the terminal on the top viewable line with simplified session
status and profile name (if available).

**01-R14:** When a session is active the banner must be updated after every envision operation to reflect the current session status and profile name (if available).

**01-R15:** When a profile is loaded into a session the variables added should be tracked.
