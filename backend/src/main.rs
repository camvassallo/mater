use std::error::Error;
use log::{error, info};
use env_logger;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use scylla::{Session, FromRow};
use chrono::{Utc, Duration};

mod init_db;
mod get_team_stats;
mod get_player_stats;
mod db_utils;
mod get_game_stats;
mod analytics_types;
mod analytics_calculator;

use crate::get_team_stats::{get_team_stats, insert_team_stats, TeamStats};
use crate::get_player_stats::{get_player_data, insert_player_stats, PlayerStats};
use crate::init_db::init_db;
use crate::db_utils::{connect_to_scylla, query_specific_player, get_players_from_db};
use crate::get_game_stats::{get_game_data, insert_game_stats, GameStats, get_all_game_stats_from_db};
use crate::analytics_calculator::{
    calculate_and_insert_season_averages,
    calculate_and_insert_season_percentiles,
    get_all_player_season_averages_from_db,
    calculate_player_averages_by_date_range
};
use crate::analytics_types::{PlayerSeasonAverages, PlayerStatsWithPercentiles, PlayerSeasonPercentiles};

#[get("/api/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust!")
}

#[get("/api/players")]
async fn get_players_endpoint(
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

    let result = get_players_from_db(&db, team_code, year).await;

    let mut players = match result {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to query Scylla for players: {e}");
            return HttpResponse::InternalServerError().body(format!("Query failed: {e}"));
        }
    };

    info!("Returned {} rows for team {}", players.len(), team_code);

    players.sort_by(|a, b| {
        b.mp.partial_cmp(&a.mp).unwrap_or(std::cmp::Ordering::Equal)
    });

    HttpResponse::Ok().json(players)
}

#[get("/api/team-stats")]
async fn get_team_stats_endpoint(
    db: web::Data<Session>,
) -> impl Responder {
    let query_cql = r#"
        SELECT rank, team, conf, record, adjoe, adjoe_rank, adjde, adjde_rank, barthag, barthag_rank,
               proj_wins, proj_losses, proj_conf_wins, proj_conf_losses, conf_record,
               sos, nconf_sos, conf_sos, proj_sos, proj_nconf_sos, proj_conf_sos,
               elite_sos, elite_ncsos, opp_adjoe, opp_adjde, opp_proj_adjoe, opp_proj_adjde,
               conf_adjoe, conf_adjde, qual_adjoe, qual_adjde, qual_barthag, qual_games,
               fun, conf_pf, conf_pa, conf_poss, conf_adj_o, conf_adj_d, conf_sos_remain,
               conf_win_perc, wab, wab_rank, fun_rank, adj_tempo
        FROM stats.team_stats
    "#;

    let prepared = match db.prepare(query_cql).await {
        Ok(stmt) => stmt,
        Err(e) => {
            error!("Failed to prepare query: {}", e);
            return HttpResponse::InternalServerError().body("Failed to prepare query");
        }
    };

    let result = db.execute(&prepared, ()).await;

    let rows = match result {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query team stats: {}", e);
            return HttpResponse::InternalServerError().body("Query failed");
        }
    };

    let mut stats = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        match TeamStats::from_row(row) {
            Ok(stat) => stats.push(stat),
            Err(e) => error!("Failed to parse row {}: {}", i, e),
        }
    }

    stats.sort_by_key(|s| s.rank);

    HttpResponse::Ok().json(stats)
}

