# CatColab Fork Guide

This directory documents how this repository diverges from the upstream
[ToposInstitute/CatColab](https://github.com/ToposInstitute/CatColab) project.
It collects auxiliary scripts and notes so that updates from upstream can be
managed with minimal friction.

## Differences from upstream

The main branch of this repository tracks `ToposInstitute/CatColab:main` with a
few additional developer conveniences:

- Flox environment files under `.flox/` for deterministic tooling.
- A `justfile` and `justfile-tests.yml` workflow for quick project setup and
  CI checks.
- Local notes in `CLAUDE.md`.

Keeping these files separate helps reduce merge conflicts when pulling in
changes from upstream.

## Updating from upstream

Run `./.topos/update-from-upstream.sh` to fetch and merge the latest commits from
`ToposInstitute/CatColab`. The script will add the `topos` remote if it does not
already exist.

```
# update local main branch
./.topos/update-from-upstream.sh
```

After merging, resolve any conflicts, run the project checks, and commit as
usual.
