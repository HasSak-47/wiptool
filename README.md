# wiptool

`wiptool` is a small CLI for keeping track of projects, tasks, and what is still in progress.

Each tracked project has project metadata and task status stored separately from the global tracker index. The default local format uses a `status.toml` file for the project data.

The project model is organized around milestones. A project is the long-lived thing being worked on, a milestone is a version, phase, release, or checkpoint within that project, and tasks are the concrete pieces of work inside a milestone.

Projects can have kinds such as programming, hardware, drawing, or reading, so the same tracker can cover both engineering and creative work.

Milestones have lifecycle statuses such as idea, planned, active, halted, stable, experimental, abandoned, or replaced. This keeps current milestone progress separate from older completed work.

## Status files

For local TOML storage, project data is written to `status.toml` by default. The tracker keeps its own index of where project data is stored and where the actual project lives, so the project directory and status file do not need to be the same location.

The current implementation focuses on root projects, milestones, tasks, local TOML storage, project listing, milestone listing, task listing, and shell completion generation. Subprojects and importing existing project files are still incomplete.