#[get("/api/game-stats")]
async fn get_game_stats_endpoint(
    db: web::Data<Session>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let pid = match query.get("pid") {
        Some(p) => match p.parse::<i32>() {
            Ok(n) => n,
            Err(_) => return HttpResponse::BadRequest().body("Invalid 'pid' query param"),
        },
        None => return HttpResponse::BadRequest().body("Missing 'pid' query param"),
    };

    let year = match query.get("year") {
        Some(y) => match y.parse::<i32>() {
            Ok(n) => n,
            Err(_) => return HttpResponse::BadRequest().body("Invalid 'year' query param"),
        },
        None => return HttpResponse::BadRequest().body("Missing 'year' query param"),
    };

    let team = match query.get("team") {
        Some(t) => t.to_string(),
        None => return HttpResponse::BadRequest().body("Missing 'team' query param"),
    };

    let query_cql = r#"
        SELECT numdate, datetext, opstyle, quality, win1, opponent, muid, win2, min_per, o_rtg, usage,
               e_fg, ts_per, orb_per, drb_per, ast_per, to_per, dunks_made, dunks_att, rim_made,
               rim_att, mid_made, mid_att, two_pm, two_pa, tpm, tpa, ftm, fta, bpm_rd, obpm,
               dbpm, bpm_net, pts, orb, drb, ast, tov, stl, blk, stl_per, blk_per, pf,
               possessions, bpm, sbpm, loc, tt, pp, inches, cls, pid, year
        FROM stats.game_stats WHERE pid = ? AND year = ? AND tt = ?
    "#;

    let prepared = match db.prepare(query_cql).await {
        Ok(stmt) => stmt,
        Err(e) => {
            error!("Failed to prepare query: {}", e);
            return HttpResponse::InternalServerError().body("Failed to prepare query");
        }
    };

    let result = db.execute(&prepared, (pid, year, team.as_str())).await;

    let rows = match result {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query game stats: {}", e);
            return HttpResponse::InternalServerError().body("Query failed");
        }
    };

    let mut game_stats = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        match GameStats::from_row(row) {
            Ok(stat) => game_stats.push(stat),
            Err(e) => error!("Failed to parse row {}: {}", i, e),
        }
    }

    game_stats.sort_by(|a, b| a.numdate.cmp(&b.numdate));

    HttpResponse::Ok().json(game_stats)
}

// NEW API ENDPOINT: Fetch player season averages for a given team and year
#[get("/api/player-season-averages")]
async fn get_player_season_averages_endpoint(
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

    let query_cql = r#"
        SELECT pid, year, team, player_name, games_played, avg_min_per, avg_o_rtg, avg_usg, avg_e_fg, avg_ts_per, avg_orb_per, avg_drb_per, avg_ast_per, avg_to_per, avg_dunks_made, avg_dunks_att, avg_rim_made, avg_rim_att, avg_mid_made, avg_mid_att, avg_two_pm, avg_two_pa, avg_tpm, avg_tpa, avg_ftm, avg_fta, avg_bpm_rd, avg_obpm, avg_dbpm, avg_bpm_net, avg_pts, avg_orb, avg_drb, avg_ast, avg_tov, avg_stl, avg_blk, avg_stl_per, avg_blk_per, avg_pf, avg_possessions, avg_bpm, avg_sbpm, avg_inches, avg_opstyle, avg_quality, avg_win1, avg_win2
        FROM stats.player_season_avg_stats WHERE team = ? AND year = ?
    "#;

    let prepared = match db.prepare(query_cql).await {
        Ok(stmt) => stmt,
        Err(e) => {
            error!("Failed to prepare query for player season averages: {}", e);
            return HttpResponse::InternalServerError().body("Failed to prepare query");
        }
    };

    let result = db.execute(&prepared, (team_code, year)).await;

    let rows = match result {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query player season averages: {}", e);
            return HttpResponse::InternalServerError().body("Query failed");
        }
    };

    let mut player_averages = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        match PlayerSeasonAverages::from_row(row) {
            Ok(avg) => player_averages.push(avg),
            Err(e) => error!("Failed to parse player season average row {}: {}", i, e),
        }
    }

    HttpResponse::Ok().json(player_averages)
}

