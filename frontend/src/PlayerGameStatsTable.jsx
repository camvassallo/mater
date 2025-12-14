import React, { useEffect, useMemo, useState } from 'react';
import { themeQuartz, colorSchemeDarkBlue } from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import {
    ModuleRegistry,
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
} from 'ag-grid-community';
import { useParams } from 'react-router-dom'; // Import useParams

ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
]);

// Helper for formatting numbers
const numberFormatter = (param) => {
    const val = param.value;
    return val == null ? '-' : Number(val).toFixed(2);
};

// PlayerGameStatsTable component now gets parameters directly from URL
const PlayerGameStatsTable = () => { // Removed props: ({ team, year, pid })
    const { team, year, pid } = useParams(); // Get team, year, pid from URL parameters

    const [rowData, setRowData] = useState([]);
    const [playerName, setPlayerName] = useState('');
    const [isLoading, setIsLoading] = useState(false); // Added loading state

    const columnDefs = useMemo(() => [
        // Game Info - Always visible
        {
            headerName: 'Game Info',
            children: [
                { headerName: 'Date', field: 'datetext', minWidth: 90, pinned: 'left' },
                { headerName: 'Opponent', field: 'opponent', minWidth: 140, pinned: 'left' },
                { headerName: 'Location', field: 'loc', minWidth: 80 },
                { headerName: 'Win', field: 'win1', minWidth: 65, hide: true },
                { headerName: 'Quality', field: 'quality', minWidth: 80, hide: true },
                { headerName: 'OP Style', field: 'opstyle', minWidth: 90, hide: true },
            ]
        },
        // Performance - Key stats
        {
            headerName: 'Performance',
            children: [
                { headerName: 'PTS', field: 'pts', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'AST', field: 'ast', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'STL', field: 'stl', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'BLK', field: 'blk', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'TOV', field: 'tov', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'Min%', field: 'min_per', minWidth: 75, valueFormatter: numberFormatter },
            ]
        },
        // Shooting
        {
            headerName: 'Shooting',
            children: [
                { headerName: 'eFG%', field: 'e_fg', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: 'TS%', field: 'ts_per', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: 'FT%', field: 'ft_per', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: '2P%', field: 'two_p_per', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: '3P%', field: 'tp_per', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: 'FTM', field: 'ftm', minWidth: 65, hide: true },
                { headerName: 'FTA', field: 'fta', minWidth: 65, hide: true },
                { headerName: '2PM', field: 'two_pm', minWidth: 65, hide: true },
                { headerName: '2PA', field: 'two_pa', minWidth: 65, hide: true },
                { headerName: '3PM', field: 'tpm', minWidth: 65, hide: true },
                { headerName: '3PA', field: 'tpa', minWidth: 65, hide: true },
                { headerName: 'FTR', field: 'ftr', minWidth: 70, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Rim%', field: 'rim_pct', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Mid%', field: 'mid_pct', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Dunk%', field: 'dunk_pct', minWidth: 80, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Rim Made', field: 'rim_made', minWidth: 90, hide: true },
                { headerName: 'Rim Att', field: 'rim_att', minWidth: 85, hide: true },
                { headerName: 'Mid Made', field: 'mid_made', minWidth: 90, hide: true },
                { headerName: 'Mid Att', field: 'mid_att', minWidth: 85, hide: true },
                { headerName: 'Dunks', field: 'dunks_made', minWidth: 80, hide: true },
                { headerName: 'Dunk Att', field: 'dunks_att', minWidth: 85, hide: true },
            ]
        },
        // Efficiency
        {
            headerName: 'Efficiency',
            children: [
                { headerName: 'ORtg', field: 'o_rtg', minWidth: 75, valueFormatter: numberFormatter },
                { headerName: 'Usage%', field: 'usage', minWidth: 85, valueFormatter: numberFormatter },
                { headerName: 'ORB%', field: 'orb_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'DRB%', field: 'drb_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'AST%', field: 'ast_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'TO%', field: 'to_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'STL%', field: 'stl_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'BLK%', field: 'blk_per', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'AST/TO', field: 'ast_tov', minWidth: 85, valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Rebounding Detail
        {
            headerName: 'Rebounding',
            children: [
                { headerName: 'ORB', field: 'orb', minWidth: 65, valueFormatter: numberFormatter },
                { headerName: 'DRB', field: 'dreb', minWidth: 65, valueFormatter: numberFormatter },
            ]
        },
        // Advanced
        {
            headerName: 'Advanced',
            children: [
                { headerName: 'BPM', field: 'bpm', minWidth: 70, valueFormatter: numberFormatter },
                { headerName: 'OBPM', field: 'obpm', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'DBPM', field: 'dbpm', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Net BPM', field: 'bpm_net', minWidth: 90, valueFormatter: numberFormatter, hide: true },
                { headerName: 'GBPM', field: 'gbpm', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'OGBPM', field: 'ogbpm', minWidth: 80, valueFormatter: numberFormatter, hide: true },
                { headerName: 'DGBPM', field: 'dgbpm', minWidth: 80, valueFormatter: numberFormatter, hide: true },
                { headerName: 'SBPM', field: 'sbpm', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'BPM RD', field: 'bpm_rd', minWidth: 80, valueFormatter: numberFormatter, hide: true },
                { headerName: 'DRtg', field: 'drtg', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'AdjDRtg', field: 'adrtg', minWidth: 90, valueFormatter: numberFormatter, hide: true },
                { headerName: 'DPORPAG', field: 'dporpag', minWidth: 95, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Stops', field: 'stops', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Poss', field: 'possessions', minWidth: 75, valueFormatter: numberFormatter, hide: true },
                { headerName: 'MP', field: 'mp', minWidth: 65, valueFormatter: numberFormatter, hide: true },
                { headerName: 'PFR', field: 'pfr', minWidth: 70, valueFormatter: numberFormatter, hide: true },
            ]
        },
        // Misc
        {
            headerName: 'Other',
            children: [
                { headerName: 'PF', field: 'pf', minWidth: 60, valueFormatter: numberFormatter, hide: true },
                { headerName: 'Win2', field: 'win2', minWidth: 70, hide: true },
                { headerName: 'MUID', field: 'muid', minWidth: 100, hide: true },
                { headerName: 'PID', field: 'pid', minWidth: 70, hide: true },
                { headerName: 'Year', field: 'year', minWidth: 70, hide: true },
            ]
        },
    ], []);

    useEffect(() => {
        // Now, team, year, and pid are directly from useParams
        if (!team || !year || !pid) {
            console.log("PlayerGameStatsTable: Missing team, year, or pid from URL. Not fetching data.");
            setRowData([]);
            setPlayerName('');
            return;
        }

        setIsLoading(true); // Set loading true when starting fetch
        setRowData([]); // Clear previous data
        setPlayerName(''); // Clear previous player name

        // Fetch game stats
        const url = `/api/game-stats?pid=${encodeURIComponent(pid)}&year=${encodeURIComponent(year)}&team=${encodeURIComponent(team)}`;
        console.log("PlayerGameStatsTable: Fetching game stats from URL:", url);

        fetch(url)
            .then(res => {
                console.log("PlayerGameStatsTable: API Response status:", res.status);
                if (!res.ok) {
                    throw new Error(`HTTP error! status: ${res.status}`);
                }
                return res.json();
            })
            .then(data => {
                console.log("PlayerGameStatsTable: Received data:", data);
                setRowData(data);
                // Try to get the player name from the first row, if available
                if (data.length > 0 && data[0].pp) {
                    setPlayerName(data[0].pp);
                } else {
                    // Fallback if no data or pp field is missing
                    setPlayerName(`Player ${pid}`);
                }
            })
            .catch(error => {
                console.error("PlayerGameStatsTable: Error fetching game stats:", error);
                setRowData([]);
                setPlayerName('Error Loading Player');
            })
            .finally(() => {
                setIsLoading(false); // Set loading false after fetch completes (success or error)
            });
    }, [team, year, pid]); // Dependencies are now directly from useParams

    return (
        <div className="section">
            <div className="container">
                <h1 className="title is-3 has-text-centered">
                    {playerName || 'Loading Player...'} Game Logs ({team || 'N/A'}, {year || 'N/A'})
                </h1>
                {isLoading ? (
                    <div style={{ textAlign: 'center', padding: '20px', color: '#eee' }}>Loading game logs...</div>
                ) : rowData.length === 0 ? (
                    <div style={{ textAlign: 'center', padding: '20px', color: '#aaa' }}>No game logs found for this player.</div>
                ) : (
                    <div style={{ height: 'calc(100vh - 180px)', width: '100%', overflow: 'hidden' }}> {/* Adjusted height for better fit */}
                        <div className="ag-theme-quartz ag-theme-compact" style={{ height: '100%', width: '100%' }}>
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
                                theme={themeQuartz.withPart(colorSchemeDarkBlue)}
                            />
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

export default PlayerGameStatsTable;
