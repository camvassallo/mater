use std::error::Error;
use log::info;
use serde::{Deserialize, Serialize};
use scylla::{Session, SerializeRow, FromRow}; // FromRow is already here

#[derive(Debug, Clone, Serialize, Deserialize, SerializeRow, FromRow)]
pub struct TeamStats {
    pub rank: i32,
    pub team: String,
    pub conf: String,
    pub record: String,
    pub adjoe: f64,
    pub adjoe_rank: i32,
    pub adjde: f64,
    pub adjde_rank: i32,
    pub barthag: f64,
    pub barthag_rank: i32,
    pub proj_wins: i32,
    pub proj_losses: i32,
    pub proj_conf_wins: i32,
    pub proj_conf_losses: i32,
    pub conf_record: String,
    pub sos: f64,
    pub nconf_sos: f64,
    pub conf_sos: f64,
    pub proj_sos: f64,
    pub proj_nconf_sos: f64,
    pub proj_conf_sos: f64,
    pub elite_sos: f64,
    pub elite_ncsos: f64,
    pub opp_adjoe: f64,
    pub opp_adjde: f64,
    pub opp_proj_adjoe: f64,
    pub opp_proj_adjde: f64,
    pub conf_adjoe: f64,
    pub conf_adjde: f64,
    pub qual_adjoe: f64,
    pub qual_adjde: f64,
    pub qual_barthag: f64,
    pub qual_games: i32,
    pub fun: f64,
    pub conf_pf: f32,
    pub conf_pa: f32,
    pub conf_poss: f64,
    pub conf_adj_o: f64,
    pub conf_adj_d: f64,
    pub conf_sos_remain: f64,
    pub conf_win_perc: f64,
    pub wab: f64,
    pub wab_rank: i32,
    pub fun_rank: i32,
    pub adj_tempo: f64,
}

pub async fn get_team_stats() -> Result<Vec<TeamStats>, Box<dyn Error>> {
    let url = "https://barttorvik.com/2025_team_results.json";
    let response = reqwest::get(url).await?.json::<Vec<TeamStats>>().await?;
    info!("Fetched {} records", response.len());

    Ok(response)
}

pub async fn insert_team_stats(
    session: &Session,
    teams: &[TeamStats],
) -> Result<(), scylla::transport::errors::QueryError> {
    let query = r#"
        INSERT INTO stats.team_stats (
            rank, team, conf, record, adjoe, adjoe_rank, adjde, adjde_rank, barthag, barthag_rank,
            proj_wins, proj_losses, proj_conf_wins, proj_conf_losses, conf_record,
            sos, nconf_sos, conf_sos, proj_sos, proj_nconf_sos, proj_conf_sos,
            elite_sos, elite_ncsos, opp_adjoe, opp_adjde, opp_proj_adjoe, opp_proj_adjde,
            conf_adjoe, conf_adjde, qual_adjoe, qual_adjde, qual_barthag, qual_games,
            fun, conf_pf, conf_pa, conf_poss, conf_adj_o, conf_adj_d, conf_sos_remain,
            conf_win_perc, wab, wab_rank, fun_rank, adj_tempo
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        );
    "#;

    let prepared = session.prepare(query).await?;

    for team in teams {
        session.execute(&prepared, &team).await?;
    }

    Ok(())
}
