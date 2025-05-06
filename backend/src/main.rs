mod init_db;

use reqwest; // Use the blocking client
use csv::{ReaderBuilder, StringRecord, Reader}; // Import Reader explicitly
use serde::{Deserialize, Serialize}; // Use serde for deserialization
use std::error::Error;
// Import the logging macros from the `log` crate
use log::{error, warn, info, debug, trace, LevelFilter};
// Import the `env_logger` initializer
use env_logger;
use scylla::{FromRow, IntoTypedRows, SerializeRow, Session, SessionBuilder};
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use actix_web::body::MessageBody;
use scylla::transport::session;
use crate::init_db::init_db;

// Define a struct that matches the columns you expect in the CSV.
#[derive(Debug, Clone, Deserialize, Serialize, FromRow, SerializeRow)]
pub struct PlayerStats {
    pub player_name: String,
    pub team: String,
    pub conf: String,
    pub gp: Option<i32>,
    pub min_per: Option<f64>,
    pub o_rtg: Option<f64>,
    pub usg: Option<f64>,
    pub e_fg: Option<f64>,
    pub ts_per: Option<f64>,
    pub orb_per: Option<f64>,
    pub drb_per: Option<f64>,
    pub ast_per: Option<f64>,
    pub to_per: Option<f64>,
    pub ftm: Option<i32>,
    pub fta: Option<i32>,
    pub ft_per: Option<f64>,
    pub two_pm: Option<i32>,
    pub two_pa: Option<i32>,
    pub two_p_per: Option<f64>,
    pub tpm: Option<i32>,
    pub tpa: Option<i32>,
    pub tp_per: Option<f64>,
    pub blk_per: Option<f64>,
    pub stl_per: Option<f64>,
    pub ftr: Option<f64>,
    pub yr: Option<String>,
    pub ht: Option<String>,
    pub num: Option<String>,
    pub porpag: Option<f64>,
    pub adjoe: Option<f64>,
    pub pfr: Option<f64>,
    pub year: Option<i32>,
    pub pid: Option<i32>,
    pub player_type: Option<String>,
    pub rec_rank: Option<f64>,
    pub ast_tov: Option<f64>,
    pub rim_made: Option<f64>,
    pub rim_attempted: Option<f64>,
    pub mid_made: Option<f64>,
    pub mid_attempted: Option<f64>,
    pub rim_pct: Option<f64>,
    pub mid_pct: Option<f64>,
    pub dunks_made: Option<f64>,
    pub dunks_attempted: Option<f64>,
    pub dunk_pct: Option<f64>,
    pub pick: Option<f64>,
    pub drtg: Option<f64>,
    pub adrtg: Option<f64>,
    pub dporpag: Option<f64>,
    pub stops: Option<f64>,
    pub bpm: Option<f64>,
    pub obpm: Option<f64>,
    pub dbpm: Option<f64>,
    pub gbpm: Option<f64>,
    pub mp: Option<f64>,
    pub ogbpm: Option<f64>,
    pub dgbpm: Option<f64>,
    pub oreb: Option<f64>,
    pub dreb: Option<f64>,
    pub treb: Option<f64>,
    pub ast: Option<f64>,
    pub stl: Option<f64>,
    pub blk: Option<f64>,
    pub pts: Option<f64>,
}

const KEYSPACE: &str = "stats";
const TABLE: &str = "player_stats";
const NODE_ADDRESS: &str = "127.0.0.1:9042";


async fn connect_to_scylla() -> Session {

    info!("Connecting to ScyllaDB at {}...", NODE_ADDRESS);

    let session = SessionBuilder::new()
        .known_node(NODE_ADDRESS) // adjust if running in Docker
        .build()
        .await
        .expect("Failed to connect to ScyllaDB");
    
    session.use_keyspace(KEYSPACE, true).await.expect("TODO: panic message");
    session
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
                player.pts.unwrap(),
                player.treb.unwrap(),
                player.ast.unwrap(),
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
            info!("{}", player.usg.unwrap());
        }
    }

    Ok(players)
}

