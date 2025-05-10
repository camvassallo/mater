import React, { useEffect, useMemo, useState } from 'react';
import { themeQuartz, colorSchemeDarkBlue } from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
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
        { headerName: 'Name', field: 'player_name', minWidth: 200},
        { field: 'team' },
        { field: 'conf' },
        { field: 'gp' },
        { field: 'min_per', valueFormatter: numberFormatter },
        { field: 'o_rtg', valueFormatter: numberFormatter },
        { field: 'usg', valueFormatter: numberFormatter },
        { field: 'e_fg', valueFormatter: numberFormatter },
        { field: 'ts_per', valueFormatter: numberFormatter },
        { field: 'orb_per', valueFormatter: numberFormatter },
        { field: 'drb_per', valueFormatter: numberFormatter },
        { field: 'ast_per', valueFormatter: numberFormatter },
        { field: 'to_per', valueFormatter: numberFormatter },
        { field: 'ftm' },
        { field: 'fta' },
        { field: 'ft_per', valueFormatter: numberFormatter },
        { field: 'two_pm' },
        { field: 'two_pa' },
        { field: 'two_p_per', valueFormatter: numberFormatter },
        { field: 'tpm' },
        { field: 'tpa' },
        { field: 'tp_per', valueFormatter: numberFormatter },
        { field: 'blk_per', valueFormatter: numberFormatter },
        { field: 'stl_per', valueFormatter: numberFormatter },
        { field: 'ftr', valueFormatter: numberFormatter },
        { field: 'yr' },
        { field: 'ht' },
        { field: 'num' },
        { field: 'porpag', valueFormatter: numberFormatter },
        { field: 'adjoe', valueFormatter: numberFormatter },
        { field: 'pfr', valueFormatter: numberFormatter },
        { field: 'year' },
        { field: 'pid' },
        { field: 'player_type' },
        { field: 'rec_rank', valueFormatter: numberFormatter },
        { field: 'ast_tov', valueFormatter: numberFormatter },
        { field: 'rim_made', valueFormatter: numberFormatter },
        { field: 'rim_attempted', valueFormatter: numberFormatter },
        { field: 'mid_made', valueFormatter: numberFormatter },
        { field: 'mid_attempted', valueFormatter: numberFormatter },
        { field: 'rim_pct', valueFormatter: numberFormatter },
        { field: 'mid_pct', valueFormatter: numberFormatter },
        { field: 'dunks_made', valueFormatter: numberFormatter },
        { field: 'dunks_attempted', valueFormatter: numberFormatter },
        { field: 'dunk_pct', valueFormatter: numberFormatter },
        { field: 'pick', valueFormatter: numberFormatter },
        { field: 'drtg', valueFormatter: numberFormatter },
        { field: 'adrtg', valueFormatter: numberFormatter },
        { field: 'dporpag', valueFormatter: numberFormatter },
        { field: 'stops', valueFormatter: numberFormatter },
        { field: 'bpm', valueFormatter: numberFormatter },
        { field: 'obpm', valueFormatter: numberFormatter },
        { field: 'dbpm', valueFormatter: numberFormatter },
        { field: 'gbpm', valueFormatter: numberFormatter },
        { field: 'mp', valueFormatter: numberFormatter },
        { field: 'ogbpm', valueFormatter: numberFormatter },
        { field: 'dgbpm', valueFormatter: numberFormatter },
        { field: 'oreb', valueFormatter: numberFormatter },
        { field: 'dreb', valueFormatter: numberFormatter },
        { field: 'treb', valueFormatter: numberFormatter },
        { field: 'ast', valueFormatter: numberFormatter },
        { field: 'stl', valueFormatter: numberFormatter },
        { field: 'blk', valueFormatter: numberFormatter },
        { field: 'pts', valueFormatter: numberFormatter },
    ], []);

    useEffect(() => {
        if (!team || !year) return;

        const url = `/api/players?team=${encodeURIComponent(team)}&year=${encodeURIComponent(year)}`;
        fetch(url)
            .then(res => res.json())
            .then(setRowData)
            .catch(console.error);
    }, [team, year]);

    console.log(rowData)

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
                                minWidth: 100, // was 80+
                                flex: 1,
                            }}
                            pagination={true}
                            paginationPageSize={20}
                            className="ag-theme-quartz ag-theme-compact" // ✅ this applies your CSS
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
