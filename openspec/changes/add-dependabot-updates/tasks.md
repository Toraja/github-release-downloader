## 1. CI And Action Hardening

- [x] 1.1 Separate the `push` and `pull_request` triggers in `.github/workflows/test.yml`, run `Test` for every pull request targeting `main`, retain relevant push-side path filters including `Cargo.toml` and `Cargo.lock`, and verify the workflow parses and its trigger conditions match the design.
- [x] 1.2 Replace external action tags in manually maintained workflows and composite actions with verified full-length commit SHAs plus same-line version comments, and verify every non-local `uses:` reference outside the generated release workflow is immutable.
- [x] 1.3 Keep cargo-dist as the sole owner of `.github/workflows/release.yml`, regenerate it with normal action tags and no manual action-commit map, and verify cargo-dist accepts the generated file without dirty-CI errors.

## 2. Dependabot Policy

- [x] 2.1 Add `.github/dependabot.yml` entries for Cargo and GitHub Actions with weekly schedules, seven-day cooldowns, and separate patch/minor groups, and verify GitHub's Dependabot configuration syntax accepts both ecosystems.
- [x] 2.2 Exclude `.github/workflows/release.yml` from routine GitHub Actions version updates, leave major updates outside compatible groups, and verify the configuration expresses both boundaries without excluding manually maintained workflow or composite-action files.

## 3. Safe Automerge

- [x] 3.1 Add a `pull_request_target` workflow restricted to `dependabot[bot]` in this repository, grant only `contents: write` and `pull-requests: write`, pin `dependabot/fetch-metadata` to a verified full SHA, and verify the workflow never checks out or executes pull-request content.
- [x] 3.2 Use Dependabot metadata to enable native squash auto-merge only when the highest update type is patch or minor, and verify major updates and non-Dependabot pull requests cannot reach the merge command.

## 4. Repository Enforcement

- [x] 4.1 Enable native auto-merge for the GitHub repository and verify the setting is active through the GitHub API or repository settings.
- [x] 4.2 Protect `main` with the `Test` job as a required status check, without requiring human approval or the release workflow, and verify the resulting ruleset or branch-protection response matches that policy.

## 5. Verification

- [x] 5.1 Run `just format`, `just lint`, and `just test`, validate all edited YAML files, and verify the repository's local quality checks pass.
- [ ] 5.2 Trigger or inspect initial Dependabot updates and verify the seven-day cooldown, ecosystem grouping, release-workflow exclusion, SHA maintenance, required `Test` check, patch/minor auto-merge eligibility, and manual handling of major updates are observable as designed.
- [x] 5.3 Record cargo-dist as a quarterly manual maintenance item and verify the documented process requires reviewing release notes, updating `cargo-dist-version`, and regenerating `.github/workflows/release.yml` rather than patching generated output.
