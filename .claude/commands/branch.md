# branch

Create a new branch from the current base branch (main).

## Arguments
- `$ARGUMENTS` (required): Branch description or ticket reference (e.g., "add-circuit-breaker", "optimize-memory-layout", "fix-zero-price-bug")

## Instructions

1. Run `git branch --show-current` to see the current branch

2. Run `git status` to check if the working directory is clean

3. If working directory is dirty:
   - Warn the user that there are uncommitted changes
   - Ask if they want to stash, commit, or cancel the operation
   - If user wants to stash: `git stash save "WIP: stashing before creating new branch"`

4. Determine the branch type from `$ARGUMENTS`:
   - If related to a new feature: `feature/`
   - If fixing a bug: `fix/`
   - If refactoring: `refactor/`
   - If performance work: `perf/`
   - If documentation: `docs/`
   - If tests: `test/`
   - If chores/maintenance: `chore/`
   - If CI/CD changes: `ci/`

5. Construct the branch name:
   - Prefix with type (e.g., `feature/`, `fix/`)
   - Use lowercase with hyphens (e.g., `add-circuit-breaker`)
   - Keep it concise but descriptive

6. Check if branch already exists: `git branch --list <branch-name>`

7. Create and checkout the new branch from main:
   ```bash
   git checkout -b <branch-name> main
   ```

8. Confirm success: `git branch --show-current`

## Important Notes
- Always create from `main` branch (this is the project's main branch per CLAUDE.md)
- Never create a branch with uncommitted changes unless explicitly instructed
- Branch names should be descriptive enough to understand at a glance
- Use hyphens (not underscores or camelCase) in branch names
