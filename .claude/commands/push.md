# push

Push the current branch to its remote tracking branch.

## Arguments
- Optional: flags like `--force`, `--force-with-lease`, `--no-verify`

## Instructions

1. Run `git branch --show-current` to get the current branch name

2. Check if current branch is `main` or `master`:
   - If yes, warn the user:
     ```
     WARNING: You are about to push to the protected branch '<branch-name>'.
     This should only be done after PR merge or for emergency hotfixes.

     Do you want to continue? (y/n)
     ```
   - Wait for user confirmation before proceeding
   - If user declines, exit gracefully

3. Run `git status` to check working directory state:
   - If there are uncommitted changes, show a friendly warning:
     ```
     Note: You have uncommitted changes. These won't be pushed, but you may want to commit them first.
     ```
   - Do NOT block the push — just inform the user

4. Check if branch has an upstream tracking branch:
   ```bash
   git rev-parse --abbrev-ref @{upstream}
   ```
   - If this fails (no upstream), the branch needs to be published for the first time
   - If upstream exists, note the remote branch name

5. Show what will be pushed:
   - If upstream exists:
     ```bash
     git log --oneline @{upstream}..HEAD
     ```
   - If no upstream, show recent commits:
     ```bash
     git log --oneline -5
     ```
   - Display to user: "About to push N commit(s) to <remote>/<branch>"

6. Check for force push flags in `$ARGUMENTS`:
   - If `--force` is present:
     - If on main/master: ABORT with error message:
       ```
       ERROR: Force push to main/master is strictly forbidden.
       This can cause data loss for other contributors.

       If you absolutely need to rewrite main/master history:
       1. Coordinate with your team
       2. Ensure everyone's work is safe
       3. Run the command manually
       ```
       Exit immediately without pushing
     - If on feature branch: Show severe warning:
       ```
       ⚠️  WARNING: Force pushing will rewrite remote history!
       This will affect anyone else working on this branch.

       Safer alternative: use --force-with-lease

       Do you want to continue with --force? (y/n)
       ```
       Wait for confirmation
   - If `--force-with-lease` is present:
     - Show info message:
       ```
       Using --force-with-lease (safer than --force)
       This will only succeed if remote hasn't changed since your last fetch.
       ```

7. Execute the push:
   - If no upstream exists:
     ```bash
     git push -u origin <branch-name> $ARGUMENTS
     ```
   - If upstream exists:
     ```bash
     git push $ARGUMENTS
     ```

8. If push fails:
   - Check common failure reasons and provide specific guidance:
     - "rejected (non-fast-forward)":
       ```
       Your branch is behind the remote. Options:
       1. Pull and rebase: git pull --rebase
       2. Use /sync command to sync with main
       3. Force push (only if you're sure): git push --force-with-lease
       ```
     - "rejected (fetch first)":
       ```
       Remote has commits you don't have locally.
       Run: git fetch origin
       Then: git log HEAD..@{upstream} to see remote commits
       Then: git pull --rebase or /sync
       ```
     - Authentication failure:
       ```
       Authentication failed. Check:
       1. Git credentials are configured
       2. SSH key is added to GitHub/GitLab
       3. Token has push permissions
       ```
   - Exit with failure status

9. If push succeeds:
   - Show success confirmation:
     ```
     ✓ Successfully pushed <branch-name> to <remote>/<remote-branch>
     ```
   - Run and display remote URL:
     ```bash
     git remote get-url origin
     ```
   - Show final branch state:
     ```bash
     git status -sb
     ```
   - If on GitHub/GitLab, suggest creating a PR if this is a feature branch:
     ```
     To create a pull request, run: gh pr create
     Or visit: <remote-url>/compare/<branch-name>
     ```

## Important Notes
- NEVER force push to main/master — block it unconditionally
- ALWAYS show what commits will be pushed before pushing
- DO NOT require clean working directory (uncommitted changes are local-only)
- Use `-u origin <branch>` for first push of new branches
- Prefer `--force-with-lease` over `--force` for safer force pushing
- If user passes `--force` in arguments, intercept and warn strongly
- Accept other git push flags pass-through in `$ARGUMENTS` (e.g., `--no-verify`, `--tags`)
- This command does NOT create PRs — it only pushes to remote
