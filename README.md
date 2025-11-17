# About

## Conventional commits
The Rapid project uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) to standardize commit messages.
This helps in maintaining a clear and consistent commit history, making it easier to understand the changes made over time.

## Pre-commit
The pre-commits feature needs a Python and pip environment.
The pip environment is only needed for the automatically install of the pre-commit Python module.

To install the pre-commit module, run the following command:
```bash
pip install pre-commit
```

To install the git hooks defined in the `.pre-commit-config.yaml` file, run:
```bash
pre-commit install
```
