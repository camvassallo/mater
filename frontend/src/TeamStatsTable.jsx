import React, { useEffect, useState, useMemo } from 'react';
import { AgGridReact } from 'ag-grid-react';
import { themeQuartz, colorSchemeDarkBlue } from 'ag-grid-community';
import { Link } from 'react-router-dom';

import {
    ClientSideRowModelModule,
    ModuleRegistry,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
} from 'ag-grid-community';

ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    PaginationModule,
    TextFilterModule,
    NumberFilterModule,
]);

const TeamStatsTable = () => {
    const [rowData, setRowData] = useState([]);

    const columnDefs = useMemo(() => [
        { field: 'rank' },
        {
            field: 'team',
            cellRenderer: TeamLinkRenderer
        },
        { field: 'conf' },
        { field: 'record' },
        { field: 'barthag', valueFormatter: num },
        { field: 'adjoe', valueFormatter: num },
        { field: 'adjde', valueFormatter: num },
        { field: 'wab', valueFormatter: num },
        { field: 'conf_win_perc', valueFormatter: num },
        { field: 'adj_tempo', valueFormatter: num },
    ], []);

    useEffect(() => {
        fetch('/api/team-stats')
            .then(res => res.json())
            .then(setRowData)
            .catch(console.error);
    }, []);

    return (
        <div className="section">
            <div className="container">
                <h1 className="title is-3 has-text-centered">Team Stats Overview</h1>
                <div style={{ height: 'calc(100vh - 100px)', width: '100%' }}>
                    <AgGridReact
                        rowData={rowData}
                        columnDefs={columnDefs}
                        defaultColDef={{ sortable: true, filter: true, resizable: true }}
                        pagination={true}
                        paginationPageSize={20}
                        theme={themeQuartz.withPart(colorSchemeDarkBlue)}
                    />
                </div>
            </div>
        </div>
    );
};

const num = (param) => param.value == null ? '-' : Number(param.value).toFixed(2);

export default TeamStatsTable;

const TeamLinkRenderer = (props) => {
    const team = props.value;
    return <Link to={`/team/${encodeURIComponent(team)}/year/2025`}>{team}</Link>;
};