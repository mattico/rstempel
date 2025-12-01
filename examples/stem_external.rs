use flate2::bufread::GzDecoder;
use rstempel::external::Stemmer;
use rstempel::Stem;
use std::env;
use std::error::Error;
use std::fs;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let word = args
        .get(1)
        .expect("Missing stem word argument in position 1");
    let table = "src/tables/stemmer_2000.out.gz";
    let table = fs::File::open(table)?;
    let table = io::BufReader::new(GzDecoder::new(io::BufReader::new(table)));
    let stemmer = Stemmer::load(table)?;
    let stemmed = stemmer.stem(word);
    println!("{}\t{}", word, stemmed);
    Ok(())
}
