extern crate csv;
extern crate rand;
extern crate rustc_serialize;

use rand::{Rng, sample, thread_rng};
use std::collections::hash_set::HashSet;

const ENGLISH: u32 = 9;

#[derive(RustcDecodable)]
struct SpeciesRecord {
    species_id: u32,
    language_id: u32,
    name: String,
    genus: Option<String>,  // Some genuses are empty.
}

#[derive(RustcDecodable)]
struct FlavorRecord {
    species_id: u32,
    version_id: u32,
    language_id: u32,
    flavor_text: String,
}

/// The actual structure we use to generate responses.
#[derive(RustcDecodable)]
struct PokedexEntry {
    species: String,
    genus: String,
    species_id: u32,
    // We use a HashSet because some flavor text is repeated across versions.
    flavor: HashSet<String>,
}

fn construct_pokedex() -> Vec<PokedexEntry> {
    let mut dex: Vec<PokedexEntry> = Vec::new();

    let mut rdr = csv::Reader::from_file("./pokedex/pokemon_species_names.csv").unwrap();
    for record in rdr.decode() {
        let species: SpeciesRecord = record.unwrap();
        if species.language_id == ENGLISH {
            assert_eq!(dex.len() + 1, species.species_id as usize);
            dex.push(PokedexEntry {
                species: species.name,
                genus: species.genus.unwrap_or(String::from("")),
                species_id: species.species_id,
                flavor: HashSet::new(),
            });
        }
    }
    rdr = csv::Reader::from_file("./pokedex/pokemon_species_flavor_text.csv").unwrap();
    for record in rdr.decode() {
        let flavor: FlavorRecord = record.unwrap();
        if flavor.language_id == ENGLISH {
            // TODO: This replace still isn't quite right.
            dex[(flavor.species_id - 1) as usize].flavor.insert(
                flavor.flavor_text.replace("\n", " ").replace("\r", " "));
        }
    }
    dex
}

fn main() {
    println!("Processing CSV files....");
    let dex: &[PokedexEntry] = &construct_pokedex();
    println!("Done processing CSV files.");

    let mut rng = thread_rng();
    for x in 0..5 {
        let entry: &PokedexEntry = rng.choose(&dex).unwrap();
        println!("{}. {}, The {} Pokemon\n{}",
            entry.species_id,
            entry.species,
            entry.genus,
            sample(&mut rng, &entry.flavor, 1)[0],
        );
    }
}
