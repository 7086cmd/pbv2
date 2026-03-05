use crate::engines::Engine;

pub struct BuiltinEngine;

#[async_trait::async_trait]
impl Engine for BuiltinEngine {
    fn name() -> &'static str {
        "builtin"
    }

    async fn compile_once(_addr: std::path::PathBuf) -> anyhow::Result<std::path::PathBuf> {
        unimplemented!(
            "The builtin engine is not implemented yet. Please use a specific engine like XeLaTeX."
        )
    }
}
