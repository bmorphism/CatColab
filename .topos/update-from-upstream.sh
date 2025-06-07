#!/usr/bin/env bash
set -euo pipefail

REMOTE="topos"
BRANCH="main"

if ! git remote | grep -q "^${REMOTE}$"; then
  git remote add ${REMOTE} https://github.com/ToposInstitute/CatColab.git
fi

git fetch ${REMOTE}
git merge --ff-only ${REMOTE}/${BRANCH}
