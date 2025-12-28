wit_bindgen::generate!({
    world: "root",
    path: "wit",
    generate_all,
    generate_unused_types: true,
});

struct Contract;

impl Guest for Contract {
    async fn use_foo() -> String {
        contract::built_in::context::foo().await
    }
}

export!(Contract);
