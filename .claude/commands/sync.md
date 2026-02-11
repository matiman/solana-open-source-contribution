# sync

Fetch latest from origin and rebase current branch onto main.

## Arguments
- None

## Instructions

1. Run `git branch --show-current` to get the current branch

2. If currently on `main` branch:
   - Simply pull latest: `git pull origin main`
   - Show `git log --oneline -5` to confirm
   - Exit

3. Run `git status` to check if working directory is dirty

4. If working directory has uncommitted changes:
   - Stash them: `git stash save "WIP: auto-stash before sync"`
   - Note that changes were stashed (will be reapplied later)

5. Fetch latest from origin:
   ```bash
   git fetch origin
   ```

6. Rebase current branch onto latest main:
   ```bash
   git rebase origin/main
   ```

7. If rebase encounters conflicts:
   - Show the conflicting files: `git status`
   - Explain to the user that conflicts need to be resolved manually
   - Provide instructions:
     ```
     To resolve conflicts:
     1. Edit the conflicting files to resolve markers
     2. Stage resolved files: git add <file>
     3. Continue rebase: git rebase --continue

     To abort the rebase: git rebase --abort
     ```
   - DO NOT attempt to auto-resolve conflicts
   - Exit and let user handle it

8. If rebase succeeds and changes were stashed earlier:
   - Reapply stash: `git stash pop`
   - If stash pop has conflicts, show them and provide guidance

9. Show final state:
   - `git log --oneline -5` to show recent commits
   - `git status` to show working directory state
   - Indicate whether branch is ahead/behind remote

## Important Notes
- NEVER use `git pull --rebase` on dirty working directory without stashing first
- NEVER skip rebase conflicts — always surface them to the user
- DO NOT use `--force` flags unless explicitly requested
- If the rebase fails, explain clearly what happened and how to recover
- Preserve user's uncommitted work by stashing before rebase
- This command maintains a linear history (rebase, not merge)
