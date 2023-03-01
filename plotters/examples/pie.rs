use plotters::{prelude::*, style::full_palette::ORANGE};

const OUT_FILE_NAME: &'static str = "plotters-doc-data/pie-chart.png";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new(&OUT_FILE_NAME, (950, 700));
    let root_area = backend.to_drawing_area();
    root_area.fill(&mut backend, &WHITE)?;
    let title_style = TextStyle::from(("sans-serif", 30).into_font()).color(&(BLACK));
    root_area.titled(&mut backend, "BEST CIRCLES", title_style)?;

    let dims = root_area.dim_in_pixel();
    let center = (dims.0 as i32 / 2, dims.1 as i32 / 2);
    let radius = 300.0;
    let sizes = vec![66.0, 33.0];
    let _rgba = RGBAColor(0, 50, 255, 1.0);
    let colors = vec![RGBColor(0, 50, 255), CYAN];
    let labels = vec!["Pizza", "Pacman"];

    let mut pie = Pie::new(&center, &radius, &sizes, &colors, &labels);
    pie.start_angle(66.0);
    let label_font = ("sans-serif", 50).into_font();
    pie.label_style(label_font.color(&(ORANGE)));
    let percentage_font = ("sans-serif", radius * 0.08).into_font();
    pie.percentages(percentage_font.color(&BLACK));
    root_area.draw(&mut backend, &pie)?;

    Ok(())
}
