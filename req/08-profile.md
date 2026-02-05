## functions

- profile: Load a profile from a file, applying environment variable changes as
  defined in the profile script.

## Requirements

08-R2: When profile is invoked with a file path, the system shall verify the file exists.
08-R3: When the profile file does not exist, the system shall return an error with the file path.
08-R4: When profile is invoked with a file, the system shall validate the file has extension .profile.sh or .envision.
08-R5: When the profile file does not have a valid extension, the system shall return an error.
08-R6: When loading a profile for the first time, the system shall display the file path and request user confirmation.
08-R7: When profile is invoked with --yes flag, the system shall skip confirmation prompts.

08-R9: When loading a profile, the system shall parse the file for ENVISION_PROFILE metadata comment.
08-R10: When the ENVISION_PROFILE metadata is found, the system shall use that as the profile name.
08-R11: When no ENVISION_PROFILE metadata is found, the system shall use the filename (without extension) as the profile name.

08-R19: When a profile script execution fails, the system shall return the script's error message and exit code.
08-R20: When a profile loads successfully, if there is an active session, the system shall mark the session with the active profile name.
08-R21: When a profile loads successfully, the system shall display a confirmation with the profile name and count of variables changed.
08-R22: When profile is invoked with --dry-run flag, the system shall show what would change without applying changes.
08-R23: When running in dry-run mode, the system shall display each variable that would be set or unset.
08-R24: When loading a profile, the system shall compute and store a checksum of the profile file in an environment variable.

08-R31: When a profile file path is relative, the system shall resolve it relative to the current working directory.
08-R32: When a profile file path is absolute, the system shall use the path as provided.
08-R33: The system shall only support a single profile file command at a time
