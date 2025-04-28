use reqwest; // Use the blocking client
use csv::{ReaderBuilder, StringRecord, Reader}; // Import Reader explicitly
use serde::Deserialize; // Use serde for deserialization
use std::error::Error;
// Import the logging macros from the `log` crate
use log::{error, warn, info, debug, trace, LevelFilter};
// Import the `env_logger` initializer
use env_logger;
use scylla::{IntoTypedRows, Session, SessionBuilder};

// Define a struct that matches the columns you expect in the CSV.
#[derive(Debug, Deserialize, Clone)]
struct PlayerStats {
    player_name: String,
    team: String,
    conf: String,
    gp: Option<i32>,
    min_per: Option<f64>,
    o_rtg: Option<f64>,
    usg: Option<f64>,
    e_fg: Option<f64>,
    ts_per: Option<f64>,
    orb_per: Option<f64>,
    drb_per: Option<f64>,
    ast_per: Option<f64>,
    to_per: Option<f64>,
    ftm: Option<i32>,
    fta: Option<i32>,
    ft_per: Option<f64>,
    two_pm: Option<i32>,
    two_pa: Option<i32>,
    two_p_per: Option<f64>,
    tpm: Option<i32>,
    tpa: Option<i32>,
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
    year: Option<i32>,
    pid: Option<i32>,
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

const KEYSPACE: &str = "player_analytics";
const TABLE: &str = "player_stats";
const NODE_ADDRESS: &str = "127.0.0.1:9042";


async fn scylla(players:Vec<PlayerStats>) -> Result<Session, Box<dyn Error>> {

    info!("Connecting to ScyllaDB at {}...", NODE_ADDRESS);

    // 1. Establish Connection
    let session: Session = SessionBuilder::new()
        .known_node(NODE_ADDRESS)
        .build()
        .await?;

    info!("Connection successful!");

    // 2. Create Keyspace (if it doesn't exist)
    let create_keyspace_cql = format!(
        "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
        KEYSPACE
    );
    session.query(create_keyspace_cql, &[]).await?;
    info!("Keyspace '{}' ensured.", KEYSPACE);

    // Use the keyspace for subsequent operations
    session.use_keyspace(KEYSPACE, true).await?;

    // 3. Create Table (if it doesn't exist)
    //    IMPORTANT: Choose a primary key suitable for your data model.
    //    Here, we use (team, player_name) assuming a player on a specific team is unique.
    //    `team` is the partition key (determines data distribution).
    //    `player_name` is the clustering key (determines order within a partition).
    let create_table_cql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            player_name TEXT,
            team TEXT,
            conf TEXT,
            gp INT,         // Scylla INT can store i32 fine
            min_per DOUBLE, // Scylla DOUBLE maps well to f64
            o_rtg DOUBLE,
            usg DOUBLE,
            PRIMARY KEY ((team), player_name) // Composite Primary Key: Partition by team, cluster by player_name
        )",
        TABLE
    );
    session.query(create_table_cql, &[]).await?;
    info!("Table '{}.{}' ensured.", KEYSPACE, TABLE);

    // TODO: get the data here
    // DONE - passed in as arg

    // 5. Prepare the INSERT statement for efficiency
    let insert_cql = format!(
        "INSERT INTO {} (player_name, team, conf, gp, min_per, o_rtg, usg) VALUES (?, ?, ?, ?, ?, ?, ?)",
        TABLE
    );
    let prepared_insert = session.prepare(insert_cql).await?;
    info!("Prepared INSERT statement.");

    // 6. Iterate and Insert Data
    info!("Inserting player data...");
    for player in players {
        info!("Inserting stats for: {}", player.player_name);
        // The driver handles Option<T> correctly, mapping None to NULL.
        // The order of values in the tuple MUST match the order of '?' in the CQL.
        session
            .execute(
                &prepared_insert,
                (
                    &player.player_name, // Use references where possible
                    &player.team,
                    &player.conf,
                    player.gp,        // Pass Option<i32> directly
                    player.min_per,   // Pass Option<f64> directly
                    player.o_rtg,
                    player.usg,
                ),
            )
            .await?;
    }

    info!("Data insertion complete!");
    Ok(session)
}

async fn query_specific_player(session: Session, team_name: &str, player_name_filter: &str) -> Result<(), Box<dyn Error>> {
    info!("\nQuerying for player '{}' on team '{}'...", player_name_filter, team_name);

    // 1. Define CQL with placeholders for team and player_name
    let select_cql = format!(
        "SELECT player_name, team, gp FROM {} WHERE team = ? AND player_name = ? LIMIT 1", // Added WHERE clause and LIMIT 1
        TABLE
    );

    // 2. Execute the query, passing values in a tuple
    //    The order in the tuple MUST match the order of '?' in the CQL
    if let Some(rows) = session
        .query(select_cql, (team_name, player_name_filter)) // Pass values here
        .await?
        .rows
    {
        if rows.is_empty() {
            warn!("No player found matching Name='{}' and Team='{}'", player_name_filter, team_name);
        } else {
            // Expecting only one row due to primary key uniqueness or LIMIT 1
            for row in rows.into_typed::<(String, String, Option<i32>)>() { // Types match SELECT clause
                match row {
                    Ok((name, team, gp)) => {
                        info!("Retrieved: Name={}, Team={}, GP={:?}", name, team, gp)
                    }
                    Err(e) => error!("Error parsing row: {}", e),
                }
            }
        }
    } else {
        info!("Query returned no rows structure for Name='{}' and Team='{}'", player_name_filter, team_name);
    }

    Ok(())
}

async fn get_data() -> Result<Vec<PlayerStats>, Box<dyn Error>> {
    // The URL - using 2024 data as 2025 might not be available yet
    // Using csv=0 as csv=1 might add headers we don't want here
    let url = "https://barttorvik.com/getadvstats.php?year=2025&csv=1";

    info!("Fetching data from: {}", url);

    // Fetch the CSV data as text using the blocking client
    let csv_data = reqwest::get(url).await?.text().await?;

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

    // TODO: learn why we use &players here
    // the error was "Value used after being moved"
    for player in &players {
        if player.player_name == "Cooper Flagg" {
            info!("{}", player.adjoe.unwrap());
        }
    }

    Ok(players)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    let mut players: Vec<PlayerStats> = get_data().await?;

    info!("Players collected: {}", players.len());

    players.clear();

    let scylla_db: Session = scylla(players).await?;
    query_specific_player(scylla_db, "Providence", "Bryce Hopkins").await
}
