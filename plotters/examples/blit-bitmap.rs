use plotters::prelude::*;

use image::{imageops::FilterType, ImageFormat};

use std::fs::File;
use std::io::BufReader;

const OUT_FILE_NAME: &'static str = "plotters-doc-data/blit-bitmap.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new(OUT_FILE_NAME, (1024, 768));
    let root = backend.to_drawing_area();
    root.fill(&mut backend, &WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Bitmap Example", ("sans-serif", 30))
        .margin(5)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(&mut backend, 0.0..1.0, 0.0..1.0)?;

    chart.configure_mesh().disable_mesh().draw(&mut backend)?;

    let (w, h) = chart.plotting_area().dim_in_pixel();
    let image = image::load(
        BufReader::new(
            File::open("plotters-doc-data/cat.png").map_err(|e| {
                eprintln!("Unable to open file plotters-doc-data.png, please make sure you have clone this repo with --recursive");
                e
            })?),
        ImageFormat::Png,
    )?
    .resize_exact(w - w / 10, h - h / 10, FilterType::Nearest);

    let elem: BitMapElement<_> = ((0.05, 0.95), image).into();

    chart.draw_series(&mut backend, std::iter::once(elem))?;
    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present(&mut backend).expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
