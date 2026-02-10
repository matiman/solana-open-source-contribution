# pr

Push the current branch and create a GitHub pull request.

## Arguments
- `$ARGUMENTS` (optional): Additional description or context for the PR body

## Instructions

1. Run the following commands in parallel to understand the branch state:
   - `git status` (NEVER use `-uall` flag)
   - `git diff` to see unstaged changes
   - `git diff --cached` to see staged changes
   - `git branch --show-current` to get current branch name
   - `git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "no-upstream"` to check if remote tracking branch exists
   - `git log --oneline main..HEAD` to see all commits since diverging from main
   - `git diff main...HEAD --stat` to see the full diff since diverging from main

2. If there are uncommitted changes (staged or unstaged):
   - Warn the user
   - Ask if they want to commit first (suggest using `/commit` command)
   - Do not proceed until working directory is clean

3. Analyze ALL commits and changes that will be included in the PR:
   - Review the complete commit history from `git log main..HEAD`
   - Review the full diff from `git diff main...HEAD`
   - Consider `$ARGUMENTS` if provided for additional context
   - Draft a PR title (under 70 characters, follows conventional commit style)
   - Draft a PR body with:
     - **Summary**: 1-3 bullet points explaining what changed and why
     - **Test plan**: How to verify the changes (reference build/test commands from CLAUDE.md)
     - If `$ARGUMENTS` provided, incorporate it into the description

4. Check if remote tracking branch exists. If not, push with `-u` flag:
   ```bash
   git push -u origin <branch-name>
   ```
   If remote tracking branch exists, just push:
   ```bash
   git push
   ```

5. Create the PR using `gh pr create` with a HEREDOC:
   ```bash
   gh pr create --title "type(scope): PR title" --body "$(cat <<'EOF'
   ## Summary
   - First key change
   - Second key change
   - Third key change

   ## Test plan
   - [ ] Run `cargo test` to verify tests pass
   - [ ] Run `cargo run --release` to verify benchmark completes
   - [ ] Verify performance metrics (if applicable)

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

6. Return the PR URL so the user can view it

## Important Notes
- NEVER merge PRs into main — that's the user's job
- This command only CREATES pull requests
- Always base PRs against the `main` branch
- PR title should follow conventional commit format: `type(scope): description`
- Use the description/body for details, not the title
- Include relevant testing commands from CLAUDE.md in the test plan
- DO NOT push to main/master branches
- If asked to force push, warn the user unless it's explicitly requested
