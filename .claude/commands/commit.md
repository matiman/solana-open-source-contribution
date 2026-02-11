# commit

Stage relevant changes and create a well-crafted conventional commit message.

## Arguments
- `$ARGUMENTS` (optional): Hint about what changed (e.g., "added logging", "fixed price validation")

## Instructions

1. Run `git status` to see all untracked and modified files (NEVER use `-uall` flag)

2. Run `git diff` to see unstaged changes

3. Run `git diff --cached --stat` to see what's already staged (if anything)

4. Run `git log --oneline -5` to see recent commit message style

5. Analyze the changes:
   - Identify the type: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `chore`, `ci`, `build`
   - Determine the scope (e.g., `matcher`, `order`, `gateway`, `client`, `bench`)
   - If `$ARGUMENTS` is provided, use it as a hint but verify against the actual diff
   - Draft a commit message following this format:
     ```
     type(scope): concise description in imperative mood

     [optional body explaining what and why, not how]

     Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
     ```

6. Stage the relevant files using `git add <file1> <file2> ...` (prefer specific files over `git add -A` or `git add .`)

7. Create the commit using a HEREDOC for proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): description

   Optional body explaining what and why.

   Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
   EOF
   )"
   ```

8. After commit completes, run `git log --oneline -3` to verify success

9. Run `git status` to show final state

## Important Notes
- Keep the subject line under 72 characters
- Use imperative mood ("add" not "added" or "adds")
- DO NOT commit files that likely contain secrets (.env, credentials.json, etc.)
- NEVER skip hooks (--no-verify, --no-gpg-sign)
- If pre-commit hook fails, fix the issue and create a NEW commit (never --amend unless explicitly requested)
- Always include `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` in the commit message
