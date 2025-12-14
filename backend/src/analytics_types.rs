use serde::{Deserialize, Serialize};
use scylla::{FromRow, SerializeRow};

/// Represents a player's average statistics over an entire season.
/// All statistical fields are `f64` as they are averages.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SerializeRow)]
pub struct PlayerSeasonAverages {
    pub pid: i32,
    pub year: i32,
    pub team: String,
    pub player_name: String,
    pub games_played: i32,

    pub avg_min_per: f64,
    pub avg_o_rtg: f64,
    pub avg_usg: f64,
    pub avg_e_fg: f64,
    pub avg_ts_per: f64,
    pub avg_orb_per: f64,
    pub avg_drb_per: f64,
    pub avg_ast_per: f64,
    pub avg_to_per: f64,
    pub avg_dunks_made: f64,
    pub avg_dunks_att: f64,
    pub avg_rim_made: f64,
    pub avg_rim_att: f64,
    pub avg_mid_made: f64,
    pub avg_mid_att: f64,
    pub avg_two_pm: f64,
    pub avg_two_pa: f64, // FIX: Changed f66 to f64
    pub avg_tpm: f64,
    pub avg_tpa: f64,
    pub avg_ftm: f64,
    pub avg_fta: f64,
    pub avg_bpm_rd: f64,
    pub avg_obpm: f64,
    pub avg_dbpm: f64,
    pub avg_bpm_net: f64,
    pub avg_pts: f64,
    pub avg_orb: f64,
    pub avg_drb: f64,
    pub avg_ast: f64,
    pub avg_tov: f64,
    pub avg_stl: f64, // Corrected from f66 to f64
    pub avg_blk: f64,
    pub avg_stl_per: f64,
    pub avg_blk_per: f64,
    pub avg_pf: f64,
    pub avg_possessions: f64,
    pub avg_bpm: f64,
    pub avg_sbpm: f64,
    pub avg_inches: f64,
    pub avg_opstyle: f64,
    pub avg_quality: f64,
    pub avg_win1: f64,
    pub avg_win2: f64,
}

/// Player rolling averages with additional season-long constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRollingAverages {
    #[serde(flatten)]
    pub averages: PlayerSeasonAverages,

    // Season-long constants (not in DB, populated separately from player_stats)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_type: Option<String>,  // Role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yr: Option<String>,  // Class
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ht: Option<String>,  // Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub porpag: Option<f64>,  // PORPAGATU (Points Over Replacement Per Adjusted Game At That Usage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dporpag: Option<f64>,  // Defensive PORPAGATU
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drtg: Option<f64>,  // Defensive Rating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjoe: Option<f64>,  // Adjusted Offensive Efficiency
}

/// Represents a player's percentile ranks for their season average statistics.
/// Percentile values are from 0.0 to 100.0.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, SerializeRow)]
pub struct PlayerSeasonPercentiles {
    pub pid: i32,
    pub year: i32,
    pub team: String,
    pub player_name: String,

    pub pct_min_per: f64,
    pub pct_o_rtg: f64,
    pub pct_usg: f64,
    pub pct_e_fg: f64,
    pub pct_ts_per: f64,
    pub pct_orb_per: f64,
    pub pct_drb_per: f64,
    pub pct_ast_per: f64,
    pub pct_to_per: f64,
    pub pct_dunks_made: f64,
    pub pct_dunks_att: f64,
    pub pct_rim_made: f64,
    pub pct_rim_att: f64,
    pub pct_mid_made: f64,
    pub pct_mid_att: f64,
    pub pct_two_pm: f64,
    pub pct_two_pa: f64,
    pub pct_tpm: f64,
    pub pct_tpa: f64,
    pub pct_ftm: f64,
    pub pct_fta: f64,
    pub pct_bpm_rd: f64,
    pub pct_obpm: f64,
    pub pct_dbpm: f64,
    pub pct_bpm_net: f64,
    pub pct_pts: f64,
    pub pct_orb: f64,
    pub pct_drb: f64,
    pub pct_ast: f64,
    pub pct_tov: f64,
    pub pct_stl: f64,
    pub pct_blk: f64,
    pub pct_stl_per: f64,
    pub pct_blk_per: f64,
    pub pct_pf: f64,
    pub pct_possessions: f64,
    pub pct_bpm: f64,
    pub pct_sbpm: f64,
    pub pct_inches: f64,
    pub pct_opstyle: f64,
    pub pct_quality: f64,
    pub pct_win1: f64,
    pub pct_win2: f64,
}

