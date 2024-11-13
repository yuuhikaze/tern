# Tern

Tern is a modular batch conversion interface

### Features

-   Recursively scans and converts files from a specified directory [TODO]
-   Destination of output files is customizable [TODO]
-   Conversion engine: Outsources conversion work to an external program (scriptable through Lua) [TODO]
-   Stores and manages options related to the external program [TODO]
-   Converts only modified files and allows to ignore files specified by git patterns [TODO]

### Installation

Download a binary from the releases section.

### Usage

```bash
tern # Runs conversion engines set in .tern-profiles.json, if no configuration is found, `tern` is resolved to `tern --profile-manager`
tern --profile-manager/-pm # Displays a GUI to get user parameters: source directory, output directory, conversion engine, options, ignore patterns
```
