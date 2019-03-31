extern crate chrono;
extern crate csv;
extern crate rand;
extern crate rustc_serialize;
extern crate slack;

use chrono::prelude::{Datelike, Utc};
use rand::{Rng, sample, thread_rng};
use rustc_serialize::json;
use std::collections::hash_set::HashSet;
use std::time::Duration;

const ENGLISH: u32 = 9;

#[derive(RustcDecodable)]
struct SpeciesRecord {
    species_id: u32,
    language_id: u32,
    name: String,
    genus: Option<String>,  // Some genuses are empty.
}

#[derive(RustcDecodable)]
#[allow(dead_code)]
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

struct SlackHandler {
    pokedex: Vec<PokedexEntry>,
}

/// Construct an attachment object for a given pokemon species.
fn attachment_for_pokemon(pokemon: &PokedexEntry) -> slack::Attachment {
    let image_url = format!("http://assets.pokemon.com/assets/cms2/img/pokedex/full/{:03}.png",
                            pokemon.species_id);
    let flavor = &sample(&mut thread_rng(), &pokemon.flavor, 1)[0];

    // TODO: Create our own Attachment record or use a less verbose way of filling these optional
    // fields.
    slack::Attachment {
        fallback: Some(format!("#{:03} {}, The {} Pokémon\n{}\n{}",
                          pokemon.species_id,
                          &pokemon.species,
                          &pokemon.genus,
                          flavor,
                          &image_url)),
        color: None,
        pretext: None,
        author_name: Some(format!("#{:03}", pokemon.species_id)),
        author_link: None,
        author_icon: None,
        title: Some(pokemon.species.clone()),
        title_link: None,
        text: Some(format!("_The {}_\n\n{}", &pokemon.genus, flavor)),
        fields: None,
        mrkdwn_in: Some(vec![String::from("text")]),
        image_url: Some(image_url),
        thumb_url: None,
    }
}

#[allow(unused_attributes, unused_variables)]
impl slack::EventHandler for SlackHandler {
    fn on_event(&mut self, cli: &mut slack::RtmClient, event: Result<&slack::Event, slack::Error>,
                raw_json: &str) {
        match event {
            // Sample structure.
            // Ok(Message(Standard { ts: "1465616511.000007", channel: Some("G0RFEFRF1"),
            //    user: Some("U04R67MSW"), text: Some("#sloakme"), is_starred: None,
            //    pinned_to: None, reactions: None, edited: None, attachments: None }))
            Ok(ev) => if let &slack::Event::Message(slack::Message::Standard { channel: Some(ref channel), user: Some(ref user), text: Some(ref text), .. }) = ev {
                if text.contains("#pokeme") {
                    // On trans visibility day, March 31st, everyone is Sylveon.
                    let today = Utc::today();
                    let pokemon = if today.month() == 3 && today.day() == 31 {
                        &self.pokedex[699]
                    } else {
                        thread_rng().choose(&self.pokedex).unwrap()
                    };
                    let attachment_json = json::encode(&vec![attachment_for_pokemon(pokemon)]).unwrap();
                    let _ = cli.post_message(channel,
                                             &format!("You are a {}!", &pokemon.species),
                                             Some(&attachment_json));
                }
            },
            Err(err) => println!("Error on event: {}", err),
        }
    }

    fn on_ping(&mut self, cli: &mut slack::RtmClient) { }

    fn on_close(&mut self, cli: &mut slack::RtmClient) {
        println!("Closed.")
    }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) {
        println!("Connected.");
    }
}

/// Constructs an English-language pokedex from csv files.
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
    let args: Vec<String> = std::env::args().collect();
    let api_key = match args.len() {
        0 | 1 => panic!("No api-key in args! Usage: cargo run -- <api-key>"),
        x => {
            args[x - 1].clone()
        }
    };

    println!("Processing CSV files....");
    let mut slack_handler = SlackHandler { pokedex: construct_pokedex() };
    println!("Done processing CSV files. Connecting to slack.");

    let mut cli = slack::RtmClient::new(&api_key);
    loop {
        match cli.login_and_run::<SlackHandler>(&mut slack_handler) {
            Ok(_) => {}
            Err(err) => println!("Error: {}", err),
        }
        println!("Disconnected.  Sleeping and reconnecting....");
        std::thread::sleep(Duration::new(5, 0));
    }
}
