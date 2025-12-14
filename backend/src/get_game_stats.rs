use std::error::Error;
use log::{info, error};
use serde::{Deserialize, Serialize};
use scylla::{FromRow, SerializeRow, Session}; // Removed Bytes as it's no longer directly used with query_iter
use scylla::transport::errors::QueryError;
use flate2::read::GzDecoder;
use std::io::Read;
use std::time::Duration;
use scylla::query::Query;
use futures_util::stream::StreamExt; // NEW: Import StreamExt for the .next() method


// Helper function to parse a serde_json::Value into an Option<f64>
// Handles direct numbers and numeric strings, including empty strings for None.
fn get_opt_f64(value: &serde_json::Value) -> Result<Option<f64>, Box<dyn Error>> {
    if value.is_f64() {
        Ok(value.as_f64())
    } else if value.is_i64() {
        Ok(value.as_i64().map(|i| i as f64))
    } else if value.is_string() {
        let s = value.as_str().unwrap_or("");
        if s.is_empty() {
            Ok(None)
        } else {
            s.parse::<f64>().map(Some).map_err(|e| e.into())
        }
    } else {
        Ok(None)
    }
}

// Helper function to deserialize potentially empty strings or direct numbers into Option<i32>
fn get_opt_i32(value: &serde_json::Value) -> Result<Option<i32>, Box<dyn Error>> {
    if value.is_i64() {
        Ok(value.as_i64().map(|i| i as i32))
    } else if value.is_f64() { // Handle floats that might be integers (e.g., 2025.0)
        Ok(value.as_f64().map(|f| f as i32))
    }
    else if value.is_string() {
        let s = value.as_str().unwrap_or("");
        if s.is_empty() {
            Ok(None)
        } else {
            s.parse::<i32>().map(Some).map_err(|e| e.into())
        }
    } else {
        Ok(None)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SerializeRow)]
pub struct GameStats {
    pub numdate: String,
    pub datetext: String,
    pub opstyle: Option<i32>,
    pub quality: Option<i32>,
    pub win1: Option<i32>,
    pub opponent: String,
    pub muid: String,
    pub win2: Option<i32>,
    pub min_per: Option<f64>,
    pub o_rtg: Option<f64>,
    pub usage: Option<f64>,
    pub e_fg: Option<f64>,
    pub ts_per: Option<f64>,
    pub orb_per: Option<f64>,
    pub drb_per: Option<f64>,
    pub ast_per: Option<f64>,
    pub to_per: Option<f64>,
    pub dunks_made: Option<i32>,
    pub dunks_att: Option<i32>,
    pub rim_made: Option<i32>,
    pub rim_att: Option<i32>,
    pub mid_made: Option<i32>,
    pub mid_att: Option<i32>,
    pub two_pm: Option<i32>,
    pub two_pa: Option<i32>,
    pub tpm: Option<i32>,
    pub tpa: Option<i32>,
    pub ftm: Option<i32>,
    pub fta: Option<i32>,
    pub bpm_rd: Option<f64>,
    pub obpm: Option<f64>,
    pub dbpm: Option<f64>,
    pub bpm_net: Option<f64>,
    pub pts: Option<f64>,
    pub orb: Option<f64>,
    pub drb: Option<f64>,
    pub ast: Option<f64>,
    pub tov: Option<f64>,
    pub stl: Option<f64>,
    pub blk: Option<f64>,
    pub stl_per: Option<f64>,
    pub blk_per: Option<f64>,
    pub pf: Option<f64>,
    pub possessions: Option<f64>,
    pub bpm: Option<f64>,
    pub sbpm: Option<f64>,
    pub loc: String,
    pub tt: String,
    pub pp: String,
    pub inches: Option<i32>,
    pub cls: String,
    pub pid: Option<i32>,
    pub year: Option<i32>,
}

impl GameStats {
    pub fn from_json_array(arr: &[serde_json::Value]) -> Result<Self, Box<dyn Error>> {
        let get_str_val = |idx: usize| -> Result<String, Box<dyn Error>> {
            arr.get(idx)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| format!("Missing or invalid string at index {}", idx).into())
        };

        let get_raw_val = |idx: usize| -> &serde_json::Value {
            arr.get(idx).unwrap_or(&serde_json::Value::Null)
        };