/// Type alias for PlayerRollingPercentiles, as it will have the same structure as season percentiles.
pub type PlayerRollingPercentiles = PlayerSeasonPercentiles;

/// Combined structure that includes both averages and percentiles for a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatsWithPercentiles {
    // Basic info
    pub pid: i32,
    pub year: i32,
    pub team: String,
    pub player_name: String,
    pub games_played: i32,

    // Averages
    pub avg_min_per: f64,
    pub avg_o_rtg: f64,
    pub avg_usg: f64,
    pub avg_e_fg: f64,
    pub avg_ts_per: f64,
    pub avg_orb_per: f64,
    pub avg_drb_per: f64,
    pub avg_ast_per: f64,
    pub avg_to_per: f64,
    pub avg_dunks_made: f64,
    pub avg_dunks_att: f64,
    pub avg_rim_made: f64,
    pub avg_rim_att: f64,
    pub avg_mid_made: f64,
    pub avg_mid_att: f64,
    pub avg_two_pm: f64,
    pub avg_two_pa: f64,
    pub avg_tpm: f64,
    pub avg_tpa: f64,
    pub avg_ftm: f64,
    pub avg_fta: f64,
    pub avg_bpm_rd: f64,
    pub avg_obpm: f64,
    pub avg_dbpm: f64,
    pub avg_bpm_net: f64,
    pub avg_pts: f64,
    pub avg_orb: f64,
    pub avg_drb: f64,
    pub avg_ast: f64,
    pub avg_tov: f64,
    pub avg_stl: f64,
    pub avg_blk: f64,
    pub avg_stl_per: f64,
    pub avg_blk_per: f64,
    pub avg_pf: f64,
    pub avg_possessions: f64,
    pub avg_bpm: f64,
    pub avg_sbpm: f64,
    pub avg_inches: f64,
    pub avg_opstyle: f64,
    pub avg_quality: f64,
    pub avg_win1: f64,
    pub avg_win2: f64,

    // Percentiles
    pub pct_min_per: f64,
    pub pct_o_rtg: f64,
    pub pct_usg: f64,
    pub pct_e_fg: f64,
    pub pct_ts_per: f64,
    pub pct_orb_per: f64,
    pub pct_drb_per: f64,
    pub pct_ast_per: f64,
    pub pct_to_per: f64,
    pub pct_dunks_made: f64,
    pub pct_dunks_att: f64,
    pub pct_rim_made: f64,
    pub pct_rim_att: f64,
    pub pct_mid_made: f64,
    pub pct_mid_att: f64,
    pub pct_two_pm: f64,
    pub pct_two_pa: f64,
    pub pct_tpm: f64,
    pub pct_tpa: f64,
    pub pct_ftm: f64,
    pub pct_fta: f64,
    pub pct_bpm_rd: f64,
    pub pct_obpm: f64,
    pub pct_dbpm: f64,
    pub pct_bpm_net: f64,
    pub pct_pts: f64,
    pub pct_orb: f64,
    pub pct_drb: f64,
    pub pct_ast: f64,
    pub pct_tov: f64,
    pub pct_stl: f64,
    pub pct_blk: f64,
    pub pct_stl_per: f64,
    pub pct_blk_per: f64,
    pub pct_pf: f64,
    pub pct_possessions: f64,
    pub pct_bpm: f64,
    pub pct_sbpm: f64,
    pub pct_inches: f64,
    pub pct_opstyle: f64,
    pub pct_quality: f64,
    pub pct_win1: f64,
    pub pct_win2: f64,
}
