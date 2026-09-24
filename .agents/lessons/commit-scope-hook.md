# Commit scope hook

DomainForge requires `<type>(<scope>): <subject>` commit messages. An unscoped
message such as `docs: update guide` fails the `commit-msg` hook. Use a
kebab-case scope, for example `docs(repo): update guide`. The hook runner now
prints this example when commitlint rejects a message.
