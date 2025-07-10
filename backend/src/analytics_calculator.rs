use std::collections::HashMap;
use log::{info, error};
use scylla::Session;
// Removed: use scyyla::transport::errors::QueryError; // No longer directly used here

use crate::get_game_stats::GameStats;
use crate::analytics_types::PlayerSeasonAverages;

/// Calculates and inserts player season average statistics into ScyllaDB.
/// This function groups game stats by player and year, computes averages,
/// and then persists them.
pub async fn calculate_and_insert_season_averages(
    session: &Session,
    all_game_stats: &[GameStats],
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Calculating player season averages...");

    // Group game stats by (pid, year, team)
    // HashMap<(pid, year, team), Vec<GameStats>>
    let mut player_season_games: HashMap<(i32, i32, String), Vec<&GameStats>> = HashMap::new();

    for game in all_game_stats {
        if let (Some(pid), Some(year)) = (game.pid, game.year) {
            // Ensure team is not empty for the key
            if !game.tt.is_empty() {
                player_season_games.entry((pid, year, game.tt.clone()))
                    .or_default()
                    .push(game);
            } else {
                error!("Skipping game record due to empty team name (tt) for player PID: {:?}, Year: {:?}", game.pid, game.year);
            }
        } else {
            error!("Skipping game record due to missing PID or Year: PID {:?}, Year {:?}", game.pid, game.year);
        }
    }

    let mut season_averages: Vec<PlayerSeasonAverages> = Vec::new();

    for ((pid, year, team), games) in player_season_games {
        let games_played = games.len() as i32;
        if games_played == 0 {
            continue; // Should not happen if entry exists, but good safeguard
        }

        let mut sum_min_per = 0.0;
        let mut sum_o_rtg = 0.0;
        let mut sum_usg = 0.0;
        let mut sum_e_fg = 0.0;
        let mut sum_ts_per = 0.0;
        let mut sum_orb_per = 0.0;
        let mut sum_drb_per = 0.0;
        let mut sum_ast_per = 0.0;
        let mut sum_to_per = 0.0;
        let mut sum_dunks_made = 0.0;
        let mut sum_dunks_att = 0.0;
        let mut sum_rim_made = 0.0;
        let mut sum_rim_att = 0.0;
        let mut sum_mid_made = 0.0;
        let mut sum_mid_att = 0.0;
        let mut sum_two_pm = 0.0;
        let mut sum_two_pa = 0.0;
        let mut sum_tpm = 0.0;
        let mut sum_tpa = 0.0;
        let mut sum_ftm = 0.0;
        let mut sum_fta = 0.0;
        let mut sum_bpm_rd = 0.0;
        let mut sum_obpm = 0.0;
        let mut sum_dbpm = 0.0;
        let mut sum_bpm_net = 0.0;
        let mut sum_pts = 0.0;
        let mut sum_orb = 0.0;
        let mut sum_drb = 0.0;
        let mut sum_ast = 0.0;
        let mut sum_tov = 0.0;
        let mut sum_stl = 0.0;
        let mut sum_blk = 0.0;
        let mut sum_stl_per = 0.0;
        let mut sum_blk_per = 0.0;
        let mut sum_pf = 0.0;
        let mut sum_possessions = 0.0;
        let mut sum_bpm = 0.0;
        let mut sum_sbpm = 0.0;
        let mut sum_inches = 0.0;
        let mut sum_opstyle = 0.0;
        let mut sum_quality = 0.0;
        let mut sum_win1 = 0.0;
        let mut sum_win2 = 0.0;

        let mut player_name = "Unknown".to_string(); // Default, will be updated

        for game in games {
            // Use unwrap_or_default() for Option<f64> and Option<i32> fields
            // For integer-like fields, cast to f64 for summation
            sum_min_per += game.min_per.unwrap_or_default();
            sum_o_rtg += game.o_rtg.unwrap_or_default();
            sum_usg += game.usage.unwrap_or_default();
            sum_e_fg += game.e_fg.unwrap_or_default();
            sum_ts_per += game.ts_per.unwrap_or_default();
            sum_orb_per += game.orb_per.unwrap_or_default();
            sum_drb_per += game.drb_per.unwrap_or_default();
            sum_ast_per += game.ast_per.unwrap_or_default();
            sum_to_per += game.to_per.unwrap_or_default();
            sum_dunks_made += game.dunks_made.unwrap_or_default() as f64;
            sum_dunks_att += game.dunks_att.unwrap_or_default() as f64;
            sum_rim_made += game.rim_made.unwrap_or_default() as f64;
            sum_rim_att += game.rim_att.unwrap_or_default() as f64;
            sum_mid_made += game.mid_made.unwrap_or_default() as f64;
            sum_mid_att += game.mid_att.unwrap_or_default() as f64;
            sum_two_pm += game.two_pm.unwrap_or_default() as f64;
            sum_two_pa += game.two_pa.unwrap_or_default() as f64;
            sum_tpm += game.tpm.unwrap_or_default() as f64;
            sum_tpa += game.tpa.unwrap_or_default() as f64;
            sum_ftm += game.ftm.unwrap_or_default() as f64;
            sum_fta += game.fta.unwrap_or_default() as f64;
            sum_bpm_rd += game.bpm_rd.unwrap_or_default();
            sum_obpm += game.obpm.unwrap_or_default();
            sum_dbpm += game.dbpm.unwrap_or_default();
            sum_bpm_net += game.bpm_net.unwrap_or_default();
            sum_pts += game.pts.unwrap_or_default();
            sum_orb += game.orb.unwrap_or_default();
            sum_drb += game.drb.unwrap_or_default();
            sum_ast += game.ast.unwrap_or_default();
            sum_tov += game.tov.unwrap_or_default();
            sum_stl += game.stl.unwrap_or_default();
            sum_blk += game.blk.unwrap_or_default();
            sum_stl_per += game.stl_per.unwrap_or_default();
            sum_blk_per += game.blk_per.unwrap_or_default();
            sum_pf += game.pf.unwrap_or_default();
            sum_possessions += game.possessions.unwrap_or_default();
            sum_bpm += game.bpm.unwrap_or_default();
            sum_sbpm += game.sbpm.unwrap_or_default();
            sum_inches += game.inches.unwrap_or_default() as f64;
            sum_opstyle += game.opstyle.unwrap_or_default() as f64;
            sum_quality += game.quality.unwrap_or_default() as f64;
            sum_win1 += game.win1.unwrap_or_default() as f64;
            sum_win2 += game.win2.unwrap_or_default() as f64;

            // Update player name (should be consistent across games for a player/team/season)
            if !game.pp.is_empty() {
                player_name = game.pp.clone();
            }
        }

        let avg_games_played = games_played as f64;

        season_averages.push(PlayerSeasonAverages {
            pid,
            year,
            team: team.clone(),
            player_name: player_name.clone(),
            games_played,
            avg_min_per: sum_min_per / avg_games_played,
            avg_o_rtg: sum_o_rtg / avg_games_played,
            avg_usg: sum_usg / avg_games_played,
            avg_e_fg: sum_e_fg / avg_games_played,
            avg_ts_per: sum_ts_per / avg_games_played,
            avg_orb_per: sum_orb_per / avg_games_played,
            avg_drb_per: sum_drb_per / avg_games_played,
            avg_ast_per: sum_ast_per / avg_games_played,
            avg_to_per: sum_to_per / avg_games_played,
            avg_dunks_made: sum_dunks_made / avg_games_played,
            avg_dunks_att: sum_dunks_att / avg_games_played,
            avg_rim_made: sum_rim_made / avg_games_played,
            avg_rim_att: sum_rim_att / avg_games_played,
            avg_mid_made: sum_mid_made / avg_games_played,
            avg_mid_att: sum_mid_att / avg_games_played,
            avg_two_pm: sum_two_pm / avg_games_played,
            avg_two_pa: sum_two_pa / avg_games_played,
            avg_tpm: sum_tpm / avg_games_played,
            avg_tpa: sum_tpa / avg_games_played,
            avg_ftm: sum_ftm / avg_games_played,
            avg_fta: sum_fta / avg_games_played,
            avg_bpm_rd: sum_bpm_rd / avg_games_played,
            avg_obpm: sum_obpm / avg_games_played,
            avg_dbpm: sum_dbpm / avg_games_played,
            avg_bpm_net: sum_bpm_net / avg_games_played,
            avg_pts: sum_pts / avg_games_played,
            avg_orb: sum_orb / avg_games_played,
            avg_drb: sum_drb / avg_games_played,
            avg_ast: sum_ast / avg_games_played,
            avg_tov: sum_tov / avg_games_played,
            avg_stl: sum_stl / avg_games_played,
            avg_blk: sum_blk / avg_games_played,
            avg_stl_per: sum_stl_per / avg_games_played,
            avg_blk_per: sum_blk_per / avg_games_played,
            avg_pf: sum_pf / avg_games_played,
            avg_possessions: sum_possessions / avg_games_played,
            avg_bpm: sum_bpm / avg_games_played,
            avg_sbpm: sum_sbpm / avg_games_played,
            avg_inches: sum_inches / avg_games_played,
            avg_opstyle: sum_opstyle / avg_games_played,
            avg_quality: sum_quality / avg_games_played,
            avg_win1: sum_win1 / avg_games_played,
            avg_win2: sum_win2 / avg_games_played,
        });
    }

    info!("Inserting {} player season average records into ScyllaDB", season_averages.len());
    let query = r#"
        INSERT INTO stats.player_season_avg_stats (
            pid, year, team, player_name, games_played, avg_min_per, avg_o_rtg, avg_usg, avg_e_fg, avg_ts_per, avg_orb_per, avg_drb_per, avg_ast_per, avg_to_per, avg_dunks_made, avg_dunks_att, avg_rim_made, avg_rim_att, avg_mid_made, avg_mid_att, avg_two_pm, avg_two_pa, avg_tpm, avg_tpa, avg_ftm, avg_fta, avg_bpm_rd, avg_obpm, avg_dbpm, avg_bpm_net, avg_pts, avg_orb, avg_drb, avg_ast, avg_tov, avg_stl, avg_blk, avg_stl_per, avg_blk_per, avg_pf, avg_possessions, avg_bpm, avg_sbpm, avg_inches, avg_opstyle, avg_quality, avg_win1, avg_win2
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
    "#;

    let prepared = session.prepare(query).await?;

    for avg in season_averages {
        session.execute(&prepared, &avg).await?;
    }

    Ok(())
}
