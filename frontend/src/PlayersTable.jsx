import React, { useEffect, useMemo, useState } from 'react';
import { themeQuartz, colorSchemeDarkBlue } from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import { Link } from 'react-router-dom'; // NEW: Import Link
import {
    ModuleRegistry,
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
    ValidationModule,
    CellStyleModule,
} from 'ag-grid-community';

ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
    ValidationModule,
    CellStyleModule,
]);

// Helper function to get color based on percentile
const getPercentileColor = (percentile, invert = false) => {
    if (percentile == null || isNaN(percentile)) {
        console.log('No percentile data for this cell');
        return 'transparent';
    }

    // Invert percentile for "bad" stats (turnovers, fouls, etc.)
    const pct = invert ? (100 - percentile) : percentile;

    // Red to green gradient - MORE VIBRANT
    if (pct >= 80) {
        return 'rgba(34, 139, 34, 0.6)'; // Forest green - increased opacity
    } else if (pct >= 60) {
        return 'rgba(144, 238, 144, 0.5)'; // Light green - increased opacity
    } else if (pct >= 40) {
        return 'rgba(255, 255, 102, 0.4)'; // Yellow - increased opacity
    } else if (pct >= 20) {
        return 'rgba(255, 165, 0, 0.5)'; // Orange - increased opacity
    } else {
        return 'rgba(220, 53, 69, 0.5)'; // Red - increased opacity
    }
};

