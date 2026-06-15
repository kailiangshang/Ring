# PR CI Fixes Plan

## Phases

1. Inspect current PR branch and reproduce reported failures locally.
2. Run Rust formatter and verify `cargo fmt --check`.
3. Fix frontend lint errors without changing intended behavior.
4. Run local verification.
5. Commit and push updates to `origin/feature/optimize-graph`.

## Risks

- Some lint failures may be pre-existing, but CI blocks the whole PR, so they still need resolution unless excluded by config.
- React compiler lint rules can require small state-flow changes rather than superficial formatting.

## Validation

- `cargo fmt --check`
- `npm run lint`
- `npm run build`
- Targeted tests if touched code warrants them.
