use std::{fs, path::Path};

use anyhow::{Context, Result};
use wasmtime::{
    Engine, Store,
    component::{Accessor, Component, HasData, Linker, Val},
};
use wit_component::ComponentEncoder;

wasmtime::component::bindgen!({
    path: "wit",
    imports: {
        default: trappable,
    }
});

#[derive(Clone)]
pub struct Runtime {
    pub engine: Engine,
    pub linker: Linker<Self>,
}

impl Runtime {
    pub fn new_engine() -> Result<Engine> {
        let mut config = wasmtime::Config::new();
        config.async_support(true);
        config.wasm_component_model_async(true);
        config.consume_fuel(true);
        // Ensure deterministic execution
        config.wasm_threads(false);
        config.wasm_relaxed_simd(false);
        config.cranelift_nan_canonicalization(true);
        Engine::new(&config)
    }

    pub fn new_linker(engine: &Engine) -> Result<Linker<Self>> {
        let mut linker = Linker::new(engine);
        Root::add_to_linker::<_, Self>(&mut linker, |s| s)?;
        Ok(linker)
    }

    pub fn make_store(&self, fuel: u64) -> Result<Store<Runtime>> {
        let mut s = Store::new(&self.engine, self.clone());
        s.set_fuel(fuel)?;
        Ok(s)
    }

    pub async fn new() -> Result<Self> {
        let engine = Self::new_engine()?;
        let linker = Self::new_linker(&engine)?;
        Ok(Self { engine, linker })
    }

    pub async fn execute(&self, module_bytes: &[u8]) -> Result<()> {
        let component_bytes = ComponentEncoder::default()
            .module(module_bytes)
            .context("Failed to parse module bytes")?
            .validate(true)
            .encode()
            .context("Failed to encode component")?;
        let component = Component::from_binary(&self.engine, &component_bytes)
            .context("Failed to create component")?;
        let mut store = self.make_store(100000)?;
        let instance = self
            .linker
            .instantiate_async(&mut store, &component)
            .await?;
        let func = instance
            .get_func(&mut store, "use-foo")
            .expect("func use-foo not found");
        let mut results = [Val::String("".to_string())];
        func.call_async(store, &[], &mut results).await?;
        println!("Result: {:?}", results[0]);
        Ok(())
    }
}

impl HasData for Runtime {
    type Data<'a> = &'a mut Runtime;
}

impl contract::built_in::context::Host for Runtime {}
impl contract::built_in::context::HostWithStore for Runtime {
    async fn foo<T>(_accessor: &Accessor<T, Self>) -> Result<String> {
        Ok("foo".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let path = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-unknown-unknown/debug/guest.wasm");
    if !path.exists() {
        return Err(anyhow::anyhow!("guest.wasm not found"));
    }
    let bs = fs::read(path)?;
    let runtime = Runtime::new().await?;
    runtime.execute(&bs).await?;
    Ok(())
}