const PlayersTable = ({ team, year }) => {
    const [rowData, setRowData] = useState([]);
    const [useLast30Days, setUseLast30Days] = useState(false);

    const columnDefs = useMemo(() => [
        // Basic Info - Always visible
        {
            headerName: 'Player Info',
            children: [
                {
                    headerName: 'Name',
                    field: 'player_name',
                    minWidth: 180,
                    pinned: 'left',
                    cellRenderer: (params) => {
                        if (params.data && params.data.pid) {
                            const team = encodeURIComponent(params.data.team);
                            const year = encodeURIComponent(params.data.year);
                            const pid = encodeURIComponent(params.data.pid);
                            const playerName = params.value;
                            return <Link to={`/player/${team}/${year}/${pid}`}>{playerName}</Link>;
                        }
                        return params.value;
                    }
                },
                { field: 'team', minWidth: 140 },
                { field: 'conf', headerName: 'Conf', minWidth: 80 },
                { field: 'gp', headerName: 'GP', minWidth: 60 },
                { field: 'yr', headerName: 'Yr', minWidth: 60 },
                { field: 'ht', headerName: 'Ht', minWidth: 70 },
                { field: 'num', headerName: '#', minWidth: 60, hide: true },
                { field: 'player_type', headerName: 'Role', minWidth: 80 },
            ]
        },
        // Scoring - Key offensive stats
        {
            headerName: 'Scoring',
            children: [
                {
                    field: 'pts',
                    headerName: 'PTS',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_pts) })
                },
                {
                    field: 'o_rtg',
                    headerName: 'ORtg',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_o_rtg) })
                },
                {
                    field: 'usg',
                    headerName: 'Usage%',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_usg) })
                },
                {
                    field: 'min_per',
                    headerName: 'Min%',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_min_per) })
                },
                { field: 'porpag', headerName: 'PRPG!', valueFormatter: numberFormatter },
                { field: 'adjoe', headerName: 'AdjOE', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Shooting - All shooting efficiency
        {
            headerName: 'Shooting',
            children: [
                {
                    field: 'e_fg',
                    headerName: 'eFG%',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_e_fg) })
                },
                {
                    field: 'ts_per',
                    headerName: 'TS%',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_ts_per) })
                },
                { field: 'ft_per', headerName: 'FT%', valueFormatter: numberFormatter },
                { field: 'two_p_per', headerName: '2P%', valueFormatter: numberFormatter },
                { field: 'tp_per', headerName: '3P%', valueFormatter: numberFormatter },
                { field: 'ftm', headerName: 'FT', valueFormatter: numberFormatter },
                { field: 'fta', headerName: 'FTA', hide: true },
                { field: 'two_pm', headerName: '2P', valueFormatter: numberFormatter },
                { field: 'two_pa', headerName: '2PA', hide: true },
                { field: 'tpm', headerName: '3P', valueFormatter: numberFormatter },
                { field: 'tpa', headerName: '3PA', hide: true },
                { field: 'ftr', headerName: 'FTR', valueFormatter: numberFormatter },
                { field: 'three_pr', headerName: '3PR', valueFormatter: numberFormatter },
                { field: 'three_p_per_100', headerName: '3P/100', valueFormatter: numberFormatter },
                { field: 'rim_pct', headerName: 'Rim%', valueFormatter: numberFormatter, hide: true },
                { field: 'mid_pct', headerName: 'Mid%', valueFormatter: numberFormatter, hide: true },
                { field: 'dunk_pct', headerName: 'Dunk%', valueFormatter: numberFormatter, hide: true },
                { field: 'rim_made', headerName: 'Close 2', valueFormatter: numberFormatter },
                { field: 'rim_attempted', headerName: 'Rim Att', valueFormatter: numberFormatter, hide: true },
                { field: 'mid_made', headerName: 'Far 2', valueFormatter: numberFormatter },
                { field: 'mid_attempted', headerName: 'Mid Att', valueFormatter: numberFormatter, hide: true },
                { field: 'dunks_made', headerName: 'Dunks', valueFormatter: numberFormatter },
                { field: 'dunks_attempted', headerName: 'Dunk Att', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Rebounding
        {
            headerName: 'Rebounding',
            children: [
                {
                    field: 'treb',
                    headerName: 'TRB',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor((params.data?.pct_orb + params.data?.pct_drb) / 2) })
                },
                {
                    field: 'oreb',
                    headerName: 'ORB',
                    valueFormatter: numberFormatter,
                    hide: true,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_orb) })
                },
                {
                    field: 'dreb',
                    headerName: 'DRB',
                    valueFormatter: numberFormatter,
                    hide: true,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_drb) })
                },
                {
                    field: 'orb_per',
                    headerName: 'OR',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_orb_per) })
                },
                {
                    field: 'drb_per',
                    headerName: 'DR',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_drb_per) })
                },
            ]
        },
        // Playmaking
        {
            headerName: 'Playmaking',
            children: [
                {
                    field: 'ast',
                    headerName: 'AST',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_ast) })
                },
                {
                    field: 'ast_per',
                    headerName: 'Ast',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_ast_per) })
                },
                {
                    field: 'to_per',
                    headerName: 'TO',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_to_per, true) }) // Inverted
                },
                { field: 'ast_tov', headerName: 'A/TO', valueFormatter: numberFormatter },
            ]
        },
        // Defense
        {
            headerName: 'Defense',
            children: [
                {
                    field: 'stl',
                    headerName: 'STL',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_stl) })
                },
                {
                    field: 'blk',
                    headerName: 'BLK',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_blk) })
                },
                {
                    field: 'stl_per',
                    headerName: 'Stl',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_stl_per) })
                },
                {
                    field: 'blk_per',
                    headerName: 'Blk',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_blk_per) })
                },
                { field: 'drtg', headerName: 'D-Rtg', valueFormatter: numberFormatter },
                { field: 'adrtg', headerName: 'AdjDRtg', valueFormatter: numberFormatter, hide: true },
                { field: 'dporpag', headerName: 'D-PRPG', valueFormatter: numberFormatter },
                { field: 'stops', headerName: 'Stops', valueFormatter: numberFormatter, hide: true },
                { field: 'fc_per_40', headerName: 'FC/40', valueFormatter: numberFormatter },
            ]
        },
        // Advanced Stats
        {
            headerName: 'Advanced',
            children: [
                {
                    field: 'bpm',
                    headerName: 'BPM',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_bpm) })
                },
                {
                    field: 'obpm',
                    headerName: 'OBPM',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_obpm) })
                },
                {
                    field: 'dbpm',
                    headerName: 'DBPM',
                    valueFormatter: numberFormatter,
                    cellStyle: params => ({ backgroundColor: getPercentileColor(params.data?.pct_dbpm) })
                },
                { field: 'gbpm', headerName: 'GBPM', valueFormatter: numberFormatter, hide: true },
                { field: 'ogbpm', headerName: 'OGBPM', valueFormatter: numberFormatter, hide: true },
                { field: 'dgbpm', headerName: 'DGBPM', valueFormatter: numberFormatter, hide: true },
                { field: 'mp', headerName: 'MP', valueFormatter: numberFormatter, hide: true },
                { field: 'pfr', headerName: 'PFR', valueFormatter: numberFormatter, hide: true },
                { field: 'rec_rank', headerName: 'Recruit Rank', valueFormatter: numberFormatter, hide: true },
                { field: 'pick', headerName: 'Draft Pick', valueFormatter: numberFormatter, hide: true },
            ]
        },
    ], []);

    useEffect(() => {
        if (!team || !year) return;

        let url;
        let usePercentiles = false;

        if (useLast30Days) {
            url = `/api/player-rolling-averages?team=${encodeURIComponent(team)}&year=${encodeURIComponent(year)}&last_n_days=30`;
        } else {
            url = `/api/player-stats-with-percentiles?team=${encodeURIComponent(team)}&year=${encodeURIComponent(year)}`;
            usePercentiles = true;
        }

        fetch(url)
            .then(res => res.json())
            .then(data => {
                // Map the field names to match the column definitions
                if (useLast30Days) {
                    // Rolling averages mapping (no percentiles)
                    const mappedData = data.map(player => ({
                        player_name: player.player_name,
                        team: player.team,
                        conf: player.conf || '',
                        gp: player.games_played,
                        yr: player.yr || '',
                        ht: player.ht || '',
                        num: '', // Not available
                        player_type: player.player_type || '',
                        pts: player.avg_pts,
                        o_rtg: player.avg_o_rtg,
                        usg: player.avg_usg,
                        min_per: player.avg_min_per,
                        porpag: player.porpag || null,
                        adjoe: player.adjoe || null,
                        e_fg: player.avg_e_fg,
                        ts_per: player.avg_ts_per,
                        ft_per: player.avg_fta > 0 ? (player.avg_ftm / player.avg_fta) * 100 : null,
                        two_p_per: player.avg_two_pa > 0 ? (player.avg_two_pm / player.avg_two_pa) * 100 : null,
                        tp_per: player.avg_tpa > 0 ? (player.avg_tpm / player.avg_tpa) * 100 : null,
                        ftm: player.avg_ftm,
                        fta: player.avg_fta,
                        two_pm: player.avg_two_pm,
                        two_pa: player.avg_two_pa,
                        tpm: player.avg_tpm,
                        tpa: player.avg_tpa,
                        ftr: (player.avg_two_pa + player.avg_tpa) > 0 ? player.avg_fta / (player.avg_two_pa + player.avg_tpa) : null,
                        three_pr: (player.avg_two_pa + player.avg_tpa) > 0 ? (player.avg_tpa / (player.avg_two_pa + player.avg_tpa)) * 100 : null,
                        three_p_per_100: player.avg_possessions > 0 ? (player.avg_tpm / player.avg_possessions) * 100 : null,
                        fc_per_40: player.avg_min_per > 0 ? (player.avg_pf / player.avg_min_per) * 40 : null,
                        rim_pct: player.avg_rim_att > 0 ? (player.avg_rim_made / player.avg_rim_att) * 100 : null,
                        mid_pct: player.avg_mid_att > 0 ? (player.avg_mid_made / player.avg_mid_att) * 100 : null,
                        dunk_pct: player.avg_dunks_att > 0 ? (player.avg_dunks_made / player.avg_dunks_att) * 100 : null,
                        rim_made: player.avg_rim_made,
                        rim_attempted: player.avg_rim_att,
                        mid_made: player.avg_mid_made,
                        mid_attempted: player.avg_mid_att,
                        dunks_made: player.avg_dunks_made,
                        dunks_attempted: player.avg_dunks_att,
                        treb: player.avg_orb + player.avg_drb,
                        oreb: player.avg_orb,
                        dreb: player.avg_drb,
                        orb_per: player.avg_orb_per,
                        drb_per: player.avg_drb_per,
                        ast: player.avg_ast,
                        ast_per: player.avg_ast_per,
                        to_per: player.avg_to_per,
                        ast_tov: player.avg_tov > 0 ? player.avg_ast / player.avg_tov : null,
                        stl: player.avg_stl,
                        blk: player.avg_blk,
                        stl_per: player.avg_stl_per,
                        blk_per: player.avg_blk_per,
                        drtg: player.drtg || null,
                        adrtg: null, // Not available in rolling averages
                        dporpag: player.dporpag || null,
                        stops: null, // Not available in rolling averages
                        bpm: player.avg_bpm,
                        obpm: player.avg_obpm,
                        dbpm: player.avg_dbpm,
                        gbpm: null, // Not available in rolling averages
                        ogbpm: null, // Not available in rolling averages
                        dgbpm: null, // Not available in rolling averages
                        mp: null, // Not available in rolling averages
                        pfr: null, // Not available in rolling averages
                        rec_rank: null, // Not available in rolling averages
                        pick: null, // Not available in rolling averages
                        pid: player.pid,
                        year: player.year,
                    }));
                    setRowData(mappedData);
                } else {
                    // Player stats with percentiles mapping
                    const mappedData = data.map(player => ({
                        player_name: player.player_name,
                        team: player.team,
                        conf: '', // Not available in season averages
                        gp: player.games_played,
                        yr: '', // Not available in season averages
                        ht: '', // Not available in season averages
                        num: '', // Not available in season averages
                        player_type: '', // Not available in season averages
                        pts: player.avg_pts,
                        o_rtg: player.avg_o_rtg,
                        usg: player.avg_usg,
                        min_per: player.avg_min_per,
                        porpag: null,
                        adjoe: null,
                        e_fg: player.avg_e_fg,
                        ts_per: player.avg_ts_per,
                        ft_per: player.avg_fta > 0 ? (player.avg_ftm / player.avg_fta) * 100 : null,
                        two_p_per: player.avg_two_pa > 0 ? (player.avg_two_pm / player.avg_two_pa) * 100 : null,
                        tp_per: player.avg_tpa > 0 ? (player.avg_tpm / player.avg_tpa) * 100 : null,
                        ftm: player.avg_ftm,
                        fta: player.avg_fta,
                        two_pm: player.avg_two_pm,
                        two_pa: player.avg_two_pa,
                        tpm: player.avg_tpm,
                        tpa: player.avg_tpa,
                        ftr: (player.avg_two_pa + player.avg_tpa) > 0 ? player.avg_fta / (player.avg_two_pa + player.avg_tpa) : null,
                        three_pr: (player.avg_two_pa + player.avg_tpa) > 0 ? (player.avg_tpa / (player.avg_two_pa + player.avg_tpa)) * 100 : null,
                        three_p_per_100: player.avg_possessions > 0 ? (player.avg_tpm / player.avg_possessions) * 100 : null,
                        fc_per_40: player.avg_min_per > 0 ? (player.avg_pf / player.avg_min_per) * 40 : null,
                        rim_pct: player.avg_rim_att > 0 ? (player.avg_rim_made / player.avg_rim_att) * 100 : null,
                        mid_pct: player.avg_mid_att > 0 ? (player.avg_mid_made / player.avg_mid_att) * 100 : null,
                        dunk_pct: player.avg_dunks_att > 0 ? (player.avg_dunks_made / player.avg_dunks_att) * 100 : null,
                        rim_made: player.avg_rim_made,
                        rim_attempted: player.avg_rim_att,
                        mid_made: player.avg_mid_made,
                        mid_attempted: player.avg_mid_att,
                        dunks_made: player.avg_dunks_made,
                        dunks_attempted: player.avg_dunks_att,
                        treb: player.avg_orb + player.avg_drb,
                        oreb: player.avg_orb,
                        dreb: player.avg_drb,
                        orb_per: player.avg_orb_per,
                        drb_per: player.avg_drb_per,
                        ast: player.avg_ast,
                        ast_per: player.avg_ast_per,
                        to_per: player.avg_to_per,
                        ast_tov: player.avg_tov > 0 ? player.avg_ast / player.avg_tov : null,
                        stl: player.avg_stl,
                        blk: player.avg_blk,
                        stl_per: player.avg_stl_per,
                        blk_per: player.avg_blk_per,
                        drtg: null,
                        adrtg: null,
                        dporpag: null,
                        stops: null,
                        bpm: player.avg_bpm,
                        obpm: player.avg_obpm,
                        dbpm: player.avg_dbpm,
                        gbpm: null,
                        ogbpm: null,
                        dgbpm: null,
                        mp: null,
                        pfr: null,
                        rec_rank: null,
                        pick: null,
                        pid: player.pid,
                        year: player.year,
                        // Add percentiles for color coding
                        pct_pts: player.pct_pts,
                        pct_o_rtg: player.pct_o_rtg,
                        pct_usg: player.pct_usg,
                        pct_min_per: player.pct_min_per,
                        pct_e_fg: player.pct_e_fg,
                        pct_ts_per: player.pct_ts_per,
                        pct_two_pm: player.pct_two_pm,
                        pct_two_pa: player.pct_two_pa,
                        pct_tpm: player.pct_tpm,
                        pct_tpa: player.pct_tpa,
                        pct_ftm: player.pct_ftm,
                        pct_fta: player.pct_fta,
                        pct_orb: player.pct_orb,
                        pct_drb: player.pct_drb,
                        pct_orb_per: player.pct_orb_per,
                        pct_drb_per: player.pct_drb_per,
                        pct_ast: player.pct_ast,
                        pct_ast_per: player.pct_ast_per,
                        pct_to_per: player.pct_to_per,
                        pct_tov: player.pct_tov,
                        pct_stl: player.pct_stl,
                        pct_blk: player.pct_blk,
                        pct_stl_per: player.pct_stl_per,
                        pct_blk_per: player.pct_blk_per,
                        pct_bpm: player.pct_bpm,
                        pct_obpm: player.pct_obpm,
                        pct_dbpm: player.pct_dbpm,
                        pct_rim_made: player.pct_rim_made,
                        pct_rim_att: player.pct_rim_att,
                        pct_mid_made: player.pct_mid_made,
                        pct_mid_att: player.pct_mid_att,
                        pct_dunks_made: player.pct_dunks_made,
                        pct_dunks_att: player.pct_dunks_att,
                    }));
                    setRowData(mappedData);
                }
            })
            .catch(console.error);
    }, [team, year, useLast30Days]);

    console.log('Row data sample:', rowData[0]);
    console.log('Has percentile data?', rowData[0]?.pct_pts !== undefined);

    return (
        <div className="section">
            <div className="container">
                <h1 className="title is-3 has-text-centered">{team} Players ({year})</h1>
                <div style={{ marginBottom: '16px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <label style={{ display: 'flex', alignItems: 'center', color: '#eee', fontSize: '1.1em', cursor: 'pointer' }}>
                        <input
                            type="checkbox"
                            checked={useLast30Days}
                            onChange={(e) => setUseLast30Days(e.target.checked)}
                            style={{ marginRight: '8px', width: '18px', height: '18px', cursor: 'pointer' }}
                        />
                        Last 30 Days
                    </label>
                </div>
                <div style={{height: 'calc(100vh - 150px)', width: '100%'}}>
                    <div style={{height: 'calc(100vh - 150px)', width: '100%'}}>
                        <AgGridReact
                            rowData={rowData}
                            columnDefs={columnDefs}
                            defaultColDef={{
                                sortable: true,
                                filter: true,
                                resizable: true,
                                minWidth: 150,
                                flex: 1,
                            }}
                            pagination={true}
                            paginationPageSize={20}
                            className="ag-theme-quartz ag-theme-compact"
                            theme={themeQuartz.withPart(colorSchemeDarkBlue)}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};

const numberFormatter = (param) => {
    const val = param.value;
    return val == null ? '-' : Number(val).toFixed(2);
};

export default PlayersTable;