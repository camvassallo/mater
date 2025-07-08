use std::error::Error;
use log::{info, error, warn};
use scylla::{Session, SessionBuilder, IntoTypedRows, FromRow}; // Added FromRow
use crate::get_player_stats::PlayerStats; // Import PlayerStats struct

const KEYSPACE: &str = "stats";
const TABLE: &str = "player_stats";
const NODE_ADDRESS: &str = "127.0.0.1:9042";

pub async fn connect_to_scylla() -> Session {
    info!("Connecting to ScyllaDB at {}...", NODE_ADDRESS);
    let session = SessionBuilder::new()
        .known_node(NODE_ADDRESS)
        .build()
        .await
        .expect("Failed to connect to ScyllaDB");
    session.use_keyspace(KEYSPACE, true).await.expect("Failed to use keyspace");
    session
}

pub async fn query_specific_player(session: &Session, team_name: &str, player_name_filter: &str, year: i32) -> Result<(), Box<dyn Error>> {
    info!("\nQuerying for player '{}' on team '{}' for year {}...", player_name_filter, team_name, year);

    let select_cql = format!(
        "SELECT player_name, team, gp FROM {} WHERE team = ? AND year = ? AND player_name = ? LIMIT 1", // Added year to WHERE
        TABLE
    );

    if let Some(rows) = session
        .query(select_cql, (team_name, year, player_name_filter)) // Pass year here
        .await?
        .rows
    {
        if rows.is_empty() {
            warn!("No player found matching Name='{}', Team='{}', Year='{}'", player_name_filter, team_name, year);
        } else {
            for row in rows.into_typed::<(String, String, Option<i32>)>() {
                match row {
                    Ok((name, team, gp)) => {
                        info!("Retrieved: Name={}, Team={}, GP={:?}", name, team, gp)
                    }
                    Err(e) => error!("Error parsing row: {}", e),
                }
            }
        }
    } else {
        info!("Query returned no rows structure for Name='{}', Team='{}', Year='{}'", player_name_filter, team_name, year);
    }
    Ok(())
}

pub async fn get_players_from_db(
    session: &Session,
    team_code: &str,
    year: i32,
) -> Result<Vec<PlayerStats>, scylla::transport::errors::QueryError> {
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

    let prepared = session.prepare(query).await?;
    let result = session.execute(&prepared, (team_code, year)).await?;
    let rows = result.rows.unwrap_or_default();

    let mut players: Vec<PlayerStats> = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        match PlayerStats::from_row(row) {
            Ok(player) => players.push(player),
            Err(e) => {
                error!("Row {} failed to convert: {}", i, e);
            }
        }
    }
    Ok(players)
}
