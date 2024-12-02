# Tern

Tern is a modular batch conversion interface

# Disclaimer

Tern is now in a usable status (beta), so no guarantees; also, some features are incomplete.

### Features

-   Recursively scans and converts files from a specified directory
-   Incrementally updates converted files [TODO]
-   Conversion engine: Outsources conversion work to an external program (scriptable through Lua)
-   Stores and manages options related to the external program
-   Converts only modified files and allows to select files through git ignore patterns
-   Destination of output files is customizable

### Installation

Download a binary from the releases section.

### Usage

```bash
tern # Runs conversion engines set in .tern-profiles.json, if no configuration is found, `tern` is resolved to `tern --profile-manager`
tern -h # Prints help, details other settable options
```
