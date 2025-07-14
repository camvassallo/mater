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
        { headerName: 'Date', field: 'datetext', minWidth: 100 },
        { headerName: 'Opponent', field: 'opponent', minWidth: 150 },
        { headerName: 'Min_per', field: 'min_per', valueFormatter: numberFormatter },
        { headerName: 'ORtg', field: 'o_rtg', valueFormatter: numberFormatter },
        { headerName: 'Usage', field: 'usage', valueFormatter: numberFormatter },
        { headerName: 'eFG', field: 'e_fg', valueFormatter: numberFormatter },
        { headerName: 'TS%', field: 'ts_per', valueFormatter: numberFormatter },
        { headerName: 'ORB%', field: 'orb_per', valueFormatter: numberFormatter },
        { headerName: 'DRB%', field: 'drb_per', valueFormatter: numberFormatter },
        { headerName: 'AST%', field: 'ast_per', valueFormatter: numberFormatter },
        { headerName: 'TO%', field: 'to_per', valueFormatter: numberFormatter },
        { headerName: 'FTM', field: 'ftm' },
        { headerName: 'FTA', field: 'fta' },
        { headerName: 'FT%', field: 'ft_per', valueFormatter: numberFormatter },
        { headerName: '2PM', field: 'two_pm' },
        { headerName: '2PA', field: 'two_pa' },
        { headerName: '2P%', field: 'two_p_per', valueFormatter: numberFormatter },
        { headerName: '3PM', field: 'tpm' },
        { headerName: '3PA', field: 'tpa' },
        { headerName: '3P%', field: 'tp_per', valueFormatter: numberFormatter },
        { headerName: 'Blk%', field: 'blk_per', valueFormatter: numberFormatter },
        { headerName: 'Stl%', field: 'stl_per', valueFormatter: numberFormatter },
        { headerName: 'FTR', field: 'ftr', valueFormatter: numberFormatter },
        { headerName: 'BPM RD', field: 'bpm_rd', valueFormatter: numberFormatter },
        { headerName: 'OBPM', field: 'obpm', valueFormatter: numberFormatter },
        { headerName: 'DBPM', field: 'dbpm', valueFormatter: numberFormatter },
        { headerName: 'Net BPM', field: 'bpm_net', valueFormatter: numberFormatter },
        { headerName: 'PTS', field: 'pts', valueFormatter: numberFormatter },
        { headerName: 'ORB', field: 'orb', valueFormatter: numberFormatter },
        { headerName: 'DRB', field: 'dreb', valueFormatter: numberFormatter },
        { headerName: 'AST', field: 'ast', valueFormatter: numberFormatter },
        { headerName: 'TOV', field: 'tov', valueFormatter: numberFormatter },
        { headerName: 'STL', field: 'stl', valueFormatter: numberFormatter },
        { headerName: 'BLK', field: 'blk', valueFormatter: numberFormatter },
        { headerName: 'PF', field: 'pf', valueFormatter: numberFormatter },
        { headerName: 'Poss', field: 'possessions', valueFormatter: numberFormatter },
        { headerName: 'BPM', field: 'bpm', valueFormatter: numberFormatter },
        { headerName: 'SBPM', field: 'sbpm', valueFormatter: numberFormatter },
        { headerName: 'Loc', field: 'loc' },
        { headerName: 'MUID', field: 'muid' }, // Maybe useful for debugging or linking to external sites
        { headerName: 'Win1', field: 'win1' },
        { headerName: 'Win2', field: 'win2' },
        { headerName: 'OP Style', field: 'opstyle' },
        { headerName: 'Quality', field: 'quality' },
        { headerName: 'Dunks Made', field: 'dunks_made' },
        { headerName: 'Dunks Att', field: 'dunks_att' },
        { headerName: 'Rim Made', field: 'rim_made' },
        { headerName: 'Rim Att', field: 'rim_att' },
        { headerName: 'Mid Made', field: 'mid_made' },
        { headerName: 'Mid Att', field: 'mid_att' },
        { headerName: 'PFR', field: 'pfr', valueFormatter: numberFormatter },
        { headerName: 'Year', field: 'year' },
        { headerName: 'PID', field: 'pid' },
        { headerName: 'Rec Rank', field: 'rec_rank', valueFormatter: numberFormatter },
        { headerName: 'Ast/TOV', field: 'ast_tov', valueFormatter: numberFormatter },
        { headerName: 'Rim Pct', field: 'rim_pct', valueFormatter: numberFormatter },
        { headerName: 'Mid Pct', field: 'mid_pct', valueFormatter: numberFormatter },
        { headerName: 'Dunk Pct', field: 'dunk_pct', valueFormatter: numberFormatter },
        { headerName: 'Pick', field: 'pick', valueFormatter: numberFormatter },
        { headerName: 'DRTG', field: 'drtg', valueFormatter: numberFormatter },
        { headerName: 'ADRTG', field: 'adrtg', valueFormatter: numberFormatter },
        { headerName: 'DPORPAG', field: 'dporpag', valueFormatter: numberFormatter },
        { headerName: 'Stops', field: 'stops', valueFormatter: numberFormatter },
        { headerName: 'GBPM', field: 'gbpm', valueFormatter: numberFormatter },
        { headerName: 'MP', field: 'mp', valueFormatter: numberFormatter },
        { headerName: 'OGBPM', field: 'ogbpm', valueFormatter: numberFormatter },
        { headerName: 'DGBPM', field: 'dgbpm', valueFormatter: numberFormatter },
        { headerName: 'numdate', field: 'numdate' }, // raw date number
        // 'pp' and 'tt' are player name and team code, not needed in every row of individual player games
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
                                    minWidth: 100,
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
