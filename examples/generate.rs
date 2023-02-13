use flate2::bufread::GzDecoder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use rstempel::rust::generate::RustGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("src/tables/stemmer_2000.out.gz");
    let license = r"Converted from `stemmer_2000.out` from Stempel by Andrzej Bialecki. http://www.getopt.org/stempel/index.html
    Offered under the Apache License 2.0. https://www.apache.org/licenses/LICENSE-2.0";
    convert_java_table(path, license)?;

    Ok(())
}

fn convert_java_table(input: &Path, license: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = input.with_extension("rs");

    let input = fs::File::open(input)?;
    let input = io::BufReader::new(GzDecoder::new(io::BufReader::new(input)));

    let gen = RustGenerator::load_java_table(input)?;

    let output = fs::File::create(output)?;
    let mut output = io::BufWriter::new(output);

    for line in license.lines() {
        writeln!(output, "// {}", line)?;
    }
    writeln!(output)?;

    gen.write_rust_table(&mut output)?;
    writeln!(output)?;

    Ok(())
}
