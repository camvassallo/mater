use std::error::Error;
use log::{error, info};
use env_logger;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use scylla::{Session, FromRow};

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
    get_all_player_season_averages_from_db
};
use crate::analytics_types::PlayerSeasonAverages; // Import PlayerSeasonAverages

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
            .service(get_player_season_averages_endpoint) // NEW: Add the new endpoint
            .service(hello)
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await.expect("Failed to start API listener");

    Ok(())
}
