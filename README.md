# Tern

Tern is a modular batch conversion interface

# Disclaimer

Tern is now in a usable status (beta), so no guarantees; also, some features are incomplete.

### Features

-   Recursively scans and converts files from a specified directory
-   Incrementally updates converted files
-   Converter: Outsources conversion work to an external program (scriptable through Lua)
-   Stores and manages options related to the external program
-   Converts only modified files and allows to select files through git ignore patterns
-   Destination of output files is customizable

### Installation

1.  Clone the repository and `cd` into it
1.  Run: `cargo install --path . --locked`

> You require 1.86.0-nightly (or greater) Rust compiler

### Usage

Converters are manually created by the user and must be placed under `converters` found in the [project's data directory](https://docs.rs/directories/5.0.1/directories/struct.ProjectDirs.html#method.data_dir).

```bash
tern # Runs configured conversion engines; if there is no such configuration, `tern` is resolved to `tern --profile-manager`
tern -h # Prints help
```

### Demo

```lua
-- /home/user/.local/share/tern/converters/pandoc.lua
function concat_with_space(...)
  return table.concat({...}, " ")
end

function convert(input, output, options)
    local command = concat_with_space("pandoc", options[1], input, "-o", output)
    return os.execute(command)
end

return convert
```

[VIDEO HERE]
