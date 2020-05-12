extern crate chrono;
extern crate csv;
extern crate rand;
extern crate rustc_serialize;
extern crate serenity;

use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::user::User;
use serenity::prelude::{EventHandler, Context, TypeMapKey};
use serenity::framework::standard::{
    StandardFramework,
    CommandError,
    CommandResult,
    macros::{
        command,
        group
    }
};
use serenity::utils::MessageBuilder;

use chrono::prelude::*;
use chrono::Duration;
use rand::{Rng, sample, thread_rng};
use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;
use std::env;

// Pokedex stuff
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
#[derive(RustcDecodable, Clone)]
struct PokedexEntry {
    species: String,
    genus: String,
    species_id: u32,
    // We use a HashSet because some flavor text is repeated across versions.
    flavor: HashSet<String>,
}

// Discord stuff
#[group]
#[commands(pokeme)]
struct General;

struct Handler;

impl EventHandler for Handler {}

struct Pokedex;

struct TimeoutMap {
    epoch: DateTime<Utc>,
    assignments: HashMap<UserId, String>
}

impl TypeMapKey for TimeoutMap {
    type Value = TimeoutMap;
}

/// Hour of the day (UTC) when the map expires.
static RESET_HOUR: u32 = 14;  // 2 PM

impl TypeMapKey for Pokedex {
    type Value = Vec<PokedexEntry>;
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

/// Calculate the most recent checkpoint.
fn current_checkpoint_time() -> DateTime::<Utc> {
    let now = Utc::now();
    if now.hour() >= RESET_HOUR {
        now.date().and_hms(RESET_HOUR, 0, 0)
    } else {
        (now - Duration::days(1)).date().and_hms(RESET_HOUR, 0, 0)
    }
}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP));
    {
        let mut data = client.data.write();
        // Set-up our timeout system
        data.insert::<TimeoutMap>(TimeoutMap {
            epoch: current_checkpoint_time(),
            assignments: HashMap::new()
        });
        println!("Processing CSV files....");
        data.insert::<Pokedex>(construct_pokedex());
    }
    println!("Done processing CSV files. Connecting to discord.");

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn pokeme(ctx: &mut Context, msg: &Message) -> CommandResult {
    let pokemon: PokedexEntry;
    {
        let data = ctx.data.read();
        let pokedex = data.get::<Pokedex>().unwrap();
        // On trans visibility day, March 31st, everyone is Sylveon.
        let today = Utc::today();
        pokemon = if today.month() == 3 && today.day() == 31 {
            pokedex[699].clone()
        } else {
            thread_rng().choose(pokedex).unwrap().clone()
        };
    }
    // Check if they've already gotten a horoscope in the recent
    // timestamp.
    let epoch = current_checkpoint_time();
    let timeout_reply: Option<String>;
    {
        let mut data = ctx.data.write();
        let tom = data.get_mut::<TimeoutMap>().unwrap();
        if epoch > tom.epoch {
            // Reset!
            tom.epoch = epoch;
            tom.assignments.clear();
        }
        match tom.assignments.get(&msg.author.id) {
            Some(species) => {
                // Don't do IO while holding a write lock.
                timeout_reply = Some(format!("You are still a {}. Try again tomorrow.", species));
            },
            None => {
                tom.assignments.insert(msg.author.id, pokemon.species.clone());
                timeout_reply = None;
            }
        }
    }
    if let Some(rep) = timeout_reply {
        msg.reply(ctx, rep);
        return Ok(())
    }

    let image_url = format!("http://assets.pokemon.com/assets/cms2/img/pokedex/full/{:03}.png",
        pokemon.species_id);
    let flavor = &sample(&mut thread_rng(), &pokemon.flavor, 1)[0];
    
    let msg = msg.channel_id.send_message(&ctx.http, |m| {
        m.content(format!("**{}**:\nYou are a **{}** (#{:03})!",
                          &msg.author,
                          &pokemon.species,
                          &pokemon.species_id));
        m.embed(|e| {
            e.title(format!("The {}", &pokemon.genus));
            e.description(flavor);
            e.image(image_url);
            e
        });

        m
    });

    if let Err(why) = msg {
        Err(CommandError(why.to_string()))
    } else {
        Ok(())
    }
}
