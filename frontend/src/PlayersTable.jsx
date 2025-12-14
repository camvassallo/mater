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
} from 'ag-grid-community';

ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
    ValidationModule,
]);

const PlayersTable = ({ team, year }) => {
    const [rowData, setRowData] = useState([]);

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
                { field: 'yr', headerName: 'Class', minWidth: 80, hide: true },
                { field: 'ht', headerName: 'Height', minWidth: 80, hide: true },
                { field: 'num', headerName: '#', minWidth: 60, hide: true },
                { field: 'player_type', headerName: 'Type', minWidth: 100, hide: true },
            ]
        },
        // Scoring - Key offensive stats
        {
            headerName: 'Scoring',
            children: [
                { field: 'pts', headerName: 'PTS', valueFormatter: numberFormatter },
                { field: 'o_rtg', headerName: 'ORtg', valueFormatter: numberFormatter },
                { field: 'usg', headerName: 'Usage%', valueFormatter: numberFormatter },
                { field: 'min_per', headerName: 'Min%', valueFormatter: numberFormatter, hide: true },
                { field: 'porpag', headerName: 'PORPAG', valueFormatter: numberFormatter, hide: true },
                { field: 'adjoe', headerName: 'AdjOE', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Shooting - All shooting efficiency
        {
            headerName: 'Shooting',
            children: [
                { field: 'e_fg', headerName: 'eFG%', valueFormatter: numberFormatter },
                { field: 'ts_per', headerName: 'TS%', valueFormatter: numberFormatter },
                { field: 'ft_per', headerName: 'FT%', valueFormatter: numberFormatter },
                { field: 'two_p_per', headerName: '2P%', valueFormatter: numberFormatter },
                { field: 'tp_per', headerName: '3P%', valueFormatter: numberFormatter },
                { field: 'ftm', headerName: 'FTM', hide: true },
                { field: 'fta', headerName: 'FTA', hide: true },
                { field: 'two_pm', headerName: '2PM', hide: true },
                { field: 'two_pa', headerName: '2PA', hide: true },
                { field: 'tpm', headerName: '3PM', hide: true },
                { field: 'tpa', headerName: '3PA', hide: true },
                { field: 'ftr', headerName: 'FTR', valueFormatter: numberFormatter, hide: true },
                { field: 'rim_pct', headerName: 'Rim%', valueFormatter: numberFormatter, hide: true },
                { field: 'mid_pct', headerName: 'Mid%', valueFormatter: numberFormatter, hide: true },
                { field: 'dunk_pct', headerName: 'Dunk%', valueFormatter: numberFormatter, hide: true },
                { field: 'rim_made', headerName: 'Rim Made', valueFormatter: numberFormatter, hide: true },
                { field: 'rim_attempted', headerName: 'Rim Att', valueFormatter: numberFormatter, hide: true },
                { field: 'mid_made', headerName: 'Mid Made', valueFormatter: numberFormatter, hide: true },
                { field: 'mid_attempted', headerName: 'Mid Att', valueFormatter: numberFormatter, hide: true },
                { field: 'dunks_made', headerName: 'Dunks', valueFormatter: numberFormatter, hide: true },
                { field: 'dunks_attempted', headerName: 'Dunk Att', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Rebounding
        {
            headerName: 'Rebounding',
            children: [
                { field: 'treb', headerName: 'TRB', valueFormatter: numberFormatter },
                { field: 'oreb', headerName: 'ORB', valueFormatter: numberFormatter, hide: true },
                { field: 'dreb', headerName: 'DRB', valueFormatter: numberFormatter, hide: true },
                { field: 'orb_per', headerName: 'ORB%', valueFormatter: numberFormatter, hide: true },
                { field: 'drb_per', headerName: 'DRB%', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Playmaking
        {
            headerName: 'Playmaking',
            children: [
                { field: 'ast', headerName: 'AST', valueFormatter: numberFormatter },
                { field: 'ast_per', headerName: 'AST%', valueFormatter: numberFormatter },
                { field: 'to_per', headerName: 'TO%', valueFormatter: numberFormatter },
                { field: 'ast_tov', headerName: 'AST/TO', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Defense
        {
            headerName: 'Defense',
            children: [
                { field: 'stl', headerName: 'STL', valueFormatter: numberFormatter },
                { field: 'blk', headerName: 'BLK', valueFormatter: numberFormatter },
                { field: 'stl_per', headerName: 'STL%', valueFormatter: numberFormatter },
                { field: 'blk_per', headerName: 'BLK%', valueFormatter: numberFormatter },
                { field: 'drtg', headerName: 'DRtg', valueFormatter: numberFormatter, hide: true },
                { field: 'adrtg', headerName: 'AdjDRtg', valueFormatter: numberFormatter, hide: true },
                { field: 'dporpag', headerName: 'DPORPAG', valueFormatter: numberFormatter, hide: true },
                { field: 'stops', headerName: 'Stops', valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Advanced Stats
        {
            headerName: 'Advanced',
            children: [
                { field: 'bpm', headerName: 'BPM', valueFormatter: numberFormatter },
                { field: 'obpm', headerName: 'OBPM', valueFormatter: numberFormatter, hide: true },
                { field: 'dbpm', headerName: 'DBPM', valueFormatter: numberFormatter, hide: true },
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

        const url = `/api/players?team=${encodeURIComponent(team)}&year=${encodeURIComponent(year)}`;
        fetch(url)
            .then(res => res.json())
            .then(setRowData)
            .catch(console.error);
    }, [team, year]);

    console.log(rowData) //

    return (
        <div className="section">
            <div className="container">
                <h1 className="title is-3 has-text-centered">{team} Players ({year})</h1>
                <div style={{height: 'calc(100vh - 100px)', width: '100%'}}>
                    <div style={{height: 'calc(100vh - 100px)', width: '100%'}}>
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