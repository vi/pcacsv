use std::{path::PathBuf, io::Write};

use gumdrop::Options;
use ndarray::Axis;
use trimothy::TrimSlice;

#[derive(Debug, Options)]
struct MyOptions {
    /// List of columns to use as coordinates. First column is number 1. Parsing support ranges with steps like 3,4,10:5:100.
    /// See `number_range` Rust crate for details.
    /// Use `xsv headers your_file.csv` to find out column numbers.
    #[options(required, free)]
    columns: utils::ColumnsSpecifier,

    /// Input CSV file
    #[options(free)]
    input_path: Option<PathBuf>,

    /// Save file there instead of stdout
    output: Option<PathBuf>,

    /// First line of the CSV is not headers
    no_header: bool,

    /// Do not output CSV header even though input has headers
    no_output_header: bool,

    /// Field delimiter in CSV files. Comma by default.
    delimiter: Option<utils::DelimiterSpecifier>,

    /// Override line delimiter in CSV files.
    record_delimiter: Option<utils::DelimiterSpecifier>,

    /// Tolerance for excluding low variance components. If not specified, all components are kept.
    tolerance: Option<f64>,

    help: bool,
}

mod utils;

type Arr2 = ndarray::Array2<f64>;


fn main() -> anyhow::Result<()> {
    let opts = MyOptions::parse_args_default_or_exit();

    //println!("{:#?}", opts);
    let f = opts.get_istream()?;
    let mut f = opts.get_csv_reader().from_reader(f);

    let mut records = Vec::<csv::ByteRecord>::with_capacity(1024);
    let header: Option<csv::ByteRecord> = if f.has_headers() && !opts.no_output_header {
        Some(f.byte_headers()?.clone())
    } else {
        None
    };
    for record in f.into_byte_records() {
        let record = record?;
        records.push(record);
    }

    let n_rows = records.len();
    let n_input_coords = opts.columns.0.len();
    let mut inputvals = Arr2::zeros((n_rows, n_input_coords));

    for (j, record) in records.iter().enumerate() {
        let mut ctr = 0;
        for (i, field) in record.iter().enumerate() {
            if opts.columns.0.contains(&(i + 1)) {
                let field = field.trim();
                let x: f64 = std::str::from_utf8(field)?.parse()?;
                inputvals[(j, ctr)] = x;
                ctr += 1;
            }
        }
        if ctr != opts.columns.0.len() {
            anyhow::bail!("Field list contains invalid column numbers");
        }
    }

    let mut pca = pca::PCA::new();

    if let Err(e) = pca.fit(inputvals.clone(), opts.tolerance) {
        eprintln!("Error: {e}");
    }
    let Ok(ret) = pca.transform(inputvals) else {
        anyhow::bail!("Error transforming");
    };

    let n_out_coords = ret.len_of(Axis(1));

    let f = opts.get_ostream()?;
    let f = opts.get_csv_writer().from_writer(f);
    save_csv(&header, n_out_coords, f, &records, ret.view())?;

    Ok(())
}


fn save_csv<'a>(
    header: &Option<csv::ByteRecord>,
    n_out_coords: usize,
    mut f: csv::Writer<impl Write>,
    records: &Vec<csv::ByteRecord>,
    coords: ndarray::ArrayView2<'a, f64>,
) -> Result<(), anyhow::Error> {
    if let Some(h) = &header {
        for i in 1..=n_out_coords {
            f.write_field(format!("coord{}", i))?;
        }
        f.write_record(h)?;
    }
    Ok(for (j, record) in records.iter().enumerate() {
        for i in 0..n_out_coords {
            f.write_field(format!("{:.4}", coords[(j, i)]))?;
        }
        f.write_record(record)?;
    })
}
