use std::error::Error;
use std::env;
use std::io;
use rstempel::Stem;
use rstempel::java::Stemmer;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let word = args.get(1).expect("Missing stem word argument in position 1");
    let table = include_bytes!("../tables/stemmer_2000.out");
    let stemmer = Stemmer::load(io::Cursor::new(table))?;
    let stemmed = stemmer.stem(word);
    println!("{}\t{}", word, stemmed);
    Ok(())
}
