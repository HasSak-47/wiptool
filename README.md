# wiptool

`wiptool` is a small CLI for keeping track of projects, tasks, and what is still in progress.

Each tracked project has project metadata and task status stored separately from the global tracker index. The default local format uses a `status.toml` file for the project data.

## Status files

For local TOML storage, project data is written to `status.toml`. The tracker keeps its own index of where project data is stored and where the actual project lives.
