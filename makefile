
.PHONY: publish publish-macros publish-macro-impl

publish: publish-macros
	cargo publish -p checked-rs

publish-macros: publish-macro-impl
	cargo publish -p checked-rs-macro

publish-macro-impl:
	cargo publish -p checked-rs-macro-impl

