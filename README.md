# Wascript

Wascript is a programming language that specifically compile to WebAssembly (WASM) for the Web. The aim of this project is to make writing WASM simpler and thus provides a simple syntax.
[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

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
cargo run -q -- run examples/simple.wz
cargo run -q -- build examples/simple.wz
```