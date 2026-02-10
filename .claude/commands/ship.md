# ship

Complete current work by testing, staging, committing, pushing, and creating a PR in one command.

## Arguments
- `$ARGUMENTS` (optional): Additional context for commit message body or PR description

## Instructions

### Step 1: Verify Branch

1. Run `git branch --show-current` to get current branch name

2. Check if on main or master:
   ```bash
   if [[ "$(git branch --show-current)" == "main" || "$(git branch --show-current)" == "master" ]]; then
     echo "ERROR: Cannot /ship from main/master branch"
   fi
   ```
   - If on main/master, **ABORT** with error:
     ```
     ERROR: You are on the 'main' branch.

     Use /start <description> to create a feature branch first.

     Example: /start add-circuit-breaker
     ```
   - Exit immediately without proceeding

### Step 2: Run Tests

3. Run tests to verify code quality:
   ```bash
   cargo test
   ```
   - If tests fail, **ABORT** with error:
     ```
     ERROR: Tests failed. Cannot ship broken code.

     Fix the failing tests and run /ship again.
     ```
   - Exit immediately without committing
   - If tests pass, display:
     ```
     ✓ All tests passed
     ```

### Step 3: Analyze Changes

4. Run the following commands in parallel to understand what's being shipped:
   - `git status` (NEVER use `-uall` flag)
   - `git diff` to see unstaged changes
   - `git diff --cached` to see already-staged changes
   - `git log --oneline -5` to see recent commit message style
   - `git log --oneline main..HEAD` to see commits on this branch
   - `git diff main...HEAD --stat` to see full diff since diverging from main

5. Analyze all changes (both current uncommitted changes AND previous commits on branch):
   - Identify the overall type: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `ci`, `build`
   - Determine the scope (e.g., `matcher`, `order`, `gateway`, `client`, `bench`, `protocol`)
   - If `$ARGUMENTS` is provided, use it as a hint but verify against the actual diff
   - Consider if there are already commits on this branch — the new commit should complement them

### Step 4: Stage Changes

6. Stage relevant files using specific file names (prefer explicit over `git add -A` or `git add .`):
   - List all modified/new files from `git status`
   - Exclude files that likely contain secrets (.env, credentials.json, etc.)
   - Use `git add <file1> <file2> <file3> ...`

7. Verify what's staged:
   ```bash
   git diff --cached --stat
   ```

### Step 5: Create Commit

8. Draft commit message following conventional commit format:
   ```
   type(scope): concise description in imperative mood

   [optional body explaining what and why, not how]
   [if $ARGUMENTS provided, incorporate it here]

   Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
   ```
   - Keep subject line under 72 characters
   - Use imperative mood ("add" not "added" or "adds")
   - Body should explain motivation and contrast with previous behavior
   - If `$ARGUMENTS` provided, use as context for body

9. Create the commit using HEREDOC for proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): description

   Optional body explaining what and why.

   Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
   EOF
   )"
   ```

10. Verify commit success:
    ```bash
    git log --oneline -3
    ```

### Step 6: Push to Remote

11. Check if remote tracking branch exists:
    ```bash
    git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "no-upstream"
    ```

12. Push to remote:
    - If no upstream exists (first push):
      ```bash
      git push -u origin <branch-name>
      ```
    - If upstream exists:
      ```bash
      git push
      ```

13. If push fails:
    - Show error and suggest solutions:
      ```
      ERROR: Push failed.

      Common solutions:
      1. Pull and rebase: git pull --rebase
      2. Use /sync to sync with main
      3. Force push (only if safe): git push --force-with-lease
      ```
    - Exit without creating PR

14. If push succeeds, display:
    ```
    ✓ Pushed to origin/<branch-name>
    ```

### Step 7: Create Pull Request

15. Analyze ALL commits and changes for PR:
    - Review complete commit history from `git log main..HEAD`
    - Review full diff from `git diff main...HEAD`
    - Consider `$ARGUMENTS` if provided
    - Draft PR title (under 70 characters, conventional commit style)
    - Draft PR body:
      ```markdown
      ## Summary
      - First key change
      - Second key change
      - Third key change

      ## Test plan
      - [x] Run `cargo test` — all tests passed
      - [ ] Run `cargo run --release` to verify benchmark completes
      - [ ] Verify performance metrics (if applicable)

      🤖 Generated with [Claude Code](https://claude.com/claude-code)
      ```

16. Create the PR using `gh pr create` with HEREDOC:
    ```bash
    gh pr create --title "type(scope): PR title" --body "$(cat <<'EOF'
    ## Summary
    - First key change
    - Second key change

    ## Test plan
    - [x] Run `cargo test` — all tests passed
    - [ ] Run `cargo run --release` to verify benchmark completes
    - [ ] Verify performance metrics (if applicable)

    🤖 Generated with [Claude Code](https://claude.com/claude-code)
    EOF
    )"
    ```

17. If PR creation fails:
    - Show error and suggest manual creation:
      ```
      ERROR: Failed to create PR.

      Your code has been committed and pushed successfully.
      Create PR manually at: https://github.com/<user>/<repo>/compare/<branch-name>
      ```
    - Exit with partial success status

### Step 8: Confirm Success

18. Display final summary:
    ```
    ✓ All tests passed
    ✓ Staged N files
    ✓ Created commit: <commit-message-subject>
    ✓ Pushed to origin/<branch-name>
    ✓ Created PR: <PR-URL>

    Your work is shipped! 🚀
    ```

19. Run `git status` to show final state

## Important Notes
- **NEVER** proceed if on main/master branch — user must create feature branch first with `/start`
- **ALWAYS** run tests before committing — broken code cannot be shipped
- Keep commit subject line under 72 characters
- Use imperative mood in commit messages ("add" not "added")
- DO NOT commit files that likely contain secrets (.env, credentials.json, etc.)
- NEVER skip hooks (--no-verify, --no-gpg-sign) unless explicitly requested
- If pre-commit hook fails, fix the issue and run `/ship` again (never --amend)
- Always include `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` in commit
- PR title follows conventional commit format: `type(scope): description`
- Use PR body for details, not the title
- Always base PRs against `main` branch
- This command does NOT merge PRs — that's the user's job
- Tests must pass before commit — no exceptions
- If tests fail, abort immediately and let user fix the issues
