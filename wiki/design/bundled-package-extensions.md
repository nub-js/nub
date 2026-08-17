# Bundled package extensions

Bundled package extensions repair known manifest omissions during resolution. User extensions take precedence, and an existing dependency declaration is never overwritten.

Nub and pnpm identities receive a supplemental catalog from `vendor/package-extensions/unified.json`, alongside the engine's existing compatibility database and the shared phantom-derived snapshot in `crates/nub-cli/assets/bundled-package-extensions.json`. Shared snapshot fields take precedence over supplemental entries. Other identities retain both existing databases without the supplemental map. The `ignore-compatibility-db` setting disables all bundled sources.

Bundled entries stay outside the user-config checksum. The supplemental map is loaded by [[crates/nub-cli/src/pm_engine/mod.rs#bundled_package_extensions_defaults]].
