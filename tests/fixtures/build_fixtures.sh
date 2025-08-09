#!/bin/bash

echo "Building test WASM fixtures..."

# Test agent module
cat > test_agent.wat << 'EOF'
(module
    (func $hello (result i32)
        i32.const 42)
    (export "hello" (func $hello))
    (memory (export "memory") 1)
)
EOF
wat2wasm test_agent.wat -o test_agent.wasm

# Memory hog module
cat > memory_hog.wat << 'EOF'
(module
    (func $allocate_memory
        (local i32)
        i32.const 0
        (local.set 0)
        (loop
            (local.get 0)
            i32.const 1000000
            i32.lt_s
            (if
                (then
                    (local.get 0)
                    i32.const 1
                    i32.add
                    (local.set 0)
                    (br 1)
                )
            )
        )
    )
    (export "allocate_memory" (func $allocate_memory))
    (memory (export "memory") 1 100)
)
EOF
wat2wasm memory_hog.wat -o memory_hog.wasm

# Infinite loop module
cat > infinite_loop.wat << 'EOF'
(module
    (func $infinite_loop
        (loop
            (br 0)
        )
    )
    (export "infinite_loop" (func $infinite_loop))
    (memory (export "memory") 1)
)
EOF
wat2wasm infinite_loop.wat -o infinite_loop.wasm

# Valid agent module
cat > valid_agent.wat << 'EOF'
(module
    (func $process (param i32) (result i32)
        local.get 0
        i32.const 2
        i32.mul)
    (export "process" (func $process))
    (memory (export "memory") 1)
)
EOF
wat2wasm valid_agent.wat -o valid_agent.wasm

# Host function test module
cat > host_function_test.wat << 'EOF'
(module
    (import "env" "log" (func $log (param i32 i32)))
    (import "env" "get_time" (func $get_time (result i64)))
    (import "env" "send_message" (func $send_message (param i32 i32 i32) (result i32)))

    (func $test_host_functions
        i32.const 0
        i32.const 10
        call $log

        call $get_time
        drop

        i32.const 1
        i32.const 0
        i32.const 5
        call $send_message
        drop
    )
    (export "test" (func $test_host_functions))
    (memory (export "memory") 1)
)
EOF
wat2wasm host_function_test.wat -o host_function_test.wasm

# Minimal agent module
cat > minimal_agent.wat << 'EOF'
(module
    (func $noop)
    (export "start" (func $noop))
    (memory (export "memory") 1)
)
EOF
wat2wasm minimal_agent.wat -o minimal_agent.wasm

# Cooperative agent module
cat > cooperative_agent.wat << 'EOF'
(module
    (func $long_computation (result i32)
        (local i32)
        i32.const 0
        (local.set 0)
        (block
            (loop
                (local.get 0)
                i32.const 1000
                i32.ge_s
                (br_if 1)

                (local.get 0)
                i32.const 1
                i32.add
                (local.set 0)
                (br 0)
            )
        )
        (local.get 0)
    )
    (export "long_computation" (func $long_computation))
    (memory (export "memory") 1)
)
EOF
wat2wasm cooperative_agent.wat -o cooperative_agent.wasm

# Resource test module
cat > resource_test.wat << 'EOF'
(module
    (func $use_resources (param i32) (result i32)
        (local i32)
        i32.const 0
        (local.set 1)
        (block
            (loop
                (local.get 1)
                (local.get 0)
                i32.ge_s
                (br_if 1)

                (local.get 1)
                i32.const 1
                i32.add
                (local.set 1)
                (br 0)
            )
        )
        (local.get 1)
    )
    (export "use_resources" (func $use_resources))
    (memory (export "memory") 1 10)
)
EOF
wat2wasm resource_test.wat -o resource_test.wasm

echo "Test fixtures built successfully!"
rm *.wat
