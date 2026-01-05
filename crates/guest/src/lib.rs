wit_bindgen::generate!({
    world: "root",
    path: "wit",
    generate_all,
    generate_unused_types: true,
    async: [
        "-import:contract:built-in/context#foo"
    ]
});

struct Contract;

impl Guest for Contract {
    fn use_foo() -> String {
        contract::built_in::context::foo()
    }
}

export!(Contract);
