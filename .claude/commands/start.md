# start

Begin a new feature workflow by creating a branch from main.

## Arguments
- `$ARGUMENTS` (required): Branch description (e.g., "add-circuit-breaker", "fix-zero-price-bug", "optimize-memory-layout")

## Instructions

1. Run `git branch --show-current` to see the current branch

2. Run `git status` to check if the working directory is clean (NEVER use `-uall` flag)

3. If working directory is dirty:
   - Warn the user that there are uncommitted changes:
     ```
     WARNING: You have uncommitted changes.

     Options:
     1. Stash them: git stash save "WIP: describe changes"
     2. Commit them first: use /commit
     3. Cancel this operation

     What would you like to do?
     ```
   - Wait for user response
   - If user wants to stash: `git stash save "WIP: stashing before creating new branch"`
   - If user wants to commit: guide them to use `/commit` command first
   - If user cancels: exit gracefully

4. Fetch the latest main branch:
   ```bash
   git fetch origin main
   ```

5. Check if local main is behind origin/main:
   ```bash
   git rev-list --count main..origin/main
   ```
   - If count > 0, warn:
     ```
     Your local main is N commit(s) behind origin/main.
     Consider syncing first: git checkout main && git pull

     Do you want to continue creating branch from local main? (y/n)
     ```
   - Wait for confirmation

6. Determine the branch type from `$ARGUMENTS`:
   - If contains "add" or "new" or implies new feature: `feature/`
   - If contains "fix" or "bug": `fix/`
   - If contains "refactor" or "cleanup" or "simplify": `refactor/`
   - If contains "perf" or "optim" or "speed" or "faster": `perf/`
   - If contains "doc": `docs/`
   - If contains "test": `test/`
   - If contains "ci" or "pipeline" or "workflow": `ci/`
   - If contains "chore" or "deps" or "update": `chore/`
   - Default: `feature/`

7. Construct the branch name:
   - Prefix with type (e.g., `feature/`, `fix/`, `perf/`)
   - Use lowercase with hyphens (e.g., `add-circuit-breaker`)
   - Keep it concise but descriptive
   - Clean `$ARGUMENTS`: remove spaces, special chars, use hyphens

8. Check if branch already exists:
   ```bash
   git branch --list <branch-name>
   ```
   - If exists, error:
     ```
     ERROR: Branch '<branch-name>' already exists.

     Options:
     1. Check it out: git checkout <branch-name>
     2. Choose a different name
     3. Delete the old one: git branch -D <branch-name> (if safe)
     ```
   - Exit without creating

9. Create and checkout the new branch from main:
   ```bash
   git checkout -b <branch-name> main
   ```

10. Confirm success:
    ```bash
    git branch --show-current
    ```
    - Display to user:
      ```
      ✓ Created and switched to branch: <branch-name>
      ✓ Ready for work!

      When done, use /ship to commit, push, and create PR.
      ```

## Important Notes
- Always create from `main` branch (this is the project's main branch per CLAUDE.md)
- Never create a branch with uncommitted changes unless explicitly instructed
- Branch names should be descriptive enough to understand at a glance
- Use hyphens (not underscores or camelCase) in branch names
- Fetching latest main ensures you're branching from the most recent code
- This command prepares you for work — use `/ship` when done to complete the workflow
