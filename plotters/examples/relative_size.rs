use plotters::coord::Shift;
use plotters::prelude::*;

fn draw_chart(backend: &mut BitMapBackend, root: &DrawingArea<Shift>) -> DrawResult<()> {
    let mut chart = ChartBuilder::on(root)
        .caption(
            "Relative Size Example",
            ("sans-serif", (5).percent_height()),
        )
        .x_label_area_size((10).percent_height())
        .y_label_area_size((10).percent_width())
        .margin(5)
        .build_cartesian_2d(backend, -5.0..5.0, -1.0..1.0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", (3).percent_height()))
        .draw(backend)?;

    chart.draw_series(
        backend,
        LineSeries::new(
            (0..1000)
                .map(|x| x as f64 / 100.0 - 5.0)
                .map(|x| (x, x.sin())),
            &RED,
        ),
    )?;
    Ok(())
}

const OUT_FILE_NAME: &'static str = "plotters-doc-data/relative_size.png";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new(OUT_FILE_NAME, (1024, 768));
    let root = backend.to_drawing_area();

    root.fill(&mut backend, &WHITE)?;

    let (left, right) = root.split_horizontally((70).percent_width());

    draw_chart(&mut backend, &left)?;

    let (upper, lower) = right.split_vertically(300);

    draw_chart(&mut backend, &upper)?;
    draw_chart(&mut backend, &lower)?;
    let root = root.shrink((200, 200), (150, 100));
    draw_chart(&mut backend, &root)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present(&mut backend).expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
