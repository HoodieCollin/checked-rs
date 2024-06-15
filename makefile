
.PHONY: release-patch release-minor release-major publish-all publish-macros publish-macro-impl bump-patch bump-minor bump-major check-dirty

# Command to count the number of changes
CHECK_DIRTY_CMD = expr $(shell git status --porcelain 2>/dev/null | egrep "^(M| M)" | wc -l)

release-patch: bump-patch publish-all

release-minor: bump-minor publish-all

release-major: bump-major publish-all

publish-all: publish-macros
	cargo publish -p checked-rs

publish-macros: publish-macro-impl
	cargo publish -p checked-rs-macros

publish-macro-impl:
	cargo publish -p checked-rs-macro-impl

bump-patch: check-dirty
	cargo set-version -p checked-rs --bump patch
	cargo set-version -p checked-rs-macros --bump patch
	cargo set-version -p checked-rs-macro-impl --bump patch
	gum confirm 'Do you want to commit this change?' && git commit -am "Bump version"

bump-minor: check-dirty
	cargo set-version -p checked-rs --bump minor
	cargo set-version -p checked-rs-macros --bump minor
	cargo set-version -p checked-rs-macro-impl --bump minor
	gum confirm 'Do you want to commit this change?' && git commit -am "Bump version"

bump-major: check-dirty
	cargo set-version -p checked-rs --bump major
	cargo set-version -p checked-rs-macros --bump major
	cargo set-version -p checked-rs-macro-impl --bump major
	gum confirm 'Do you want to commit this change?' && git commit -am "Bump version"

check-dirty:
	@dirty_count=$$( $(CHECK_DIRTY_CMD) ); if [ $$dirty_count -gt 0 ]; then echo 'There are outstanding changes. Please commit or stash them before bumping the version'; exit 1; else echo "Repository is clean"; fi
