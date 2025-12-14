// src/get_player_stats.rs
use std::error::Error;
use log::{info, error};
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, StringRecord, Reader};
use scylla::{FromRow, SerializeRow, Session};
use scylla::transport::errors::QueryError;

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

pub async fn get_player_data() -> Result<Vec<PlayerStats>, Box<dyn Error>> {
    let url = "https://barttorvik.com/getadvstats.php?year=2026&csv=1";
    info!("Fetching data from: {}", url);
    let csv_data = reqwest::get(url).await?.text().await?;
    info!("Data fetched successfully. Parsing CSV...");

    let headers = StringRecord::from(vec![
        "player_name", "team", "conf", "gp", "min_per", "o_rtg", "usg", "e_fg", "ts_per",
        "orb_per", "drb_per", "ast_per", "to_per", "ftm", "fta", "ft_per", "two_pm", "two_pa",
        "two_p_per", "tpm", "tpa", "tp_per", "blk_per", "stl_per", "ftr", "yr", "ht", "num",
        "porpag", "adjoe", "pfr", "year", "pid", "player_type",
        "rec_rank", "ast_tov", "rim_made", "rim_attempted", "mid_made", "mid_attempted",
        "rim_pct", "mid_pct", "dunks_made", "dunks_attempted", "dunk_pct", "pick", "drtg",
        "adrtg", "dporpag", "stops", "bpm", "obpm", "dbpm", "gbpm", "mp", "ogbpm", "dgbpm",
        "oreb", "dreb", "treb", "ast", "stl", "blk", "pts"
    ]);

    let mut reader_builder = ReaderBuilder::new();
    reader_builder
        .has_headers(false)
        .trim(csv::Trim::All);
    let mut reader: Reader<&[u8]> = reader_builder.from_reader(csv_data.as_bytes());
    reader.set_headers(headers);

    info!("Deserializing rows into PlayerStats struct using snake_case headers:");

    let mut players: Vec<PlayerStats> = Vec::new();
    let mut error_count = 0;

    for result in reader.deserialize::<PlayerStats>() {
        match result {
            Ok(record) => {
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
    info!("Successfully parsed and collected {} player records.", players.len());
    if error_count > 0 {
        info!("Encountered {} errors during deserialization.", error_count);
    }

    if !players.is_empty() {
        info!("\nFirst few players collected:");
        for (i, player) in players.iter().enumerate().take(5) {
            info!(
                "{}. Player: {}, Team: {}, Pts: {:.1?}, Reb: {:.1?}, Ast: {:.1?}",
                i + 1,
                player.player_name,
                player.team,
                player.pts.unwrap_or_default(),
                player.treb.unwrap_or_default(),
                player.ast.unwrap_or_default(),
            );
        }
        if players.len() > 5 {
            info!("... (and {} more)", players.len() - 5);
        }
    } else {
        info!("\nNo players were collected.");
    }

    Ok(players)
}

pub async fn insert_player_stats(
    session: &Session,
    players: &[PlayerStats],
) -> Result<(), QueryError> {
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
