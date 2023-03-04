use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "plotters-doc-data/twoscale.png";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new(OUT_FILE_NAME, (1024, 768));
    let root = backend.to_drawing_area();
    root.fill(&mut backend, &WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(5)
        .caption("Dual Y-Axis Example", ("sans-serif", 50.0).into_font())
        .build_cartesian_2d(&mut backend, 0f32..10f32, (0.1f32..1e10f32).log_scale())?
        .set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_desc("Log Scale")
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw(&mut backend)?;

    chart
        .configure_secondary_axes()
        .y_desc("Linear Scale")
        .draw(&mut backend)?;

    chart
        .draw_series(
            &mut backend,
            LineSeries::new(
                (0..=100).map(|x| (x as f32 / 10.0, (1.02f32).powf(x as f32 * x as f32 / 10.0))),
                &BLUE,
            ),
        )?
        .label("y = 1.02^x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .draw_secondary_series(
            &mut backend,
            LineSeries::new(
                (0..=100).map(|x| (x as f32 / 10.0, (x as f32 / 5.0).sin())),
                &RED,
            ),
        )?
        .label("y = sin(2x)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&RGBColor(128, 128, 128))
        .draw(&mut backend)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present(&mut backend).expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
