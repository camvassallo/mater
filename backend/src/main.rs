use std::error::Error;
use log::{error, info};
use env_logger;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use scylla::{Session, FromRow}; // Added FromRow

// method imports
mod init_db;
mod get_team_stats;
mod get_player_stats; // New module for player stats
mod db_utils;       // New module for database utilities

use crate::get_team_stats::{get_team_stats, insert_team_stats, TeamStats};
use crate::get_player_stats::{get_player_data, insert_player_stats, PlayerStats};
use crate::init_db::init_db;
use crate::db_utils::{connect_to_scylla, query_specific_player, get_players_from_db};


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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    init_db().await.expect("DB setup failed");

    let db = connect_to_scylla().await;

    info!("🚀 Server running at http://localhost:8000");

    let players: Vec<PlayerStats> = get_player_data().await?;
    info!("Players collected: {}", players.len());
    insert_player_stats(&db, &players).await?;

    query_specific_player(&db, "Duke", "Cooper Flagg", 2025).await?;

    let team_stats = get_team_stats().await?;
    info!("Inserting {} team stats into ScyllaDB", team_stats.len());
    insert_team_stats(&db, &team_stats).await?;

    let db_data = web::Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(get_players_endpoint)
            .service(get_team_stats_endpoint)
            .service(hello)
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await.expect("Failed to start API listener");

    Ok(())
}