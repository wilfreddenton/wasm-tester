wit_bindgen::generate!({
    world: "root",
    path: "wit",
    generate_all,
    generate_unused_types: true,
});

struct Contract;

impl Guest for Contract {
    fn use_foo() -> String {
        contract::built_in::context::foo()
    }
}

export!(Contract);
