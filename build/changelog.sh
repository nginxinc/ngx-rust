#!/usr/bin/env bash

# This script generates a changelog for the current version of the project.

set -o errexit  # abort on nonzero exit status
set -o nounset  # abort on unbound variable
set -o pipefail # don't hide errors within pipes

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
EXCLUDED_COMMIT_TYPES="ci|chore"

pushd . > /dev/null
cd "${SCRIPT_DIR}/.."

if command -v ggrep > /dev/null; then
  GREP=ggrep
else
  GREP=grep
fi
if command -v gsed > /dev/null; then
  SED=gsed
else
  SED=sed
fi

# if gh is installed, use it to pull the last version number
if command -v gh > /dev/null; then
  LAST_RELEASE="$(gh release list --exclude-drafts --exclude-pre-releases --limit 1 | ${GREP} -E 'v[0-9]+\.[0-9]+\.[0-9]+' | cut -f1 | ${GREP} -v "${VERSION}" || true)"
else
  LAST_RELEASE="$(git tag --list v* | ${GREP} -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort --version-sort --field-separator=. --reverse | ${GREP} -v "${VERSION}" | head -n1 || true)"
fi

if [ -z "${LAST_RELEASE}" ]; then
  echo "## Initial release ${VERSION}"
  git log --format="%s	(%h)" | \
    ${GREP} -E -v "^(${EXCLUDED_COMMIT_TYPES}): .*" | \
    ${SED} 's/: /:\t/g1' | \
    column -s "	" -t | \
    ${SED} -e 's/^/ * /'
else
  LAST_RELEASE_HASH="$(git show --format=%H "${LAST_RELEASE}" | head -n1 | ${SED} -e 's/^tag //')"

  echo "## Changes between ${LAST_RELEASE} [$LAST_RELEASE_HASH] and ${VERSION}:"
  git log --format="%s	(%h)" "${LAST_RELEASE_HASH}..HEAD" | \
    ${GREP} -E -v "^(${EXCLUDED_COMMIT_TYPES}): .*" | \
    ${SED} 's/: /:\t/g1' | \
    column -s "	" -t | \
    ${SED} -e 's/^/ * /'
fi

echo ""
popd > /dev/null