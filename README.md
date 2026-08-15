# Wazi

Wazi is a programming language that compile to WebAssembly (WASM). It aims to make writing WASM module simpler and thus provide a simple syntax.

**Wazi is personal "for fun" project** [![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

```
func main() {
    local result = add(12, 28)
    print_int(result)
}

func add(a: Int, b: Int): Int {
    return a + b
}
```
## Getting Started
```
cargo build
cargo run -- run examples/simple.wz
cargo run -- build examples/simple.wz
```