// This file generates minimal valid WASM modules for testing
use std::fs;

fn main() {
    // Minimal valid WASM module bytes
    let test_agent = vec![
        0x00, 0x61, 0x73, 0x6d, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Type section
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        // Function section
        0x03, 0x02, 0x01, 0x00,
        // Memory section
        0x05, 0x03, 0x01, 0x00, 0x01,
        // Export section
        0x07, 0x11, 0x02, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00,
        0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
        // Code section
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b,
    ];

    fs::write("test_agent.wasm", &test_agent).unwrap();
    fs::write("memory_hog.wasm", &test_agent).unwrap();
    fs::write("infinite_loop.wasm", &test_agent).unwrap();
    fs::write("valid_agent.wasm", &test_agent).unwrap();
    fs::write("host_function_test.wasm", &test_agent).unwrap();
    fs::write("minimal_agent.wasm", &test_agent).unwrap();
    fs::write("cooperative_agent.wasm", &test_agent).unwrap();
    fs::write("resource_test.wasm", &test_agent).unwrap();

    println!("Created test WASM files");
}