        let game_stats = GameStats {
            numdate: get_str_val(0)?,
            datetext: get_str_val(1)?,
            opstyle: get_opt_i32(get_raw_val(2))?,
            quality: get_opt_i32(get_raw_val(3))?,
            win1: get_opt_i32(get_raw_val(4))?,
            opponent: get_str_val(5)?,
            muid: get_str_val(6)?,
            win2: get_opt_i32(get_raw_val(7))?,
            min_per: get_opt_f64(get_raw_val(8))?,
            o_rtg: get_opt_f64(get_raw_val(9))?,
            usage: get_opt_f64(get_raw_val(10))?,
            e_fg: get_opt_f64(get_raw_val(11))?,
            ts_per: get_opt_f64(get_raw_val(12))?,
            orb_per: get_opt_f64(get_raw_val(13))?,
            drb_per: get_opt_f64(get_raw_val(14))?,
            ast_per: get_opt_f64(get_raw_val(15))?,
            to_per: get_opt_f64(get_raw_val(16))?,
            dunks_made: get_opt_i32(get_raw_val(17))?,
            dunks_att: get_opt_i32(get_raw_val(18))?,
            rim_made: get_opt_i32(get_raw_val(19))?,
            rim_att: get_opt_i32(get_raw_val(20))?,
            mid_made: get_opt_i32(get_raw_val(21))?,
            mid_att: get_opt_i32(get_raw_val(22))?,
            two_pm: get_opt_i32(get_raw_val(23))?,
            two_pa: get_opt_i32(get_raw_val(24))?,
            tpm: get_opt_i32(get_raw_val(25))?,
            tpa: get_opt_i32(get_raw_val(26))?,
            ftm: get_opt_i32(get_raw_val(27))?,
            fta: get_opt_i32(get_raw_val(28))?,
            bpm_rd: get_opt_f64(get_raw_val(29))?,
            obpm: get_opt_f64(get_raw_val(30))?,
            dbpm: get_opt_f64(get_raw_val(31))?,
            bpm_net: get_opt_f64(get_raw_val(32))?,
            pts: get_opt_f64(get_raw_val(33))?,
            orb: get_opt_f64(get_raw_val(34))?,
            drb: get_opt_f64(get_raw_val(35))?,
            ast: get_opt_f64(get_raw_val(36))?,
            tov: get_opt_f64(get_raw_val(37))?,
            stl: get_opt_f64(get_raw_val(38))?,
            blk: get_opt_f64(get_raw_val(39))?,
            stl_per: get_opt_f64(get_raw_val(40))?,
            blk_per: get_opt_f64(get_raw_val(41))?,
            pf: get_opt_f64(get_raw_val(42))?,
            possessions: get_opt_f64(get_raw_val(43))?,
            bpm: get_opt_f64(get_raw_val(44))?,
            sbpm: get_opt_f64(get_raw_val(45))?,
            loc: get_str_val(46)?,
            tt: get_str_val(47)?,
            pp: get_str_val(48)?,
            inches: get_opt_i32(get_raw_val(49))?,
            cls: get_str_val(50)?,
            pid: get_opt_i32(get_raw_val(51))?,
            year: get_opt_i32(get_raw_val(52))?,
        };
        Ok(game_stats)
    }
}

