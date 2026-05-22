// Library crate root. Both the binary (`src/main.rs`) and the integration
// tests under `tests/` consume this. Keeping everything reachable via the
// library means tests don't need a parallel module tree.

pub mod api;
pub mod packages;
pub mod utils;