pub async fn insert_player_stats(
    session: &Session,
    players: &[PlayerStats],
) -> Result<(), scylla::transport::errors::QueryError> {
    let query = r#"
    INSERT INTO stats.player_stats (
        player_name, team, conf, gp, min_per, o_rtg, usg, e_fg, ts_per, orb_per,
        drb_per, ast_per, to_per, ftm, fta, ft_per, two_pm, two_pa, two_p_per,
        tpm, tpa, tp_per, blk_per, stl_per, ftr, yr, ht, num, porpag, adjoe, pfr,
        year, pid, player_type, rec_rank, ast_tov, rim_made, rim_attempted,
        mid_made, mid_attempted, rim_pct, mid_pct, dunks_made, dunks_attempted,
        dunk_pct, pick, drtg, adrtg, dporpag, stops, bpm, obpm, dbpm, gbpm, mp,
        ogbpm, dgbpm, oreb, dreb, treb, ast, stl, blk, pts
    ) VALUES (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
    )
"#;

    let prepared = session.prepare(query).await?;
    for p in players {
        session.execute(&prepared, &p).await?;
    }

    Ok(())
}

#[get("/api/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust!")
}

#[get("/api/players")]
async fn get_players(
    db: web::Data<Session>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let team_code = match query.get("team") {
        Some(code) => code,
        None => return HttpResponse::BadRequest().body("Missing 'team' query param"),
    };

    let year = match query.get("year") {
        Some(y) => match y.parse::<i32>() {
            Ok(n) => n,
            Err(_) => return HttpResponse::BadRequest().body("Invalid 'year' query param"),
        },
        None => return HttpResponse::BadRequest().body("Missing 'year' query param"),
    };

    let query = r#"
    SELECT player_name, team, conf, gp, min_per, o_rtg, usg, e_fg, ts_per, orb_per,
           drb_per, ast_per, to_per, ftm, fta, ft_per, two_pm, two_pa, two_p_per,
           tpm, tpa, tp_per, blk_per, stl_per, ftr, yr, ht, num, porpag, adjoe, pfr,
           year, pid, player_type, rec_rank, ast_tov, rim_made, rim_attempted,
           mid_made, mid_attempted, rim_pct, mid_pct, dunks_made, dunks_attempted,
           dunk_pct, pick, drtg, adrtg, dporpag, stops, bpm, obpm, dbpm, gbpm, mp,
           ogbpm, dgbpm, oreb, dreb, treb, ast, stl, blk, pts
    FROM stats.player_stats WHERE team = ? AND year = ?
"#.to_string();
    let statement = match db.prepare(query).await {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to prepare: {e}")),
    };

    let result = db.execute(&statement, (team_code.as_str(), year)).await;

    let rows = match result {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query Scylla: {e}");
            return HttpResponse::InternalServerError().body(format!("Query failed: {e}"));
        }
    };

    info!("Returned {} rows", rows.len());

    let mut players: Vec<PlayerStats> = Vec::new();

    for (i, row) in rows.into_iter().enumerate() {
        match PlayerStats::from_row(row) {
            Ok(player) => players.push(player),
            Err(e) => {
                error!("Row {} failed to convert: {}", i, e);
            }
        }
    }

    HttpResponse::Ok().json(players)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();
    init_db().await.expect("DB setup failed");

    let db = connect_to_scylla().await;

    info!("🚀 Server running at http://localhost:8000");

    let mut players: Vec<PlayerStats> = get_data().await?;

    info!("Players collected: {}", players.len());

    insert_player_stats(&db, &players).await?;

    // query_specific_player(scylla_db, "Duke", "Cooper Flagg").await;

    let db_data = web::Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(get_players)
            .service(hello) // keep your old hello handler too
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await;

    Ok(())
}
