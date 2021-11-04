# Bench Suit

## Intention

This bench suit is intended to compare different library versions and different hardware to run the library on.

## Important things for benching

The rust compiler is in favour to optimize almost everything out if not needed. So to avoid this optimization, results which come from bench functions should be used with the `core::hint::black_box()`. Another thing is, that rust optimizes iterations. So, if a bench test benches a function which uses the same parameters every iteration, the compiler calculates the function once and uses the result for the following iterations. To avoid this, the `black_box()` function can be used as well.

Another thing is, the whole code gets build in `opt-level="s"` which optimizes for code size. This avoids that the compiler does to weird things. But to have a good result of the function which should be tested for speed the `#[optimize(speed)]` attribute can be used on that function.

A short example on how such a test can look:

```Rust
#[optimize(speed)]
// function to test
fn mann(m: usize, n: usize) -> usize {
    match m {
        0 => n + 1,
        _ => match n {
            0 => mann(m - 1, 1),
            n => mann(m - 1, mann(m, n - 1)),
        },
    }
}
```

```Rust
// bench function
fn bench_test<CM: embedded_time::Clock, const N: usize>(
    bencher: &mut Bencher<CM>,
    _: &mut Context,
) {
    bencher.run_with_watch(|w| {
        let n = core::hint::black_box(3);
        w.start();
        core::hint::black_box(mann(n, N));
        w.stop();
    })
}
```