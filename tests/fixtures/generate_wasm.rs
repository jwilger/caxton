//! Generate proper WASM test fixtures

fn main() {
    // Minimal valid WASM module
    let minimal = wat::parse_str(r#"
        (module
            (memory (export "memory") 1)
        )
    "#).unwrap();
    std::fs::write("minimal_agent.wasm", &minimal).unwrap();
    std::fs::write("test_agent.wasm", &minimal).unwrap();
    std::fs::write("valid_agent.wasm", &minimal).unwrap();
    std::fs::write("cooperative_agent.wasm", &minimal).unwrap();
    std::fs::write("resource_test.wasm", &minimal).unwrap();

    // Memory hog - allocates lots of memory
    let memory_hog = wat::parse_str(r#"
        (module
            (memory (export "memory") 100)
            (func (export "allocate_memory"))
        )
    "#).unwrap();
    std::fs::write("memory_hog.wasm", &memory_hog).unwrap();

    // Infinite loop module
    let infinite_loop = wat::parse_str(r#"
        (module
            (memory (export "memory") 1)
            (func (export "infinite_loop")
                (loop br 0)
            )
        )
    "#).unwrap();
    std::fs::write("infinite_loop.wasm", &infinite_loop).unwrap();

    // Host function test
    let host_test = wat::parse_str(r#"
        (module
            (import "env" "log" (func $log (param i32 i32)))
            (import "env" "get_time" (func $get_time (result i64)))
            (import "env" "send_message" (func $send_message (param i32 i32 i32) (result i32)))

            (memory (export "memory") 1)

            (func (export "test"))
        )
    "#).unwrap();
    std::fs::write("host_function_test.wasm", &host_test).unwrap();

    println!("Generated all WASM test fixtures!");
}
