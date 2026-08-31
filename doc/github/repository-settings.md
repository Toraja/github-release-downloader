# Required GitHub Repository Settings

The Dependabot configuration and automerge workflow require the following repository settings.

## Pull Request Merges

Under **Settings > General > Pull Requests**:

- Enable **Allow squash merging**.
- Enable **Allow auto-merge**.

Other merge methods may remain enabled. Dependabot uses squash merging explicitly.

## Main Branch Protection

Protect `main` under **Settings > Branches > Branch protection rules** with these values:

- Enable **Require status checks to pass before merging**.
- Require the `Test` status check.
- Enable **Require branches to be up to date before merging**.
- Apply the rule to administrators.
- Do not require pull request approvals for Dependabot automerge.
- Do not require the cargo-dist `Release` workflow.
- Do not allow force pushes or branch deletion.

The `Test` workflow runs for every pull request targeting `main`. The branch protection rule requires this check for every merge into `main`, regardless of who opened the pull request or whether auto-merge is enabled. If pull-request path filters skip the workflow, GitHub receives no `Test` result and keeps the required check pending, blocking both manual merge and auto-merge. Pushes to `main` remain path-filtered because the required check applies before merging, not after a commit is already on `main`.

## Actions Permissions

Under **Settings > Actions > General > Workflow permissions**:

- Keep the default `GITHUB_TOKEN` permission at **Read repository contents and packages permissions**.
- The **Allow GitHub Actions to create and approve pull requests** option may remain disabled; this project does not auto-approve pull requests.

The Dependabot automerge workflow grants only the job permissions it needs:

- `contents: write`
- `pull-requests: write`

Do not add repository secrets or check out pull request code in this `pull_request_target` workflow.

## Dependabot

Under **Settings > Security > Code security and analysis**:

- Enable the dependency graph.
- Enable Dependabot alerts.
- Enable Dependabot security updates.

Version updates are configured by `.github/dependabot.yml`. Routine Cargo and GitHub Actions updates use a seven-day cooldown and weekly schedule. Security updates bypass the cooldown but still require the `Test` check before merging.

The generated `.github/workflows/release.yml` is excluded from routine Dependabot updates and from automerge. Cargo-dist owns that file; update it by changing `cargo-dist-version` in `dist-workspace.toml` and running `just dist-generate`.

## Verification

Verify merge and protection settings with the GitHub CLI:

```sh
gh api repos/Toraja/github-release-downloader \
  --jq '{allow_auto_merge, allow_squash_merge}'

gh api repos/Toraja/github-release-downloader/branches/main/protection \
  --jq '{strict: .required_status_checks.strict, required_checks: .required_status_checks.contexts, reviews: .required_pull_request_reviews}'

gh api repos/Toraja/github-release-downloader/actions/permissions/workflow
```

The expected values are:

- `allow_auto_merge: true`
- `allow_squash_merge: true`
- `strict: true`
- `required_checks: ["Test"]`
- `reviews: null`
- `default_workflow_permissions: "read"`
- `can_approve_pull_request_reviews: false`

## Sources

- [Managing pull request auto-merge](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-auto-merge-for-pull-requests-in-your-repository)
- [Automatically merging a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request)
- [Managing branch protection rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule)
- [Managing GitHub Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)
- [Using `GITHUB_TOKEN` authentication](https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication)
- [Automating Dependabot with GitHub Actions](https://docs.github.com/en/code-security/dependabot/working-with-dependabot/automating-dependabot-with-github-actions)
- [Dependabot version updates](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/about-dependabot-version-updates)
- [Dependabot configuration options](https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference)
- [GitHub security features](https://docs.github.com/en/code-security/getting-started/github-security-features)
- [Cargo-dist GitHub Actions customization and generated workflow ownership](https://opensource.axo.dev/cargo-dist/book/ci/customizing.html)
