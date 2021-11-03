use super::Bencher;

pub struct Context;

fn mann(m: usize, n: usize) -> usize {
    match m {
        0 => n + 1,
        _ => match n {
            0 => mann(m - 1, 1),
            n => mann(m - 1, mann(m, n - 1)),
        },
    }
}

pub(crate) fn bench_test<CM: embedded_time::Clock>(bencher: &mut Bencher<CM>, _: &mut Context) {
    bencher.run(|| mann(3, 5))
}
