# CI Secrets Checklist

The following secrets configuration guide applies to GitHub Repository Settings (`Settings > Secrets and variables > Actions`):

## Required
- `GITHUB_TOKEN`: (Automatic) Used for Gitleaks and codeql-action.

## Optional (Not Currently Used)
This project does not currently require additional secrets. If you add external notification integrations in the future, consider:

- `SLACK_WEBHOOK`: For team notifications via Slack (requires updating the `deployment-readiness` job in `ci.yml`).
- `DISCORD_WEBHOOK`: Alternative for team notifications via Discord.

## Best Practices
1. **Least Privilege**: The `GITHUB_TOKEN` already has the minimum required permissions (configured in `ci.yml`).
2. **Rotations**: If you add webhooks or tokens, rotate them every 90 days.
3. **Audit**: Review the Gitleaks reports in the "Security" tab of GitHub to ensure no secrets are accidentally committed.
