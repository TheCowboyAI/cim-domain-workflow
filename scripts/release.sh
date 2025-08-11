#!/bin/bash

# Release script for cim-domain-workflow
# Usage: ./scripts/release.sh [major|minor|patch]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the release type
RELEASE_TYPE=${1:-patch}

# Validate release type
if [[ ! "$RELEASE_TYPE" =~ ^(major|minor|patch)$ ]]; then
    echo -e "${RED}Error: Invalid release type. Use major, minor, or patch${NC}"
    exit 1
fi

echo -e "${GREEN}Starting $RELEASE_TYPE release...${NC}"

# Ensure we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${RED}Error: Must be on main branch to release. Current branch: $CURRENT_BRANCH${NC}"
    exit 1
fi

# Ensure working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Working directory is not clean. Commit or stash changes first.${NC}"
    exit 1
fi

# Pull latest changes
echo "Pulling latest changes..."
git pull origin main

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
cargo test

# Run clippy
echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy -- -D warnings

# Check formatting
echo -e "${YELLOW}Checking formatting...${NC}"
cargo fmt --check

# Build docs
echo -e "${YELLOW}Building documentation...${NC}"
cargo doc --no-deps

# Get current version
CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Calculate new version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

case $RELEASE_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo -e "New version: ${GREEN}$NEW_VERSION${NC}"

# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update version in README if present
if grep -q "cim-domain-workflow = \"$CURRENT_VERSION\"" README.md; then
    sed -i "s/cim-domain-workflow = \"$CURRENT_VERSION\"/cim-domain-workflow = \"$NEW_VERSION\"/g" README.md
fi

# Update CHANGELOG
echo -e "${YELLOW}Updating CHANGELOG...${NC}"
DATE=$(date +%Y-%m-%d)
sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $DATE/" CHANGELOG.md

# Add comparison link
REPO_URL="https://github.com/thecowboyai/cim-domain-workflow"
sed -i "s|\[Unreleased\]: .*|\[Unreleased\]: $REPO_URL/compare/v$NEW_VERSION...HEAD\n[$NEW_VERSION]: $REPO_URL/compare/v$CURRENT_VERSION...v$NEW_VERSION|" CHANGELOG.md

# Commit version bump
echo -e "${YELLOW}Committing version bump...${NC}"
git add Cargo.toml README.md CHANGELOG.md
git commit -m "chore: release v$NEW_VERSION"

# Create and push tag
echo -e "${YELLOW}Creating tag v$NEW_VERSION...${NC}"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

# Push changes
echo -e "${YELLOW}Pushing changes...${NC}"
git push origin main
git push origin "v$NEW_VERSION"

echo -e "${GREEN}✓ Release v$NEW_VERSION completed!${NC}"
echo ""
echo "Next steps:"
echo "1. GitHub Actions will automatically create a draft release"
echo "2. Review and publish the release on GitHub"
echo "3. The crate will be automatically published to crates.io"