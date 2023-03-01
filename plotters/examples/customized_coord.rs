use plotters::{
    coord::ranged1d::{KeyPointHint, NoDefaultFormatting, ValueFormatter},
    prelude::*,
};
const OUT_FILE_NAME: &'static str = "plotters-doc-data/customized_coord.svg";

struct CustomizedX(u32);

impl Ranged for CustomizedX {
    type ValueType = u32;
    type FormatOption = NoDefaultFormatting;
    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let size = limit.1 - limit.0;
        ((*value as f64 / self.0 as f64) * size as f64) as i32 + limit.0
    }

    fn range(&self) -> std::ops::Range<Self::ValueType> {
        0..self.0
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        if hint.max_num_points() < (self.0 as usize) {
            return vec![];
        }

        (0..self.0).collect()
    }
}

impl ValueFormatter<u32> for CustomizedX {
    fn format_ext(&self, value: &u32) -> String {
        format!("{} of {}", value, self.0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = SVGBackend::new(OUT_FILE_NAME, (1024, 760));
    let area = backend.to_drawing_area();
    area.fill(&mut backend, &WHITE)?;

    let mut chart = ChartBuilder::on(&area)
        .set_all_label_area_size(50)
        .build_cartesian_2d(&mut backend, CustomizedX(7), 0.0..10.0)?;

    chart.configure_mesh().draw(&mut backend)?;

    area.present(&mut backend).expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