pub async fn get_game_data() -> Result<Vec<GameStats>, Box<dyn Error>> {
    let url = "https://barttorvik.com/2026_all_advgames.json.gz";
    info!("Fetching gzipped game data from: {}", url);

    let response = reqwest::get(url).await?.bytes().await?;

    info!("Decompressing game data...");
    let mut gz_decoder = GzDecoder::new(&response[..]);
    let mut decompressed_data = String::new();
    gz_decoder.read_to_string(&mut decompressed_data)?;

    info!("Game data decompressed. Parsing JSON...");

    let raw_data: Vec<Vec<serde_json::Value>> = serde_json::from_str(&decompressed_data)?;

    let mut game_stats_records: Vec<GameStats> = Vec::new();
    let mut error_count = 0;

    for (i, row) in raw_data.into_iter().enumerate() {
        match GameStats::from_json_array(&row) {
            Ok(record) => {
                game_stats_records.push(record);
            }
            Err(e) => {
                error_count += 1;
                if error_count <= 5 {
                    error!("Error deserializing game row {}: {:?}", i, e);
                    error!("Problematic row data: {:?}", row);
                } else if error_count == 6 {
                    error!("... (further deserialization errors suppressed)");
                }
            }
        }
    }

    info!("Game data processing finished.");
    info!("Successfully parsed and collected {} game records.", game_stats_records.len());
    if error_count > 0 {
        info!("Encountered {} errors during deserialization.", error_count);
    }

    if !game_stats_records.is_empty() {
        info!("\nFirst few game records collected:");
        for (i, game) in game_stats_records.iter().enumerate().take(5) {
            info!(
                "{}. Player: {}, Team: {}, Opponent: {}, Date: {}, Pts: {:.1?}",
                i + 1,
                game.pp,
                game.tt,
                game.opponent,
                game.datetext,
                game.pts.unwrap_or_default()
            );
        }
        if game_stats_records.len() > 5 {
            info!("... (and {} more)", game_stats_records.len() - 5);
        }
    } else {
        info!("\nNo game records were collected.");
    }

    Ok(game_stats_records)
}

pub async fn get_all_game_stats_from_db(
    session: &Session,
) -> Result<Vec<GameStats>, Box<dyn Error>> {
    info!("Fetching all game stats from database using query_iter...");
    let query_cql = r#"
        SELECT numdate, datetext, opstyle, quality, win1, opponent, muid, win2, min_per, o_rtg, usage,
               e_fg, ts_per, orb_per, drb_per, ast_per, to_per, dunks_made, dunks_att, rim_made,
               rim_att, mid_made, mid_att, two_pm, two_pa, tpm, tpa, ftm, fta, bpm_rd, obpm,
               dbpm, bpm_net, pts, orb, drb, ast, tov, stl, blk, stl_per, blk_per, pf,
               possessions, bpm, sbpm, loc, tt, pp, inches, cls, pid, year
        FROM stats.game_stats
    "#;

    let mut all_game_stats = Vec::new();
    let page_size: i32 = 5000; // Define a reasonable page size for internal iteration

    info!("Executing query_iter: {}", query_cql);

    // Create a Query object and set options like page size and timeout
    let mut query = Query::new(query_cql);
    query.set_page_size(page_size);
    query.set_request_timeout(Some(Duration::from_secs(60))); // Apply timeout to each internal page request

    // Use session.query_iter for efficient paging and streaming of results
    let mut rows_iter = session.query_iter(query, ()).await?; // Pass the Query object here

    let mut row_count = 0;
    while let Some(row_res) = rows_iter.next().await { // Corrected: .next() is available from StreamExt
        match row_res {
            Ok(row) => {
                match GameStats::from_row(row) {
                    Ok(stat) => {
                        all_game_stats.push(stat);
                        row_count += 1;
                    },
                    Err(e) => {
                        error!("Failed to parse game stats row (total processed: {}): {}", row_count, e);
                        // Decide whether to continue or break on parse errors
                    }
                }
            },
            Err(e) => {
                error!("Failed to retrieve row from query_iter (total processed: {}): {}", row_count, e);
                return Err(Box::new(e)); // Propagate query execution errors
            }
        }
    }

    info!("Successfully fetched and parsed a total of {} game stats records from database using query_iter.", all_game_stats.len());
    Ok(all_game_stats)
}

pub async fn insert_game_stats(
    session: &Session,
    games: &[GameStats],
) -> Result<(), QueryError> {
    let query = r#"
    INSERT INTO stats.game_stats (
        numdate, datetext, opstyle, quality, win1, opponent, muid, win2, min_per, o_rtg, usage, e_fg, ts_per, orb_per, drb_per, ast_per, to_per, dunks_made, dunks_att, rim_made, rim_att, mid_made, mid_att, two_pm, two_pa, tpm, tpa, ftm, fta, bpm_rd, obpm, dbpm, bpm_net, pts, orb, drb, ast, tov, stl, blk, stl_per, blk_per, pf, possessions, bpm, sbpm, loc, tt, pp, inches, cls, pid, year
    ) VALUES (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
    )
"#;

    let prepared = session.prepare(query).await?;
    for g in games {
        session.execute(&prepared, &g).await?;
    }

    Ok(())
}
