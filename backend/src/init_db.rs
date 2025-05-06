use log::info;
use scylla::{Session, SessionBuilder};

pub async fn init_db() -> Result<(),  scylla::transport::errors::NewSessionError> {

    // Connect to ScyllaDB
    let session = SessionBuilder::new()
        .known_node("127.0.0.1:9042") // or your Docker IP
        .build()
        .await?;

    // Create keyspace
    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS stats WITH replication = { 'class': 'SimpleStrategy', 'replication_factor': 1 };",
            &[],
        )
        .await?;

    // Create table
    session
        .query(
            "CREATE TABLE IF NOT EXISTS stats.player_stats (
                player_name text,
                team text,
                conf text,
                gp int,
                min_per double,
                o_rtg double,
                usg double,
                e_fg double,
                ts_per double,
                orb_per double,
                drb_per double,
                ast_per double,
                to_per double,
                ftm int,
                fta int,
                ft_per double,
                two_pm int,
                two_pa int,
                two_p_per double,
                tpm int,
                tpa int,
                tp_per double,
                blk_per double,
                stl_per double,
                ftr double,
                yr text,
                ht text,
                num text,
                porpag double,
                adjoe double,
                pfr double,
                year int,
                pid int,
                player_type text,
                rec_rank double,
                ast_tov double,
                rim_made double,
                rim_attempted double,
                mid_made double,
                mid_attempted double,
                rim_pct double,
                mid_pct double,
                dunks_made double,
                dunks_attempted double,
                dunk_pct double,
                pick double,
                drtg double,
                adrtg double,
                dporpag double,
                stops double,
                bpm double,
                obpm double,
                dbpm double,
                gbpm double,
                mp double,
                ogbpm double,
                dgbpm double,
                oreb double,
                dreb double,
                treb double,
                ast double,
                stl double,
                blk double,
                pts double,
                PRIMARY KEY ((team, year), player_name)
            );",
            &[],
        )
        .await?;

    info!("✅ Keyspace and table are ready.");
    Ok(())
}
