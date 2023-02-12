use std::path::Path;

use rstempel::rust::generate::RustGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    RustGenerator::convert_java_table(Path::new("src/tables/stemmer_2000.out"))?;
    Ok(())
}
