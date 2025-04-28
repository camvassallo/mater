use reqwest; // Use the blocking client
use csv::{ReaderBuilder, StringRecord, Reader}; // Import Reader explicitly
use serde::Deserialize; // Use serde for deserialization
use std::error::Error;
// Import the logging macros from the `log` crate
use log::{error, warn, info, debug, trace, LevelFilter};
// Import the `env_logger` initializer
use env_logger;

// Define a struct that matches the columns you expect in the CSV.
#[derive(Debug, Deserialize)]
struct PlayerStats {
    player_name: String,
    team: String,
    conf: String,
    gp: Option<u32>,
    min_per: Option<f64>,
    o_rtg: Option<f64>,
    usg: Option<f64>,
    e_fg: Option<f64>,
    ts_per: Option<f64>,
    orb_per: Option<f64>,
    drb_per: Option<f64>,
    ast_per: Option<f64>,
    to_per: Option<f64>,
    ftm: Option<u32>,
    fta: Option<u32>,
    ft_per: Option<f64>,
    two_pm: Option<u32>,
    two_pa: Option<u32>,
    two_p_per: Option<f64>,
    tpm: Option<u32>,
    tpa: Option<u32>,
    tp_per: Option<f64>,
    blk_per: Option<f64>,
    stl_per: Option<f64>,
    ftr: Option<f64>,
    yr: Option<String>,
    ht: Option<String>,
    num: Option<String>,
    porpag: Option<f64>,
    adjoe: Option<f64>,
    pfr: Option<f64>,
    year: Option<u32>,
    pid: Option<u32>,
    player_type: Option<String>, // Renamed from 'type' to avoid keyword clash
    rec_rank: Option<f64>,
    ast_tov: Option<f64>,
    rim_made: Option<f64>,
    rim_attempted: Option<f64>,
    mid_made: Option<f64>,
    mid_attempted: Option<f64>,
    rim_pct: Option<f64>,
    mid_pct: Option<f64>,
    dunks_made: Option<f64>,
    dunks_attempted: Option<f64>,
    dunk_pct: Option<f64>,
    pick: Option<f64>,
    drtg: Option<f64>,
    adrtg: Option<f64>,
    dporpag: Option<f64>,
    stops: Option<f64>,
    bpm: Option<f64>,
    obpm: Option<f64>,
    dbpm: Option<f64>,
    gbpm: Option<f64>,
    mp: Option<f64>,
    ogbpm: Option<f64>,
    dgbpm: Option<f64>,
    oreb: Option<f64>,
    dreb: Option<f64>,
    treb: Option<f64>,
    ast: Option<f64>,
    stl: Option<f64>,
    blk: Option<f64>,
    pts: Option<f64>,
}


fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();
    
    // The URL - using 2024 data as 2025 might not be available yet
    // Using csv=0 as csv=1 might add headers we don't want here
    let url = "https://barttorvik.com/getadvstats.php?year=2025&csv=1";

    info!("Fetching data from: {}", url);

    // Fetch the CSV data as text using the blocking client
    let csv_data = reqwest::blocking::get(url)?.text()?;

    info!("Data fetched successfully. Parsing CSV...");

    // Define the headers manually in the correct order, using snake_case names.
    // These MUST match the order of columns in the CSV data and the struct field names above.
    let headers = StringRecord::from(vec![
        "player_name", "team", "conf", "gp", "min_per", "o_rtg", "usg", "e_fg", "ts_per",
        "orb_per", "drb_per", "ast_per", "to_per", "ftm", "fta", "ft_per", "two_pm", "two_pa",
        "two_p_per", "tpm", "tpa", "tp_per", "blk_per", "stl_per", "ftr", "yr", "ht", "num",
        "porpag", "adjoe", "pfr", "year", "pid", "player_type", // Use the renamed field 'player_type'
        "rec_rank", "ast_tov", "rim_made", "rim_attempted", "mid_made", "mid_attempted",
        "rim_pct", "mid_pct", "dunks_made", "dunks_attempted", "dunk_pct", "pick", "drtg",
        "adrtg", "dporpag", "stops", "bpm", "obpm", "dbpm", "gbpm", "mp", "ogbpm", "dgbpm",
        "oreb", "dreb", "treb", "ast", "stl", "blk", "pts"
    ]);

    // Create a CSV reader builder and configure it
    let mut reader_builder = ReaderBuilder::new();
    reader_builder
        .has_headers(false) // Set to false as the data does not contain headers
        .trim(csv::Trim::All); // Trim whitespace from fields

    // Build the reader from the CSV data bytes
    // Need to specify the type for the reader, e.g., Reader<&[u8]>
    let mut reader: Reader<&[u8]> = reader_builder.from_reader(csv_data.as_bytes());

    // Set the headers on the reader instance *after* it's created
    reader.set_headers(headers);

    // --- Deserialize into your struct ---
    info!("Deserializing rows into PlayerStats struct using snake_case headers:");

    // Initialize an empty vector to store the PlayerStats records
    let mut players: Vec<PlayerStats> = Vec::new();
    let mut error_count = 0;

    // Now deserialize using the reader with the headers set
    for result in reader.deserialize::<PlayerStats>() {
        match result {
            Ok(record) => {
                // Push the successfully deserialized record into the vector
                players.push(record);
            }
            Err(e) => {
                error_count += 1;
                if error_count <= 5 {
                    error!("Error deserializing row: {}", e);
                } else if error_count == 6 {
                    error!("... (further deserialization errors suppressed)");
                }
            }
        }
    }

    info!("CSV processing finished.");
    info!("Successfully parsed and collected {} player records.", players.len()); // Use players.len()
    if error_count > 0 {
        info!("Encountered {} errors during deserialization.", error_count);
    }

    // Optional: Print details of the first few collected players
    if !players.is_empty() {
        info!("\nFirst few players collected:");
        for (i, player) in players.iter().enumerate().take(5) { // Iterate over the collected players
            info!(
                "{}. Player: {}, Team: {}, Pts: {:.1?}, Reb: {:.1?}, Ast: {:.1?}",
                i + 1,
                player.player_name,
                player.team,
                player.pts,
                player.treb,
                player.ast
            );
        }
        if players.len() > 5 {
            info!("... (and {} more)", players.len() - 5);
        }
    } else {
        info!("\nNo players were collected.");
    }
    
    for player in players {
        if player.player_name == "Cooper Flagg" {
            info!("{}", player.adjoe.unwrap());
        }
    }


    Ok(())
}
