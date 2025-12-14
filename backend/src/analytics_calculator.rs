use std::collections::HashMap;
use log::{info, error};
use scylla::Session;
use scylla::query::Query;
use futures_util::stream::StreamExt;
use std::time::Duration;
use scylla::FromRow;

use crate::get_game_stats::GameStats;
use crate::analytics_types::{PlayerSeasonAverages, PlayerSeasonPercentiles};

/// Calculates average statistics for a given slice of GameStats,
/// *only including games where the player logged minutes*.
///
/// This is a reusable helper function for both full season averages and "slice" averages
/// (e.g., last 5 games, games within a date range).
///
/// Parameters:
/// - `games_raw`: A slice of `GameStats` records, potentially unfiltered.
/// - `player_pid`: The Player ID for whom averages are being calculated.
/// - `player_year`: The season year for the player.
/// - `player_team`: The team name for the player (passed as a string slice).
/// - `player_name`: The player's name (passed as a string slice).
///
/// Returns:
/// - `Option<PlayerSeasonAverages>`: `Some` with the calculated averages if games with minutes are found,
///   `None` otherwise.
pub fn calculate_averages_for_games(
    games_raw: &[&GameStats],
    player_pid: i32,
    player_year: i32,
    player_team: &str,
    player_name: &str,
) -> Option<PlayerSeasonAverages> {

    // Filter games to only include those where the player logged minutes.
    // This ensures that averages are calculated only for games where the player actually participated.
    let games: Vec<&&GameStats> = games_raw.iter()
        .filter(|&game| game.min_per.unwrap_or_default() > 0.0)
        .collect();

    let games_played = games.len() as i32;
    if games_played == 0 {
        // If after filtering for minutes, no games remain, return None.
        // This prevents division by zero and indicates no meaningful averages can be computed.
        return None;
    }

    let avg_games_played = games_played as f64;

    // Initialize sums for raw totals (used for calculating overall percentages/ratios)
    let mut total_dunks_made = 0.0;
    let mut total_dunks_att = 0.0;
    let mut total_rim_made = 0.0;
    let mut total_rim_att = 0.0;
    let mut total_mid_made = 0.0;
    let mut total_mid_att = 0.0;
    let mut total_two_pm = 0.0;
    let mut total_two_pa = 0.0;
    let mut total_tpm = 0.0;
    let mut total_tpa = 0.0;
    let mut total_ftm = 0.0;
    let mut total_fta = 0.0;
    let mut total_pts = 0.0;
    let mut total_orb = 0.0;
    let mut total_drb = 0.0;
    let mut total_ast = 0.0;
    let mut total_tov = 0.0;
    let mut total_stl = 0.0;
    let mut total_blk = 0.0;
    let mut total_pf = 0.0;
    let mut total_possessions = 0.0;
    let mut total_inches = 0.0;
    let mut total_opstyle = 0.0;
    let mut total_quality = 0.0;
    let mut total_win1 = 0.0;
    let mut total_win2 = 0.0;

    // Initialize sums for simple averages of already calculated per-game percentages/rates or complex stats
    let mut sum_min_per = 0.0;
    let mut sum_o_rtg = 0.0;
    let mut sum_usg = 0.0;
    let mut sum_bpm_rd = 0.0;
    let mut sum_obpm = 0.0;
    let mut sum_dbpm = 0.0;
    let mut sum_bpm_net = 0.0;
    let mut sum_bpm = 0.0;
    let mut sum_sbpm = 0.0;
    let mut sum_orb_per_per_game = 0.0;
    let mut sum_drb_per_per_game = 0.0;
    let mut sum_ast_per_per_game = 0.0;
    let mut sum_to_per_per_game = 0.0;
    let mut sum_stl_per_per_game = 0.0;
    let mut sum_blk_per_per_game = 0.0;

    // Iterate over the filtered games (only games with minutes played) to sum up statistics
    for game in games {
        total_dunks_made += game.dunks_made.unwrap_or_default() as f64;
        total_dunks_att += game.dunks_att.unwrap_or_default() as f64;
        total_rim_made += game.rim_made.unwrap_or_default() as f64;
        total_rim_att += game.rim_att.unwrap_or_default() as f64;
        total_mid_made += game.mid_made.unwrap_or_default() as f64;
        total_mid_att += game.mid_att.unwrap_or_default() as f64;
        total_two_pm += game.two_pm.unwrap_or_default() as f64;
        total_two_pa += game.two_pa.unwrap_or_default() as f64;
        total_tpm += game.tpm.unwrap_or_default() as f64;
        total_tpa += game.tpa.unwrap_or_default() as f64;
        total_ftm += game.ftm.unwrap_or_default() as f64;
        total_fta += game.fta.unwrap_or_default() as f64;
        total_pts += game.pts.unwrap_or_default();
        total_orb += game.orb.unwrap_or_default();
        total_drb += game.drb.unwrap_or_default();
        total_ast += game.ast.unwrap_or_default();
        total_tov += game.tov.unwrap_or_default();
        total_stl += game.stl.unwrap_or_default();
        total_blk += game.blk.unwrap_or_default();
        total_pf += game.pf.unwrap_or_default();
        total_possessions += game.possessions.unwrap_or_default();
        total_inches += game.inches.unwrap_or_default() as f64;
        total_opstyle += game.opstyle.unwrap_or_default() as f64;
        total_quality += game.quality.unwrap_or_default() as f64;
        total_win1 += game.win1.unwrap_or_default() as f64;
        total_win2 += game.win2.unwrap_or_default() as f64;

        sum_min_per += game.min_per.unwrap_or_default();
        sum_o_rtg += game.o_rtg.unwrap_or_default();
        sum_usg += game.usage.unwrap_or_default();
        sum_bpm_rd += game.bpm_rd.unwrap_or_default();
        sum_obpm += game.obpm.unwrap_or_default();
        sum_dbpm += game.dbpm.unwrap_or_default();
        sum_bpm_net += game.bpm_net.unwrap_or_default();
        sum_bpm += game.bpm.unwrap_or_default();
        sum_sbpm += game.sbpm.unwrap_or_default();
        sum_orb_per_per_game += game.orb_per.unwrap_or_default();
        sum_drb_per_per_game += game.drb_per.unwrap_or_default();
        sum_ast_per_per_game += game.ast_per.unwrap_or_default();
        sum_to_per_per_game += game.to_per.unwrap_or_default();
        sum_stl_per_per_game += game.stl_per.unwrap_or_default();
        sum_blk_per_per_game += game.blk_per.unwrap_or_default();
    }

    // Calculate true percentages for the given slice based on summed raw totals
    // Effective Field Goal Percentage (eFG%): (FGM + 0.5 * 3PM) / FGA
    // FGM = total_two_pm + total_tpm
    // FGA = total_two_pa + total_tpa
    let avg_e_fg = if (total_two_pa + total_tpa) > 0.0 {
        (total_two_pm + total_tpm + 0.5 * total_tpm) / (total_two_pa + total_tpa) // Corrected formula
    } else { 0.0 };

    // True Shooting Percentage (TS%): PTS / (2 * (FGA + 0.44 * FTA))
    // FGA = total_two_pa + total_tpa
    let avg_ts_per = if (total_two_pa + total_tpa + 0.44 * total_fta) > 0.0 {
        total_pts / (2.0 * ((total_two_pa + total_tpa) + 0.44 * total_fta))
    } else { 0.0 };

    // Average per-game rates (these are already percentages/rates per game, so simple average is appropriate)
    let avg_orb_per = sum_orb_per_per_game / avg_games_played;
    let avg_drb_per = sum_drb_per_per_game / avg_games_played;
    let avg_ast_per = sum_ast_per_per_game / avg_games_played;
    let avg_to_per = sum_to_per_per_game / avg_games_played;
    let avg_stl_per = sum_stl_per_per_game / avg_games_played;
    let avg_blk_per = sum_blk_per_per_game / avg_games_played;

    // Construct and return the PlayerSeasonAverages struct for this slice
    Some(PlayerSeasonAverages {
        pid: player_pid,
        year: player_year,
        team: player_team.to_string(), // Clone to own the String for the struct field
        player_name: player_name.to_string(), // Clone to own the String for the struct field
        games_played,
        // Simple averages for per-game values (Category 1)
        avg_min_per: sum_min_per / avg_games_played,
        avg_o_rtg: sum_o_rtg / avg_games_played,
        avg_usg: sum_usg / avg_games_played,
        avg_bpm_rd: sum_bpm_rd / avg_games_played,
        avg_obpm: sum_obpm / avg_games_played,
        avg_dbpm: sum_dbpm / avg_games_played,
        avg_bpm_net: sum_bpm_net / avg_games_played,
        avg_bpm: sum_bpm / avg_games_played,
        avg_sbpm: sum_sbpm / avg_games_played,
        avg_pf: total_pf / avg_games_played,
        avg_possessions: total_possessions / avg_games_played,
        avg_inches: total_inches / avg_games_played,
        avg_opstyle: total_opstyle / avg_games_played,
        avg_quality: total_quality / avg_games_played,
        avg_win1: total_win1 / avg_games_played,
        avg_win2: total_win2 / avg_games_played,

        // Overall percentages/rates calculated from sums (Category 2)
        avg_e_fg,
        avg_ts_per,
        avg_orb_per,
        avg_drb_per,
        avg_ast_per,
        avg_to_per,
        avg_stl_per,
        avg_blk_per,

        // Averages of Counts (Category 1)
        avg_dunks_made: total_dunks_made / avg_games_played,
        avg_dunks_att: total_dunks_att / avg_games_played,
        avg_rim_made: total_rim_made / avg_games_played,
        avg_rim_att: total_rim_att / avg_games_played,
        avg_mid_made: total_mid_made / avg_games_played,
        avg_mid_att: total_mid_att / avg_games_played,
        avg_two_pm: total_two_pm / avg_games_played,
        avg_two_pa: total_two_pa / avg_games_played,
        avg_tpm: total_tpm / avg_games_played,
        avg_tpa: total_tpa / avg_games_played,
        avg_ftm: total_ftm / avg_games_played,
        avg_fta: total_fta / avg_games_played,
        avg_pts: total_pts / avg_games_played,
        avg_orb: total_orb / avg_games_played,
        avg_drb: total_drb / avg_games_played,
        avg_ast: total_ast / avg_games_played,
        avg_tov: total_tov / avg_games_played,
        avg_stl: total_stl / avg_games_played,
        avg_blk: total_blk / avg_games_played,
    })
}


