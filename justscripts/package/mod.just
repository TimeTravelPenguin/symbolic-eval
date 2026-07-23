VER := `dasel -i toml 'package.version' < typst.toml | tr -d "'"`
PKG_NAME := `dasel -i toml 'package.name' < typst.toml | tr -d "'"`

_default:
  @just --list package

[group("Packaging")]
[doc("Prepares the package for release.")]
[no-cd]
new: init update

[private]
[no-cd]
init:
  #!/usr/bin/env sh
  set -euxo pipefail

  # see: https://github.com/typst/packages/blob/main/docs/tips.md#sparse-checkout-of-the-repository
  git clone --depth 1 --no-checkout --filter="tree:0" git@github.com:typst/packages
  cd packages
  git sparse-checkout init
  git sparse-checkout set packages/preview/{{ PKG_NAME }}
  git checkout main

  # If maintainers change, update the URL below
  git remote add fork "git@github.com:TimeTravelPenguin/packages.git"
  git fetch fork
  git rebase fork/main --onto origin/main

[group("Packaging")]
[doc("Updates the package with the latest changes.")]
[no-cd]
update:
  #!/usr/bin/env sh
  set -euo pipefail

  echo "Preparing package for version: {{ VER }}"
  
  # Check that ./dist/{{ PKG_NAME }} exists. If not, error.
  if [ ! -d "./dist/{{ PKG_NAME }}/{{ VER }}" ]; then
    echo "The directory ./dist/{{ PKG_NAME }}/{{ VER }} does not exist. Please run 'just bundle' to create the package before updating."
    exit 1
  fi

  # Check if origin has a tag matching the version. If not, error.
  if ! git ls-remote --tags origin | grep -q "refs/tags/v{{ VER }}"; then
    echo "The tag v{{ VER }} does not exist in the origin repository. Please create using:"
    echo "  git tag -a \"v{{ VER }}\" -m \"v{{ VER }}\" && git push origin \"v{{ VER }}\""
    exit 1
  fi

  # Checkout fork branch if it exists, otherwise create a new branch
  cd packages
  fork_branch=$(git ls-remote --heads fork "refs/heads/{{ PKG_NAME }}-v{{ VER }}")

  if git show-ref --verify --quiet "refs/heads/{{ PKG_NAME }}-v{{ VER }}"; then
    echo "Local branch {{ PKG_NAME }}-v{{ VER }} exists. Checking out."
    git checkout "{{ PKG_NAME }}-v{{ VER }}"
    git pull --rebase origin main
  elif [ -n "$fork_branch" ]; then
    echo "Branch fork/{{ PKG_NAME }}-v{{ VER }} exists. Checking out."
    git checkout -b "{{ PKG_NAME }}-v{{ VER }}" "fork/{{ PKG_NAME }}-v{{ VER }}"
    git pull --rebase origin main
  else
    echo "Branch fork/{{ PKG_NAME }}-v{{ VER }} does not exist. Creating new branch and checking out."
    git checkout -b "{{ PKG_NAME }}-v{{ VER }}" origin/main
  fi

  cd ..
  # The typst/packages repo nests everything under a top-level `packages/`
  # dir, so the package lives at `<clone>/packages/preview/{{ PKG_NAME }}/<version>`.
  dest="packages/packages/preview/{{ PKG_NAME }}/{{ VER }}"
  rm -rf "$dest"
  mkdir -p "$dest"
  cp -r "dist/{{ PKG_NAME }}/{{ VER }}/." "$dest"

  # Push changes to fork
  cd packages
  git add "packages/preview/{{ PKG_NAME }}/{{ VER }}"
  git commit -m "chore: update {{ PKG_NAME }} package to v{{ VER }}"
  git push -u fork "{{ PKG_NAME }}-v{{ VER }}"

[no-cd]
[confirm("Remove directory `packages`? (y/N)")]
remove:
  rm -rf packages
