use rstempel::Stem;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let word = args
        .get(1)
        .expect("Missing stem word argument in position 1");
    let stemmer = &rstempel::rust::STEMMER;
    let stemmed = stemmer.stem(word);
    println!("{}\t{}", word, stemmed);
    Ok(())
}
