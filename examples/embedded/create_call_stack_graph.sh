# needed tools:
# - cargo-call-stack
# - dot

# known errors: BUG? no callees for `i1 ({}*, [0 x i8]*, i32)`
# refere to: https://github.com/japaric/cargo-call-stack#function-pointers

mkdir tmp
cargo +nightly call-stack --bin embedded --target thumbv7em-none-eabihf main > tmp/cg.dot
dot -Tsvg tmp/cg.dot > call_stack_graph.svg