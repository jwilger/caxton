# Test Code Formatting Page

This page tests various code block formatting scenarios to ensure proper display in Jekyll documentation.

## Inline Code

This is `inline code` within a paragraph to test inline formatting.

## Simple Code Block

```javascript
function helloWorld() {
  console.log("Hello, World!");
  return true;
}
```

## Multi-line Code Block with Long Lines

```python
def complex_function_with_very_long_parameter_names(first_parameter_with_a_really_long_name, second_parameter_that_is_even_longer, third_parameter_that_continues_the_pattern):
    """This is a docstring that explains what this function does in great detail."""
    result = first_parameter_with_a_really_long_name + second_parameter_that_is_even_longer
    print(f"The result of the calculation is: {result}")
    return result * third_parameter_that_continues_the_pattern
```

## Code Block with Syntax Highlighting

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }
}
```

## YAML Configuration Example

```yaml
server:
  host: localhost
  port: 8080
  ssl:
    enabled: true
    certificate: /path/to/cert.pem
    key: /path/to/key.pem

database:
  type: postgresql
  connection:
    host: db.example.com
    port: 5432
    database: myapp
    username: appuser
    password: ${DB_PASSWORD}
```

## Shell Commands

```bash
# Install Caxton
cargo install caxton

# Run with custom configuration
caxton server --config ./custom-config.yaml --port 9090

# Deploy an agent
caxton agent deploy --name calculator --wasm ./agents/calculator.wasm
```

## JSON API Response

```json
{
  "status": "success",
  "data": {
    "agents": [
      {
        "id": "agent-001",
        "name": "calculator",
        "status": "running",
        "metrics": {
          "requests": 12345,
          "errors": 23,
          "latency_ms": 45
        }
      }
    ]
  },
  "timestamp": "2024-08-07T10:30:00Z"
}
```

## Testing Edge Cases

### Very Long Single Line

```
This is a very long single line of text that should not wrap but instead should show a horizontal scrollbar when it exceeds the width of the container element in the documentation layout.
```

### Empty Code Block

```
```

### Code with Special Characters

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test & Demo</title>
    <style>
        .code { background: #f0f0f0; }
        pre > code { white-space: pre-wrap; }
    </style>
</head>
<body>
    <h1>Testing &lt;code&gt; blocks</h1>
</body>
</html>
```