/// Calculates and inserts player season average statistics into ScyllaDB.
/// This function groups game stats by player and year, computes averages,
/// and then persists them. It now leverages the `calculate_averages_for_games` helper.
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

    for ((pid, year, team), games_for_player_season) in player_season_games {
        let player_name = games_for_player_season.first().map_or("Unknown".to_string(), |g| g.pp.clone());

        // Use the new helper function to calculate averages for the entire season's games
        if let Some(averages) = calculate_averages_for_games(
            &games_for_player_season, // Pass the raw games for the season
            pid,
            year,
            &team, // Pass a reference to team
            &player_name, // Pass a reference to player_name
        ) {
            season_averages.push(averages);
        } else {
            // Use a clone for `team` and `player_name` in the info! macro as they are moved into the closure
            info!("Player PID: {}, Year: {}, Team: {} had no games with minutes played for season averages. Skipping.", pid, year, team.clone());
        }
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

/// Fetches all player season average statistics from ScyllaDB.
pub async fn get_all_player_season_averages_from_db(
    session: &Session,
) -> Result<Vec<PlayerSeasonAverages>, Box<dyn std::error::Error>> {
    info!("Fetching all player season averages from database...");
    let query_cql = r#"
        SELECT pid, year, team, player_name, games_played, avg_min_per, avg_o_rtg, avg_usg, avg_e_fg, avg_ts_per, avg_orb_per, avg_drb_per, avg_ast_per, avg_to_per, avg_dunks_made, avg_dunks_att, avg_rim_made, avg_rim_att, avg_mid_made, avg_mid_att, avg_two_pm, avg_two_pa, avg_tpm, avg_tpa, avg_ftm, avg_fta, avg_bpm_rd, avg_obpm, avg_dbpm, avg_bpm_net, avg_pts, avg_orb, avg_drb, avg_ast, avg_tov, avg_stl, avg_blk, avg_stl_per, avg_blk_per, avg_pf, avg_possessions, avg_bpm, avg_sbpm, avg_inches, avg_opstyle, avg_quality, avg_win1, avg_win2
        FROM stats.player_season_avg_stats
    "#;

    let mut all_averages = Vec::new();
    let page_size: i32 = 5000;

    let mut query = Query::new(query_cql);
    query.set_page_size(page_size);
    query.set_request_timeout(Some(Duration::from_secs(60)));

    let mut rows_iter = session.query_iter(query, ()).await?;

    let mut row_count = 0;
    while let Some(row_res) = rows_iter.next().await {
        match row_res {
            Ok(row) => {
                match PlayerSeasonAverages::from_row(row) {
                    Ok(avg) => {
                        all_averages.push(avg);
                        row_count += 1;
                    },
                    Err(e) => {
                        error!("Failed to parse player season average row (total processed: {}): {}", row_count, e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to retrieve row from query_iter (total processed: {}): {}", row_count, e);
                return Err(Box::new(e));
            }
        }
    }

    info!("Successfully fetched and parsed a total of {} player season average records.", all_averages.len());
    Ok(all_averages)
}

/// Calculates average statistics for a player over their last 'num_games' played.
///
/// This function:
/// 1. Filters `all_game_stats` for a specific player (`pid`, `year`, `team`).
/// 2. Sorts the player's games by `numdate` to determine the chronological order.
/// 3. Selects the most recent `num_games` from the sorted list.
/// 4. Calls `calculate_averages_for_games` to compute averages for this slice.
///
/// Parameters:
/// - `all_game_stats`: A slice containing all game statistics.
/// - `player_id`: The Player ID to filter by.
/// - `player_year`: The season year to filter by.
/// - `player_team`: The team name to filter by.
/// - `num_games`: The number of most recent games to consider for the average.
///
/// Returns:
/// - `Option<PlayerSeasonAverages>`: `Some` with the calculated averages for the slice,
///   `None` if no relevant games are found or not enough games for the slice after filtering for minutes.
pub fn calculate_last_x_games_averages(
    all_game_stats: &[GameStats],
    player_id: i32,
    player_year: i32,
    player_team: &str,
    num_games: usize,
) -> Option<PlayerSeasonAverages> {
    info!("Calculating last {} game averages for player PID: {}, Year: {}, Team: {}",
        num_games, player_id, player_year, player_team);

    // Filter games for the specific player, year, and team
    let mut player_games: Vec<&GameStats> = all_game_stats.iter()
        .filter(|game| {
            game.pid == Some(player_id) &&
                game.year == Some(player_year) &&
                game.tt == player_team
        })
        .collect();

    // Sort games by numdate in ascending order to get chronological order.
    // This is crucial for correctly identifying the "last X games".
    player_games.sort_by(|a, b| {
        a.numdate.cmp(&b.numdate)
    });

    // Select the most recent `num_games`.
    // If there are fewer games than `num_games`, it will take all available games.
    // Use `copied()` to get an iterator of `&GameStats` from `&&GameStats`.
    let slice_games: Vec<&GameStats> = if player_games.len() > num_games {
        player_games.iter().copied().skip(player_games.len() - num_games).collect()
    } else {
        player_games.iter().copied().collect() // Collect references if fewer games than num_games
    };

    if slice_games.is_empty() {
        info!("No games found for player PID: {}, Year: {}, Team: {} or not enough games for slice after initial filter.",
            player_id, player_year, player_team);
        return None;
    }

    let player_name = slice_games.first().map_or("Unknown".to_string(), |g| g.pp.clone());

    // Call the generic calculation function. This function will further filter for games with minutes.
    calculate_averages_for_games(
        &slice_games,
        player_id,
        player_year,
        player_team, // Pass reference directly
        &player_name, // Pass reference directly
    )
}

/// Calculates average statistics for a player within a specified date range.
///
/// This function:
/// 1. Filters `all_game_stats` for a specific player (`pid`, `year`, `team`).
/// 2. Further filters these games to be within `start_date_num` and `end_date_num` (inclusive).
/// 3. Calls `calculate_averages_for_games` to compute averages for this slice.
///
/// `numdate` is expected to be a string in a sortable format like "YYYYMMDD" (e.g., "20230101").
///
/// Parameters:
/// - `all_game_stats`: A slice containing all game statistics.
/// - `player_id`: The Player ID to filter by.
/// - `player_year`: The season year to filter by.
/// - `player_team`: The team name to filter by.
/// - `start_date_num`: The start date (inclusive) as a string (e.g., "20230101").
/// - `end_date_num`: The end date (inclusive) as a string (e.g., "20230131").
///
/// Returns:
/// - `Option<PlayerSeasonAverages>`: `Some` with the calculated averages for the date range,
///   `None` if no relevant games are found or no games with minutes played in the range.
pub fn calculate_player_averages_by_date_range(
    all_game_stats: &[GameStats],
    player_id: i32,
    player_year: i32,
    player_team: &str,
    start_date_num: &str,
    end_date_num: &str,
) -> Option<PlayerSeasonAverages> {
    info!(
        "Calculating averages for player PID: {}, Year: {}, Team: {} from {} to {}",
        player_id, player_year, player_team, start_date_num, end_date_num
    );

    // Filter games for the specific player, year, team, and within the date range.
    let filtered_games: Vec<&GameStats> = all_game_stats.iter()
        .filter(|game| {
            game.pid == Some(player_id) &&
                game.year == Some(player_year) &&
                game.tt == player_team &&
                // Filter by date range: ensures numdate exists and falls within the specified range
                // Use as_str() directly on the String to get &str for comparison
                game.numdate.as_str() >= start_date_num && game.numdate.as_str() <= end_date_num
        })
        .collect();

    if filtered_games.is_empty() {
        info!("No games found for player PID: {}, Year: {}, Team: {} in the specified date range after initial filter.",
            player_id, player_year, player_team);
        return None;
    }

    let player_name = filtered_games.first().map_or("Unknown".to_string(), |g| g.pp.clone());

    // Call the generic calculation function. This function will further filter for games with minutes.
    calculate_averages_for_games(
        &filtered_games,
        player_id,
        player_year,
        player_team, // Pass reference directly
        &player_name, // Pass reference directly
    )
}

/// Calculates percentile rank for a given value within a sorted list of values.
/// Returns a value between 0.0 and 100.0.
pub fn calculate_percentile(value: f64, sorted_data: &[f64]) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }

    let n = sorted_data.len() as f64;
    let mut count_less = 0.0;
    let mut count_equal = 0.0;

    for &data_point in sorted_data {
        if data_point < value {
            count_less += 1.0;
        } else if data_point == value {
            count_equal += 1.0;
        }
    }

    // Standard formula for percentile rank
    // P = (Number of values below X + 0.5 * Number of values equal to X) / Total number of values * 100
    ((count_less + 0.5 * count_equal) / n) * 100.0
}


/// Calculates and inserts player season percentile statistics into ScyllaDB.
pub async fn calculate_and_insert_season_percentiles(
    session: &Session,
    all_season_averages: &[PlayerSeasonAverages],
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Calculating player season percentiles...");

    if all_season_averages.is_empty() {
        info!("No player season averages found to calculate percentiles. Skipping.");
        return Ok(());
    }

    // Collect all values for each statistical category
    let mut min_per_values = Vec::new();
    let mut o_rtg_values = Vec::new();
    let mut usg_values = Vec::new();
    let mut e_fg_values = Vec::new();
    let mut ts_per_values = Vec::new();
    let mut orb_per_values = Vec::new();
    let mut drb_per_values = Vec::new();
    let mut ast_per_values = Vec::new();
    let mut to_per_values = Vec::new();
    let mut dunks_made_values = Vec::new();
    let mut dunks_att_values = Vec::new();
    let mut rim_made_values = Vec::new();
    let mut rim_att_values = Vec::new();
    let mut mid_made_values = Vec::new();
    let mut mid_att_values = Vec::new();
    let mut two_pm_values = Vec::new();
    let mut two_pa_values = Vec::new();
    let mut tpm_values = Vec::new();
    let mut tpa_values = Vec::new();
    let mut ftm_values = Vec::new();
    let mut fta_values = Vec::new();
    let mut bpm_rd_values = Vec::new();
    let mut obpm_values = Vec::new();
    let mut dbpm_values = Vec::new();
    let mut bpm_net_values = Vec::new();
    let mut pts_values = Vec::new();
    let mut orb_values = Vec::new();
    let mut drb_values = Vec::new();
    let mut ast_values = Vec::new();
    let mut tov_values = Vec::new();
    let mut stl_values = Vec::new();
    let mut blk_values = Vec::new();
    let mut stl_per_values = Vec::new();
    let mut blk_per_values = Vec::new();
    let mut pf_values = Vec::new();
    let mut possessions_values = Vec::new();
    let mut bpm_values = Vec::new();
    let mut sbpm_values = Vec::new();
    let mut inches_values = Vec::new();
    let mut opstyle_values = Vec::new();
    let mut quality_values = Vec::new();
    let mut win1_values = Vec::new();
    let mut win2_values = Vec::new();


    for avg in all_season_averages.iter() {
        min_per_values.push(avg.avg_min_per);
        o_rtg_values.push(avg.avg_o_rtg);
        usg_values.push(avg.avg_usg);
        e_fg_values.push(avg.avg_e_fg);
        ts_per_values.push(avg.avg_ts_per);
        orb_per_values.push(avg.avg_orb_per);
        drb_per_values.push(avg.avg_drb_per);
        ast_per_values.push(avg.avg_ast_per);
        to_per_values.push(avg.avg_to_per);
        dunks_made_values.push(avg.avg_dunks_made);
        dunks_att_values.push(avg.avg_dunks_att);
        rim_made_values.push(avg.avg_rim_made);
        rim_att_values.push(avg.avg_rim_att);
        mid_made_values.push(avg.avg_mid_made);
        mid_att_values.push(avg.avg_mid_att);
        two_pm_values.push(avg.avg_two_pm);
        two_pa_values.push(avg.avg_two_pa);
        tpm_values.push(avg.avg_tpm);
        tpa_values.push(avg.avg_tpa);
        ftm_values.push(avg.avg_ftm);
        fta_values.push(avg.avg_fta);
        bpm_rd_values.push(avg.avg_bpm_rd);
        obpm_values.push(avg.avg_obpm);
        dbpm_values.push(avg.avg_dbpm);
        bpm_net_values.push(avg.avg_bpm_net);
        pts_values.push(avg.avg_pts);
        orb_values.push(avg.avg_orb);
        drb_values.push(avg.avg_drb);
        ast_values.push(avg.avg_ast);
        tov_values.push(avg.avg_tov);
        stl_values.push(avg.avg_stl);
        blk_values.push(avg.avg_blk);
        stl_per_values.push(avg.avg_stl_per);
        blk_per_values.push(avg.avg_blk_per);
        pf_values.push(avg.avg_pf);
        possessions_values.push(avg.avg_possessions);
        bpm_values.push(avg.avg_bpm);
        sbpm_values.push(avg.avg_sbpm);
        inches_values.push(avg.avg_inches);
        opstyle_values.push(avg.avg_opstyle);
        quality_values.push(avg.avg_quality);
        win1_values.push(avg.avg_win1);
        win2_values.push(avg.avg_win2);
    }

    // Sort all collected values for percentile calculation
    min_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    o_rtg_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    usg_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    e_fg_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ts_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    orb_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    drb_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ast_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    to_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dunks_made_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dunks_att_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    rim_made_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    rim_att_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    mid_made_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    mid_att_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    two_pm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    two_pa_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    tpm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    tpa_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ftm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    fta_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    bpm_rd_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    obpm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dbpm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    bpm_net_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    pts_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    orb_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    drb_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ast_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    tov_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    stl_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    blk_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    stl_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    blk_per_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    pf_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    possessions_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    bpm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sbpm_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    inches_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    opstyle_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    quality_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    win1_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    win2_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));


    let mut season_percentiles: Vec<PlayerSeasonPercentiles> = Vec::new();

    for avg in all_season_averages.iter() {
        season_percentiles.push(PlayerSeasonPercentiles {
            pid: avg.pid,
            year: avg.year,
            team: avg.team.clone(),
            player_name: avg.player_name.clone(),
            pct_min_per: calculate_percentile(avg.avg_min_per, &min_per_values),
            pct_o_rtg: calculate_percentile(avg.avg_o_rtg, &o_rtg_values),
            pct_usg: calculate_percentile(avg.avg_usg, &usg_values),
            pct_e_fg: calculate_percentile(avg.avg_e_fg, &e_fg_values),
            pct_ts_per: calculate_percentile(avg.avg_ts_per, &ts_per_values),
            pct_orb_per: calculate_percentile(avg.avg_orb_per, &orb_per_values),
            pct_drb_per: calculate_percentile(avg.avg_drb_per, &drb_per_values),
            pct_ast_per: calculate_percentile(avg.avg_ast_per, &ast_per_values),
            pct_to_per: calculate_percentile(avg.avg_to_per, &to_per_values),
            pct_dunks_made: calculate_percentile(avg.avg_dunks_made, &dunks_made_values),
            pct_dunks_att: calculate_percentile(avg.avg_dunks_att, &dunks_att_values),
            pct_rim_made: calculate_percentile(avg.avg_rim_made, &rim_made_values),
            pct_rim_att: calculate_percentile(avg.avg_rim_att, &rim_att_values),
            pct_mid_made: calculate_percentile(avg.avg_mid_made, &mid_made_values),
            pct_mid_att: calculate_percentile(avg.avg_mid_att, &mid_att_values),
            pct_two_pm: calculate_percentile(avg.avg_two_pm, &two_pm_values),
            pct_two_pa: calculate_percentile(avg.avg_two_pa, &two_pa_values),
            pct_tpm: calculate_percentile(avg.avg_tpm, &tpm_values),
            pct_tpa: calculate_percentile(avg.avg_tpa, &tpa_values),
            pct_ftm: calculate_percentile(avg.avg_ftm, &ftm_values),
            pct_fta: calculate_percentile(avg.avg_fta, &fta_values),
            pct_bpm_rd: calculate_percentile(avg.avg_bpm_rd, &bpm_rd_values),
            pct_obpm: calculate_percentile(avg.avg_obpm, &obpm_values),
            pct_dbpm: calculate_percentile(avg.avg_dbpm, &dbpm_values),
            pct_bpm_net: calculate_percentile(avg.avg_bpm_net, &bpm_net_values),
            pct_pts: calculate_percentile(avg.avg_pts, &pts_values),
            pct_orb: calculate_percentile(avg.avg_orb, &orb_values),
            pct_drb: calculate_percentile(avg.avg_drb, &drb_values),
            pct_ast: calculate_percentile(avg.avg_ast, &ast_values),
            pct_tov: calculate_percentile(avg.avg_tov, &tov_values),
            pct_stl: calculate_percentile(avg.avg_stl, &stl_values),
            pct_blk: calculate_percentile(avg.avg_blk, &blk_values),
            pct_stl_per: calculate_percentile(avg.avg_stl_per, &stl_per_values),
            pct_blk_per: calculate_percentile(avg.avg_blk_per, &blk_per_values),
            pct_pf: calculate_percentile(avg.avg_pf, &pf_values),
            pct_possessions: calculate_percentile(avg.avg_possessions, &possessions_values),
            pct_bpm: calculate_percentile(avg.avg_bpm, &bpm_values),
            pct_sbpm: calculate_percentile(avg.avg_sbpm, &sbpm_values),
            pct_inches: calculate_percentile(avg.avg_inches, &inches_values),
            pct_opstyle: calculate_percentile(avg.avg_opstyle, &opstyle_values),
            pct_quality: calculate_percentile(avg.avg_quality, &quality_values),
            pct_win1: calculate_percentile(avg.avg_win1, &win1_values),
            pct_win2: calculate_percentile(avg.avg_win2, &win2_values),
        });
    }

    info!("Inserting {} player season percentile records into ScyllaDB", season_percentiles.len());
    let query = r#"
        INSERT INTO stats.player_season_percentiles (
            pid, year, team, player_name, pct_min_per, pct_o_rtg, pct_usg, pct_e_fg, pct_ts_per, pct_orb_per, pct_drb_per, pct_ast_per, pct_to_per, pct_dunks_made, pct_dunks_att, pct_rim_made, pct_rim_att, pct_mid_made, pct_mid_att, pct_two_pm, pct_two_pa, pct_tpm, pct_tpa, pct_ftm, pct_fta, pct_bpm_rd, pct_obpm, pct_dbpm, pct_bpm_net, pct_pts, pct_orb, pct_drb, pct_ast, pct_tov, pct_stl, pct_blk, pct_stl_per, pct_blk_per, pct_pf, pct_possessions, pct_bpm, pct_sbpm, pct_inches, pct_opstyle, pct_quality, pct_win1, pct_win2
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
    "#;

    let prepared = session.prepare(query).await?;

    for pct in season_percentiles {
        session.execute(&prepared, &pct).await?;
    }

    Ok(())
}

/// Fetches all player season percentile statistics from ScyllaDB.
pub async fn get_all_player_season_percentiles_from_db(
    session: &Session,
) -> Result<Vec<PlayerSeasonPercentiles>, Box<dyn std::error::Error>> {
    info!("Fetching all player season percentiles from database...");
    let query_cql = r#"
        SELECT pid, year, team, player_name, pct_min_per, pct_o_rtg, pct_usg, pct_e_fg, pct_ts_per, pct_orb_per, pct_drb_per, pct_ast_per, pct_to_per, pct_dunks_made, pct_dunks_att, pct_rim_made, pct_rim_att, pct_mid_made, pct_mid_att, pct_two_pm, pct_two_pa, pct_tpm, pct_tpa, pct_ftm, pct_fta, pct_bpm_rd, pct_obpm, pct_dbpm, pct_bpm_net, pct_pts, pct_orb, pct_drb, pct_ast, pct_tov, pct_stl, pct_blk, pct_stl_per, pct_blk_per, pct_pf, pct_possessions, pct_bpm, pct_sbpm, pct_inches, pct_opstyle, pct_quality, pct_win1, pct_win2
        FROM stats.player_season_percentiles
    "#;

    let mut all_percentiles = Vec::new();
    let page_size: i32 = 5000;

    let mut query = Query::new(query_cql);
    query.set_page_size(page_size);
    query.set_request_timeout(Some(Duration::from_secs(60)));

    let mut rows_iter = session.query_iter(query, ()).await?;

    let mut row_count = 0;
    while let Some(row_res) = rows_iter.next().await {
        match row_res {
            Ok(row) => {
                match PlayerSeasonPercentiles::from_row(row) {
                    Ok(pct) => {
                        all_percentiles.push(pct);
                        row_count += 1;
                    },
                    Err(e) => {
                        error!("Failed to parse player season percentile row (total processed: {}): {}", row_count, e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to retrieve row from query_iter (total processed: {}): {}", row_count, e);
                return Err(Box::new(e));
            }
        }
    }

    info!("Successfully fetched and parsed a total of {} player season percentile records.", all_percentiles.len());
    Ok(all_percentiles)
}
