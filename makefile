
.PHONY: publish publish-macros publish-macro-impl dry-run dry-run-macros dry-run-macro-impl

publish: publish-macros
	cargo publish -p checked-rs

publish-macros: publish-macro-impl
	cargo publish -p checked-rs-macros

publish-macro-impl:
	cargo publish -p checked-rs-macro-impl

dry-run: dry-run-macros
	cargo publish --dry-run -p checked-rs

dry-run-macros: dry-run-macro-impl
	cargo publish --dry-run -p checked-rs-macros

dry-run-macro-impl:
	cargo publish --dry-run -p checked-rs-macro-impl