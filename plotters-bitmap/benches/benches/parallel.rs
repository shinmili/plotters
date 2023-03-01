use criterion::{criterion_group, BenchmarkId, Criterion};

use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use rayon::prelude::*;

const SIZES: &'static [u32] = &[100, 400, 800, 1000, 2000];

fn draw_plot(backend: &mut BitMapBackend, root: &DrawingArea<Shift>, pow: f64) {
    let mut chart = ChartBuilder::on(root)
        .caption(format!("y = x^{}", pow), ("Arial", 30))
        .build_cartesian_2d(backend, -1.0..1.0, -1.0..1.0)
        .unwrap();

    chart.configure_mesh().draw(backend).unwrap();

    chart
        .draw_series(
            backend,
            LineSeries::new(
                (-50..=50)
                    .map(|x| x as f64 / 50.0)
                    .map(|x| (x, x.powf(pow))),
                &RED,
            ),
        )
        .unwrap()
        .label(format!("y = x^{}", pow))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw(backend)
        .unwrap();
}

fn draw_func_1x1_seq(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_func_1x1");

    for size in SIZES {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &s| {
            let mut buffer = vec![0; (s * s * 3) as usize];
            b.iter(|| {
                let mut backend = BitMapBackend::with_buffer(&mut buffer, (s, s));
                let root = backend.to_drawing_area();
                root.fill(&mut backend, &WHITE).unwrap();
                draw_plot(&mut backend, &root, 2.0);
            })
        });
    }
}

fn draw_func_4x4(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_func_4x4");

    for size in SIZES {
        group
            .bench_with_input(BenchmarkId::new("sequential", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                b.iter(|| {
                    let mut backend = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    let root = backend.to_drawing_area();
                    let areas = root.split_evenly((4, 4));
                    areas
                        .iter()
                        .for_each(|area| draw_plot(&mut backend, &area, 2.0));
                })
            })
            .bench_with_input(BenchmarkId::new("blit", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                let mut element_buffer = vec![vec![0; (s * s / 4 * 3) as usize]; 4];
                b.iter(|| {
                    let mut backend = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    let root = backend.to_drawing_area();
                    let areas = root.split_evenly((4, 4));
                    let elements: Vec<_> = element_buffer
                        .par_iter_mut()
                        .map(|b| {
                            let mut e = BitMapElement::with_mut((0, 0), (s / 2, s / 2), b).unwrap();
                            let drawing_area = e.as_bitmap_backend().to_drawing_area();
                            draw_plot(&mut e.as_bitmap_backend(), &drawing_area, 2.0);
                            e
                        })
                        .collect();

                    areas
                        .into_iter()
                        .zip(elements.into_iter())
                        .for_each(|(a, e)| a.draw(&mut backend, &e).unwrap());
                })
            })
            .bench_with_input(BenchmarkId::new("inplace-blit", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                let mut element_buffer = vec![vec![vec![0; (s * s / 4 * 3) as usize]; 2]; 2];
                b.iter(|| {
                    let mut back = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    back.split(&[s / 2])
                        .into_iter()
                        .zip(element_buffer.iter_mut())
                        .collect::<Vec<_>>()
                        .into_par_iter()
                        .for_each(|(mut back, buffer)| {
                            let root = back.to_drawing_area();
                            let areas = root.split_evenly((1, 2));

                            let elements: Vec<_> = buffer
                                .par_iter_mut()
                                .map(|b| {
                                    let mut e =
                                        BitMapElement::with_mut((0, 0), (s / 2, s / 2), b).unwrap();
                                    let drawing_area = e.as_bitmap_backend().to_drawing_area();
                                    draw_plot(&mut e.as_bitmap_backend(), &drawing_area, 2.0);
                                    e
                                })
                                .collect();

                            areas
                                .into_iter()
                                .zip(elements.into_iter())
                                .for_each(|(a, e)| a.draw(&mut back, &e).unwrap())
                        });
                })
            });
    }
}

fn draw_func_2x1(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_func_2x1");

    for size in SIZES {
        group
            .bench_with_input(BenchmarkId::new("blit", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                let mut element_buffer = vec![vec![0; (s * s / 2 * 3) as usize]; 2];
                b.iter(|| {
                    let mut backend = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    let root = backend.to_drawing_area();
                    let areas = root.split_evenly((2, 1));
                    let elements: Vec<_> = element_buffer
                        .par_iter_mut()
                        .map(|buf| {
                            let mut element =
                                BitMapElement::with_mut((0, 0), (s, s / 2), buf).unwrap();
                            let drawing_area = element.as_bitmap_backend().to_drawing_area();
                            draw_plot(&mut element.as_bitmap_backend(), &drawing_area, 2.0);
                            element
                        })
                        .collect();

                    areas
                        .into_iter()
                        .zip(elements.into_iter())
                        .for_each(|(a, e)| a.draw(&mut backend, &e).unwrap());
                })
            })
            .bench_with_input(BenchmarkId::new("inplace", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                b.iter(|| {
                    let mut back = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    back.split(&[s / 2]).into_par_iter().for_each(|mut b| {
                        let drawing_area = b.to_drawing_area();
                        draw_plot(&mut b, &drawing_area, 2.0)
                    });
                })
            })
            .bench_with_input(BenchmarkId::new("sequential", size), size, |b, &s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                b.iter(|| {
                    let mut backend = BitMapBackend::with_buffer(&mut buffer, (s, s));
                    let root = backend.to_drawing_area();
                    root.split_evenly((2, 1))
                        .iter_mut()
                        .for_each(|area| draw_plot(&mut backend, area, 2.0));
                })
            });
    }
}

criterion_group! {
    name = parallel_group;
    config = Criterion::default().sample_size(10);
    targets =
        draw_func_1x1_seq,
        draw_func_4x4,
        draw_func_2x1,
}
