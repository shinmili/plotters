use plotters::coord::Shift;
use plotters::prelude::*;

pub fn sierpinski_carpet(
    depth: u32,
    drawing_backend: &mut BitMapBackend,
    drawing_area: &DrawingArea<Shift>,
) -> Result<(), Box<dyn std::error::Error>> {
    if depth > 0 {
        let sub_areas = drawing_area.split_evenly((3, 3));
        for (idx, sub_area) in (0..).zip(sub_areas.iter()) {
            if idx != 4 {
                sub_area.fill(drawing_backend, &BLUE)?;
                sierpinski_carpet(depth - 1, drawing_backend, sub_area)?;
            } else {
                sub_area.fill(drawing_backend, &WHITE)?;
            }
        }
    }
    Ok(())
}

const OUT_FILE_NAME: &'static str = "plotters-doc-data/sierpinski.png";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new(OUT_FILE_NAME, (1024, 768));
    let root = backend.to_drawing_area();

    root.fill(&mut backend, &WHITE)?;

    let root = root
        .titled(&mut backend, "Sierpinski Carpet Demo", ("sans-serif", 60))?
        .shrink(((1024 - 700) / 2, 0), (700, 700));

    sierpinski_carpet(5, &mut backend, &root)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present(&mut backend).expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