// NEW API ENDPOINT: Fetch player rolling averages for the last N days (default 30)
#[get("/api/player-rolling-averages")]
async fn get_player_rolling_averages_endpoint(
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

    let last_n_days = match query.get("last_n_days") {
        Some(d) => match d.parse::<i64>() {
            Ok(n) => n,
            Err(_) => return HttpResponse::BadRequest().body("Invalid 'last_n_days' query param"),
        },
        None => 30, // Default to last 30 days
    };

    info!("Fetching rolling averages for team: {}, year: {}, last {} days", team_code, year, last_n_days);

    // Calculate date range
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(last_n_days);
    let start_date_str = start_date.format("%Y%m%d").to_string();
    let end_date_str = end_date.format("%Y%m%d").to_string();

    info!("Date range: {} to {}", start_date_str, end_date_str);

    // Fetch all game stats from database
    let all_game_stats = match get_all_game_stats_from_db(&db).await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to fetch game stats from database: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to fetch game stats: {}", e));
        }
    };

    // Find all unique players for this team and year
    let mut player_keys: std::collections::HashSet<(i32, String)> = std::collections::HashSet::new();
    for game in &all_game_stats {
        if let Some(pid) = game.pid {
            if game.year == Some(year) && game.tt == *team_code {
                player_keys.insert((pid, game.pp.clone()));
            }
        }
    }

    info!("Found {} unique players for team {} in year {}", player_keys.len(), team_code, year);

    // Calculate rolling averages for each player
    let mut rolling_averages = Vec::new();
    for (pid, player_name) in player_keys {
        if let Some(avg) = calculate_player_averages_by_date_range(
            &all_game_stats,
            pid,
            year,
            team_code,
            &start_date_str,
            &end_date_str,
        ) {
            // Create PlayerRollingAverages with optional fields
            let mut rolling_avg = analytics_types::PlayerRollingAverages {
                averages: avg,
                conf: None,
                player_type: None,
                yr: None,
                ht: None,
                porpag: None,
                dporpag: None,
                drtg: None,
                adjoe: None,
            };

            // Fetch season-long constants from player_stats table
            let query_player_stats = r#"
                SELECT conf, player_type, yr, ht, porpag, dporpag, drtg, adjoe
                FROM stats.player_stats WHERE pid = ? AND year = ? AND team = ? ALLOW FILTERING
            "#;

            if let Ok(prepared) = db.prepare(query_player_stats).await {
                if let Ok(result) = db.execute(&prepared, (pid, year, team_code)).await {
                    if let Some(rows) = result.rows {
                        if let Some(row) = rows.into_iter().next() {
                            // Extract the fields manually (string fields)
                            if let Some(conf) = row.columns[0].as_ref().and_then(|v| v.as_text()).map(|s| s.to_string()) {
                                rolling_avg.conf = Some(conf);
                            }
                            if let Some(player_type) = row.columns[1].as_ref().and_then(|v| v.as_text()).map(|s| s.to_string()) {
                                rolling_avg.player_type = Some(player_type);
                            }
                            if let Some(yr) = row.columns[2].as_ref().and_then(|v| v.as_text()).map(|s| s.to_string()) {
                                rolling_avg.yr = Some(yr);
                            }
                            if let Some(ht) = row.columns[3].as_ref().and_then(|v| v.as_text()).map(|s| s.to_string()) {
                                rolling_avg.ht = Some(ht);
                            }
                            // Extract numeric fields
                            if let Some(porpag) = row.columns[4].as_ref().and_then(|v| v.as_double()) {
                                rolling_avg.porpag = Some(porpag);
                            }
                            if let Some(dporpag) = row.columns[5].as_ref().and_then(|v| v.as_double()) {
                                rolling_avg.dporpag = Some(dporpag);
                            }
                            if let Some(drtg) = row.columns[6].as_ref().and_then(|v| v.as_double()) {
                                rolling_avg.drtg = Some(drtg);
                            }
                            if let Some(adjoe) = row.columns[7].as_ref().and_then(|v| v.as_double()) {
                                rolling_avg.adjoe = Some(adjoe);
                            }
                        }
                    }
                }
            }

            rolling_averages.push(rolling_avg);
        } else {
            info!("No games found for player {} (PID: {}) in the specified date range", player_name, pid);
        }
    }

    info!("Calculated rolling averages for {} players", rolling_averages.len());

    // Calculate percentiles for all stats
    info!("Calculating percentiles for rolling averages...");

    // Collect all values for each stat (for percentile calculation)
    let mut all_min_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_min_per).collect();
    let mut all_o_rtg: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_o_rtg).collect();
    let mut all_usg: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_usg).collect();
    let mut all_e_fg: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_e_fg).collect();
    let mut all_ts_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_ts_per).collect();
    let mut all_orb_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_orb_per).collect();
    let mut all_drb_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_drb_per).collect();
    let mut all_ast_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_ast_per).collect();
    let mut all_to_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_to_per).collect();
    let mut all_pts: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_pts).collect();
    let mut all_orb: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_orb).collect();
    let mut all_drb: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_drb).collect();
    let mut all_ast: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_ast).collect();
    let mut all_stl: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_stl).collect();
    let mut all_blk: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_blk).collect();
    let mut all_stl_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_stl_per).collect();
    let mut all_blk_per: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_blk_per).collect();
    let mut all_bpm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_bpm).collect();
    let mut all_obpm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_obpm).collect();
    let mut all_dbpm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_dbpm).collect();
    let mut all_dunks_made: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_dunks_made).collect();
    let mut all_dunks_att: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_dunks_att).collect();
    let mut all_rim_made: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_rim_made).collect();
    let mut all_rim_att: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_rim_att).collect();
    let mut all_mid_made: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_mid_made).collect();
    let mut all_mid_att: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_mid_att).collect();
    let mut all_two_pm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_two_pm).collect();
    let mut all_two_pa: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_two_pa).collect();
    let mut all_tpm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_tpm).collect();
    let mut all_tpa: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_tpa).collect();
    let mut all_ftm: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_ftm).collect();
    let mut all_fta: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_fta).collect();
    let mut all_tov: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_tov).collect();
    let mut all_pf: Vec<f64> = rolling_averages.iter().map(|p| p.averages.avg_pf).collect();

    // Collect season-long stats (these are optional)
    let mut all_porpag: Vec<f64> = rolling_averages.iter().filter_map(|p| p.porpag).collect();
    let mut all_dporpag: Vec<f64> = rolling_averages.iter().filter_map(|p| p.dporpag).collect();
    let mut all_drtg: Vec<f64> = rolling_averages.iter().filter_map(|p| p.drtg).collect();
    let mut all_adjoe: Vec<f64> = rolling_averages.iter().filter_map(|p| p.adjoe).collect();

    // Sort all vectors for percentile calculation
    all_min_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_o_rtg.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_usg.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_e_fg.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_ts_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_orb_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_drb_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_ast_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_to_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_pts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_orb.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_drb.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_ast.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_stl.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_blk.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_stl_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_blk_per.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_bpm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_obpm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_dbpm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_dunks_made.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_dunks_att.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_rim_made.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_rim_att.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_mid_made.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_mid_att.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_two_pm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_two_pa.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_tpm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_tpa.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_ftm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_fta.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_tov.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_pf.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_porpag.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_dporpag.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_drtg.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_adjoe.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Create PlayerRollingAveragesWithPercentiles for each player
    let rolling_with_percentiles: Vec<analytics_types::PlayerRollingAveragesWithPercentiles> = rolling_averages.into_iter().map(|rolling_avg| {
        analytics_types::PlayerRollingAveragesWithPercentiles {
            pct_min_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_min_per, &all_min_per)),
            pct_o_rtg: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_o_rtg, &all_o_rtg)),
            pct_usg: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_usg, &all_usg)),
            pct_e_fg: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_e_fg, &all_e_fg)),
            pct_ts_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_ts_per, &all_ts_per)),
            pct_orb_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_orb_per, &all_orb_per)),
            pct_drb_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_drb_per, &all_drb_per)),
            pct_ast_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_ast_per, &all_ast_per)),
            pct_to_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_to_per, &all_to_per)),
            pct_pts: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_pts, &all_pts)),
            pct_orb: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_orb, &all_orb)),
            pct_drb: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_drb, &all_drb)),
            pct_ast: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_ast, &all_ast)),
            pct_stl: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_stl, &all_stl)),
            pct_blk: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_blk, &all_blk)),
            pct_stl_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_stl_per, &all_stl_per)),
            pct_blk_per: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_blk_per, &all_blk_per)),
            pct_bpm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_bpm, &all_bpm)),
            pct_obpm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_obpm, &all_obpm)),
            pct_dbpm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_dbpm, &all_dbpm)),
            pct_dunks_made: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_dunks_made, &all_dunks_made)),
            pct_dunks_att: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_dunks_att, &all_dunks_att)),
            pct_rim_made: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_rim_made, &all_rim_made)),
            pct_rim_att: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_rim_att, &all_rim_att)),
            pct_mid_made: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_mid_made, &all_mid_made)),
            pct_mid_att: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_mid_att, &all_mid_att)),
            pct_two_pm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_two_pm, &all_two_pm)),
            pct_two_pa: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_two_pa, &all_two_pa)),
            pct_tpm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_tpm, &all_tpm)),
            pct_tpa: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_tpa, &all_tpa)),
            pct_ftm: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_ftm, &all_ftm)),
            pct_fta: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_fta, &all_fta)),
            pct_tov: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_tov, &all_tov)),
            pct_pf: Some(analytics_calculator::calculate_percentile(rolling_avg.averages.avg_pf, &all_pf)),
            // Season-long stat percentiles (optional)
            pct_porpag: rolling_avg.porpag.map(|v| analytics_calculator::calculate_percentile(v, &all_porpag)),
            pct_dporpag: rolling_avg.dporpag.map(|v| analytics_calculator::calculate_percentile(v, &all_dporpag)),
            pct_drtg: rolling_avg.drtg.map(|v| analytics_calculator::calculate_percentile(v, &all_drtg)),
            pct_adjoe: rolling_avg.adjoe.map(|v| analytics_calculator::calculate_percentile(v, &all_adjoe)),
            rolling_avg,
        }
    }).collect();

    info!("Calculated percentiles for {} players", rolling_with_percentiles.len());

    HttpResponse::Ok().json(rolling_with_percentiles)
}

