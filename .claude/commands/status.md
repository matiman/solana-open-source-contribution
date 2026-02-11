# status

Show a comprehensive git status including branch info, uncommitted changes, recent commits, and remote tracking status.

## Arguments
- None

## Instructions

1. Run the following commands in parallel to gather complete git state:
   - `git branch --show-current` — current branch name
   - `git status` — working directory status (NEVER use `-uall` flag)
   - `git log --oneline -10` — recent commit history
   - `git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "no-upstream"` — remote tracking branch
   - `git rev-list --left-right --count HEAD...@{u} 2>/dev/null || echo "0 0"` — commits ahead/behind remote

2. Present the information in a clear, organized format:

   ```
   ## Branch Information
   - Current branch: <branch-name>
   - Remote tracking: <remote/branch or "none">
   - Ahead by: <N> commits (if > 0)
   - Behind by: <N> commits (if > 0)

   ## Working Directory Status
   - Untracked files: <count>
   - Modified files: <count>
   - Staged files: <count>

   ## Recent Commits (last 10)
   <output from git log --oneline -10>

   ## Files
   <relevant output from git status showing what's changed>
   ```

3. If working directory is dirty, highlight it clearly

4. If branch is ahead of remote, suggest pushing: `git push`

5. If branch is behind remote, suggest syncing: use `/sync` command or `git pull --rebase`

6. If there are untracked files, list them (unless there are too many, then just show count)

## Important Notes
- This is a read-only command — it only displays information, makes no changes
- Use this before performing any git operations to understand current state
- If remote tracking branch doesn't exist, note that the branch is local-only
- If on detached HEAD, explain what that means and how to recover