// NEW API ENDPOINT: Fetch player season averages with percentiles
#[get("/api/player-stats-with-percentiles")]
async fn get_player_stats_with_percentiles_endpoint(
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

    info!("Fetching player stats with percentiles for team: {}, year: {}", team_code, year);

    // Fetch averages from database
    let query_cql_avg = r#"
        SELECT pid, year, team, player_name, games_played, avg_min_per, avg_o_rtg, avg_usg, avg_e_fg, avg_ts_per, avg_orb_per, avg_drb_per, avg_ast_per, avg_to_per, avg_dunks_made, avg_dunks_att, avg_rim_made, avg_rim_att, avg_mid_made, avg_mid_att, avg_two_pm, avg_two_pa, avg_tpm, avg_tpa, avg_ftm, avg_fta, avg_bpm_rd, avg_obpm, avg_dbpm, avg_bpm_net, avg_pts, avg_orb, avg_drb, avg_ast, avg_tov, avg_stl, avg_blk, avg_stl_per, avg_blk_per, avg_pf, avg_possessions, avg_bpm, avg_sbpm, avg_inches, avg_opstyle, avg_quality, avg_win1, avg_win2
        FROM stats.player_season_avg_stats WHERE team = ? AND year = ? ALLOW FILTERING
    "#;

    let prepared_avg = match db.prepare(query_cql_avg).await {
        Ok(stmt) => stmt,
        Err(e) => {
            error!("Failed to prepare query for player season averages: {}", e);
            return HttpResponse::InternalServerError().body("Failed to prepare query");
        }
    };

    let result_avg = db.execute(&prepared_avg, (team_code, year)).await;

    let rows_avg = match result_avg {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query player season averages: {}", e);
            return HttpResponse::InternalServerError().body("Query failed");
        }
    };

    let mut player_averages = Vec::new();
    for (i, row) in rows_avg.into_iter().enumerate() {
        match PlayerSeasonAverages::from_row(row) {
            Ok(avg) => player_averages.push(avg),
            Err(e) => error!("Failed to parse player season average row {}: {}", i, e),
        }
    }

    // Fetch percentiles from database
    let query_cql_pct = r#"
        SELECT pid, year, team, player_name, pct_min_per, pct_o_rtg, pct_usg, pct_e_fg, pct_ts_per, pct_orb_per, pct_drb_per, pct_ast_per, pct_to_per, pct_dunks_made, pct_dunks_att, pct_rim_made, pct_rim_att, pct_mid_made, pct_mid_att, pct_two_pm, pct_two_pa, pct_tpm, pct_tpa, pct_ftm, pct_fta, pct_bpm_rd, pct_obpm, pct_dbpm, pct_bpm_net, pct_pts, pct_orb, pct_drb, pct_ast, pct_tov, pct_stl, pct_blk, pct_stl_per, pct_blk_per, pct_pf, pct_possessions, pct_bpm, pct_sbpm, pct_inches, pct_opstyle, pct_quality, pct_win1, pct_win2
        FROM stats.player_season_percentiles WHERE team = ? AND year = ? ALLOW FILTERING
    "#;

    let prepared_pct = match db.prepare(query_cql_pct).await {
        Ok(stmt) => stmt,
        Err(e) => {
            error!("Failed to prepare query for player season percentiles: {}", e);
            return HttpResponse::InternalServerError().body("Failed to prepare query");
        }
    };

    let result_pct = db.execute(&prepared_pct, (team_code, year)).await;

    let rows_pct = match result_pct {
        Ok(res) => res.rows.unwrap_or_default(),
        Err(e) => {
            error!("Failed to query player season percentiles: {}", e);
            return HttpResponse::InternalServerError().body("Query failed");
        }
    };

    let mut player_percentiles = Vec::new();
    for (i, row) in rows_pct.into_iter().enumerate() {
        match PlayerSeasonPercentiles::from_row(row) {
            Ok(pct) => player_percentiles.push(pct),
            Err(e) => error!("Failed to parse player season percentile row {}: {}", i, e),
        }
    }

    // Create a HashMap for quick lookup of percentiles by pid
    let percentiles_map: std::collections::HashMap<i32, &PlayerSeasonPercentiles> =
        player_percentiles.iter().map(|p| (p.pid, p)).collect();

    // Combine averages and percentiles
    let mut combined_stats = Vec::new();
    for avg in player_averages {
        if let Some(pct) = percentiles_map.get(&avg.pid) {
            combined_stats.push(PlayerStatsWithPercentiles {
                pid: avg.pid,
                year: avg.year,
                team: avg.team.clone(),
                player_name: avg.player_name.clone(),
                games_played: avg.games_played,
                avg_min_per: avg.avg_min_per,
                avg_o_rtg: avg.avg_o_rtg,
                avg_usg: avg.avg_usg,
                avg_e_fg: avg.avg_e_fg,
                avg_ts_per: avg.avg_ts_per,
                avg_orb_per: avg.avg_orb_per,
                avg_drb_per: avg.avg_drb_per,
                avg_ast_per: avg.avg_ast_per,
                avg_to_per: avg.avg_to_per,
                avg_dunks_made: avg.avg_dunks_made,
                avg_dunks_att: avg.avg_dunks_att,
                avg_rim_made: avg.avg_rim_made,
                avg_rim_att: avg.avg_rim_att,
                avg_mid_made: avg.avg_mid_made,
                avg_mid_att: avg.avg_mid_att,
                avg_two_pm: avg.avg_two_pm,
                avg_two_pa: avg.avg_two_pa,
                avg_tpm: avg.avg_tpm,
                avg_tpa: avg.avg_tpa,
                avg_ftm: avg.avg_ftm,
                avg_fta: avg.avg_fta,
                avg_bpm_rd: avg.avg_bpm_rd,
                avg_obpm: avg.avg_obpm,
                avg_dbpm: avg.avg_dbpm,
                avg_bpm_net: avg.avg_bpm_net,
                avg_pts: avg.avg_pts,
                avg_orb: avg.avg_orb,
                avg_drb: avg.avg_drb,
                avg_ast: avg.avg_ast,
                avg_tov: avg.avg_tov,
                avg_stl: avg.avg_stl,
                avg_blk: avg.avg_blk,
                avg_stl_per: avg.avg_stl_per,
                avg_blk_per: avg.avg_blk_per,
                avg_pf: avg.avg_pf,
                avg_possessions: avg.avg_possessions,
                avg_bpm: avg.avg_bpm,
                avg_sbpm: avg.avg_sbpm,
                avg_inches: avg.avg_inches,
                avg_opstyle: avg.avg_opstyle,
                avg_quality: avg.avg_quality,
                avg_win1: avg.avg_win1,
                avg_win2: avg.avg_win2,
                pct_min_per: pct.pct_min_per,
                pct_o_rtg: pct.pct_o_rtg,
                pct_usg: pct.pct_usg,
                pct_e_fg: pct.pct_e_fg,
                pct_ts_per: pct.pct_ts_per,
                pct_orb_per: pct.pct_orb_per,
                pct_drb_per: pct.pct_drb_per,
                pct_ast_per: pct.pct_ast_per,
                pct_to_per: pct.pct_to_per,
                pct_dunks_made: pct.pct_dunks_made,
                pct_dunks_att: pct.pct_dunks_att,
                pct_rim_made: pct.pct_rim_made,
                pct_rim_att: pct.pct_rim_att,
                pct_mid_made: pct.pct_mid_made,
                pct_mid_att: pct.pct_mid_att,
                pct_two_pm: pct.pct_two_pm,
                pct_two_pa: pct.pct_two_pa,
                pct_tpm: pct.pct_tpm,
                pct_tpa: pct.pct_tpa,
                pct_ftm: pct.pct_ftm,
                pct_fta: pct.pct_fta,
                pct_bpm_rd: pct.pct_bpm_rd,
                pct_obpm: pct.pct_obpm,
                pct_dbpm: pct.pct_dbpm,
                pct_bpm_net: pct.pct_bpm_net,
                pct_pts: pct.pct_pts,
                pct_orb: pct.pct_orb,
                pct_drb: pct.pct_drb,
                pct_ast: pct.pct_ast,
                pct_tov: pct.pct_tov,
                pct_stl: pct.pct_stl,
                pct_blk: pct.pct_blk,
                pct_stl_per: pct.pct_stl_per,
                pct_blk_per: pct.pct_blk_per,
                pct_pf: pct.pct_pf,
                pct_possessions: pct.pct_possessions,
                pct_bpm: pct.pct_bpm,
                pct_sbpm: pct.pct_sbpm,
                pct_inches: pct.pct_inches,
                pct_opstyle: pct.pct_opstyle,
                pct_quality: pct.pct_quality,
                pct_win1: pct.pct_win1,
                pct_win2: pct.pct_win2,
            });
        } else {
            info!("No percentile data found for player {} (PID: {})", avg.player_name, avg.pid);
        }
    }

    info!("Returning {} combined player stats with percentiles", combined_stats.len());

    HttpResponse::Ok().json(combined_stats)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    init_db().await.expect("DB setup failed");

    let db = connect_to_scylla().await;

    info!("🚀 Server running at http://localhost:8000");

    // Set this to `true` to skip fetching and inserting data for faster testing.
    // Ensure your ScyllaDB instance already has data if you set this to `true`.
    const SKIP_DATA_LOADING: bool = true; // Set to `true` to skip data loading

    #[allow(unused_assignments)] // Suppress warning about game_stats being assigned but not directly read after assignment
    let mut game_stats: Vec<GameStats> = Vec::new(); // Declare game_stats mutably outside the if block

    if !SKIP_DATA_LOADING {
        let players: Vec<PlayerStats> = get_player_data().await?;
        info!("Players collected: {}", players.len());
        insert_player_stats(&db, &players).await?;

        query_specific_player(&db, "Duke", "Cooper Flagg", 2025).await?;

        let team_stats = get_team_stats().await?;
        info!("Inserting {} team stats into ScyllaDB", team_stats.len());
        insert_team_stats(&db, &team_stats).await?;

        game_stats = get_game_data().await?; // Assign to the outer game_stats
        info!("Inserting {} game stats into ScyllaDB", game_stats.len());
        insert_game_stats(&db, &game_stats).await?;
    } else {
        info!("Skipping initial data loading and insertion as SKIP_DATA_LOADING is true.");
        // Fetch game_stats from DB when skipping initial loading, so analytics can still run.
        game_stats = get_all_game_stats_from_db(&db).await?;
    }


    // Calculate and insert player season averages
    info!("Starting player season average calculation...");
    calculate_and_insert_season_averages(&db, &game_stats).await?;
    info!("Finished player season average calculation.");

    // Calculate and insert player season percentiles
    info!("Starting player season percentile calculation...");
    // Fetch averages for percentile calculation
    let all_season_averages = get_all_player_season_averages_from_db(&db).await?;
    calculate_and_insert_season_percentiles(&db, &all_season_averages).await?;
    info!("Finished player season percentile calculation.");


    let db_data = web::Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(get_players_endpoint)
            .service(get_team_stats_endpoint)
            .service(get_game_stats_endpoint)
            .service(get_player_season_averages_endpoint)
            .service(get_player_rolling_averages_endpoint)
            .service(get_player_stats_with_percentiles_endpoint)
            .service(hello)
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await.expect("Failed to start API listener");

    Ok(())
}
