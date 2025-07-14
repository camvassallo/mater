import React, { useEffect, useState, useMemo } from 'react';
import {
    ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, ResponsiveContainer, Legend, Tooltip, Label
} from 'recharts';
import { useParams, useNavigate } from 'react-router-dom';

// Define the available stats for the dropdowns
// Map: { user_friendly_name: { field: 'api_field_name', label: 'Display Label' } }
const availableStats = {
    'avg_pts': { field: 'avg_pts', label: 'Points' },
    'avg_ast': { field: 'avg_ast', label: 'Assists' },
    'avg_drb': { field: 'avg_drb', label: 'Defensive Rebounds' },
    'avg_orb': { field: 'avg_orb', label: 'Offensive Rebounds' },
    'avg_e_fg': { field: 'avg_e_fg', label: 'Effective FG%' },
    'avg_ts_per': { field: 'avg_ts_per', label: 'True Shooting%' },
    'avg_usg': { field: 'avg_usg', label: 'Usage%' },
    'avg_o_rtg': { field: 'avg_o_rtg', label: 'Offensive Rating' },
    'avg_bpm_net': { field: 'avg_bpm_net', label: 'Net BPM' },
    'avg_bpm': { field: 'avg_bpm', label: 'BPM' },
    'avg_sbpm': { field: 'avg_sbpm', label: 'SBPM' },
    'avg_stl': { field: 'avg_stl', label: 'Steals' },
    'avg_blk': { field: 'avg_blk', label: 'Blocks' },
    'avg_pf': { field: 'avg_pf', label: 'Personal Fouls' },
    'games_played': { field: 'games_played', label: 'Games Played' },
    'avg_min_per': { field: 'avg_min_per', label: 'Minutes %' },
    'avg_ast_per': { field: 'avg_ast_per', label: 'Assist %' },
    'avg_tov': { field: 'avg_tov', label: 'Turnovers' },
    'avg_stl_per': { field: 'avg_stl_per', label: 'Steal %' },
    'avg_blk_per': { field: 'avg_blk_per', label: 'Block %' },
    'avg_possessions': { field: 'avg_possessions', label: 'Possessions' },
    'avg_inches': { field: 'avg_inches', label: 'Height (Inches)' },
    'avg_opstyle': { field: 'avg_opstyle', label: 'Offensive Style' },
    'avg_quality': { field: 'avg_quality', label: 'Quality' },
    'avg_win1': { field: 'avg_win1', label: 'Win1' }, // Assuming win rate or similar
    'avg_win2': { field: 'avg_win2', label: 'Win2' }, // Assuming win rate or similar
    'avg_dunks_made': { field: 'avg_dunks_made', label: 'Dunks Made' },
    'avg_dunks_att': { field: 'avg_dunks_att', label: 'Dunks Attempted' },
    'avg_rim_made': { field: 'avg_rim_made', label: 'Rim Shots Made' },
    'avg_rim_att': { field: 'Rim Shots Attempted', label: 'Rim Shots Attempted' },
    'avg_mid_made': { field: 'avg_mid_made', label: 'Mid-Range Made' },
    'avg_mid_att': { field: 'avg_mid_att', label: 'Mid-Range Attempted' },
    'avg_two_pm': { field: 'avg_two_pm', label: '2-Pointers Made' },
    'avg_two_pa': { field: 'avg_two_pa', label: '2-Pointers Attempted' },
    'avg_tpm': { field: 'avg_tpm', label: '3-Pointers Made' },
    'avg_tpa': { field: 'avg_tpa', label: '3-Pointers Attempted' },
    'avg_ftm': { field: 'avg_ftm', label: 'Free Throws Made' },
    'avg_fta': { field: 'avg_fta', label: 'Free Throws Attempted' },
};

// Custom Dot component for coloring based on team
const CustomDot = (props) => {
    const { cx, cy, payload, selectedTeam1, selectedTeam2 } = props;
    let fillColor = '#888888'; // Default grey color

    if (payload.team === selectedTeam1) {
        fillColor = '#007bff'; // Blue for Team 1
    } else if (payload.team === selectedTeam2) {
        fillColor = '#ffc107'; // Yellow/Amber for Team 2
    }

    return (
        <circle cx={cx} cy={cy} r={5} fill={fillColor} />
    );
};


// Custom Tooltip component to display player name and stats on hover
const CustomTooltip = ({ active, payload, availableStats, selectedXAxis, selectedYAxis, selectedTeam1, selectedTeam2 }) => { // Added selectedTeam1, selectedTeam2
    if (active && payload && payload.length) {
        // When using a single Scatter component with combined data,
        // payload[0].payload will directly contain the data object of the hovered point.
        const player = payload[0].payload;

        return (
            <div style={{
                backgroundColor: '#333', // Dark background for the tooltip
                border: '1px solid #777', // Grey border
                padding: '10px',
                color: '#fff', // White text
                borderRadius: '5px',
                fontSize: '0.9em',
                boxShadow: '0 2px 10px rgba(0,0,0,0.5)' // Subtle shadow
            }}>
                <p style={{ fontWeight: 'bold', marginBottom: '5px', color: player.team === selectedTeam1 ? '#007bff' : '#ffc107' }}> {/* Dynamic coloring */}
                    {player.player_name || 'Unknown Player'} ({player.team || 'N/A'})
                </p>
                {/* Display X-axis stat using the selectedXAxis prop */}
                <p>{`${availableStats[selectedXAxis].label}: ${player[selectedXAxis]?.toFixed(2) || 'N/A'}`}</p>
                {/* Display Y-axis stat using the selectedYAxis prop */}
                <p>{`${availableStats[selectedYAxis].label}: ${player[selectedYAxis]?.toFixed(2) || 'N/A'}`}</p>
            </div>
        );
    }
    return null;
};

// Custom Legend component to display team names and colors
const CustomChartLegend = ({ customPayload }) => { // Renamed prop to customPayload
    console.log("Legend Payload received by CustomChartLegend:", customPayload); // Debugging line
    if (!customPayload || customPayload.length === 0) {
        return <div style={{ color: '#aaa', textAlign: 'center', paddingTop: '20px' }}>No legend items to display.</div>;
    }
    return (
        <ul style={{
            display: 'flex',
            justifyContent: 'center',
            paddingTop: '20px',
            color: '#eee',
            listStyle: 'none',
            margin: 0,
            padding: 0
        }}>
            {customPayload.map((entry, index) => (
                <li key={`legend-item-${index}`} style={{
                    display: 'flex',
                    alignItems: 'center',
                    margin: '0 10px',
                }}>
                    <span style={{
                        display: 'inline-block',
                        width: '10px',
                        height: '10px',
                        borderRadius: '50%',
                        backgroundColor: entry.color,
                        marginRight: '5px',
                    }}></span>
                    {entry.value}
                </li>
            ))}
        </ul>
    );
};


const PlayerScatterPlot = () => {
    // Get year, team1, and optional team2 from URL parameters
    const { year: urlYear, team1: urlTeam1, team2: urlUrlTeam2 } = useParams();
    const navigate = useNavigate();

    const [playerData1, setPlayerData1] = useState([]);
    const [playerData2, setPlayerData2] = useState([]);
    const [selectedXAxis, setSelectedXAxis] = useState('avg_pts');
    const [selectedYAxis, setSelectedYAxis] = useState('avg_ast');
    const [selectedYear, setSelectedYear] = useState(urlYear || '2025');
    const [selectedTeam1, setSelectedTeam1] = useState(urlTeam1 || 'Duke');
    const [selectedTeam2, setSelectedTeam2] = useState(urlUrlTeam2 || '');

    const [availableTeams, setAvailableTeams] = useState([]);
    const [availableYears, setAvailableYears] = useState([]);
    const [isLoading, setIsLoading] = useState(true); // Added loading state

    // Combine player data from both teams into a single array for the ScatterChart
    const combinedPlayerData = useMemo(() => {
        return [...playerData1, ...playerData2];
    }, [playerData1, playerData2]);

    // Define custom legend payload
    const legendPayload = useMemo(() => {
        const payload = [];
        if (selectedTeam1) {
            payload.push({
                id: 'team1',
                value: `${selectedTeam1} Players`,
                type: 'circle',
                color: '#007bff',
            });
        }
        if (selectedTeam2) {
            payload.push({
                id: 'team2',
                value: `${selectedTeam2} Players`,
                type: 'circle',
                color: '#ffc107',
            });
        }
        return payload;
    }, [selectedTeam1, selectedTeam2]);


    useEffect(() => {
        setAvailableTeams(['Duke', 'Kentucky', 'Kansas', 'UCLA', 'Gonzaga', 'North Carolina', 'Villanova', 'Michigan', 'Purdue']);
        setAvailableYears(['2023', '2024', '2025']);

        if (!urlYear || !urlTeam1) {
            navigate(`/player-chart/${selectedYear}/${selectedTeam1}`, { replace: true });
        }
    }, [urlYear, urlTeam1, selectedYear, selectedTeam1, navigate]);

    useEffect(() => {
        let pathSegments = [selectedYear, selectedTeam1];
        if (selectedTeam2) {
            pathSegments.push(selectedTeam2);
        }
        const newPath = `/player-chart/${pathSegments.join('/')}`;

        let currentPathSegments = [urlYear, urlUrlTeam2];
        if (urlUrlTeam2) {
            currentPathSegments.push(urlUrlTeam2);
        }
        const currentUrlPath = `/player-chart/${currentPathSegments.filter(Boolean).join('/')}`;

        if (newPath !== currentUrlPath) {
            navigate(newPath, { replace: true });
        }
    }, [selectedYear, selectedTeam1, selectedTeam2, urlYear, urlTeam1, urlUrlTeam2, navigate]);


    // Refactored data fetching into a single useEffect to manage loading state
    useEffect(() => {
        const fetchAllPlayerData = async () => {
            setIsLoading(true); // Start loading
            setPlayerData1([]); // Clear previous data
            setPlayerData2([]); // Clear previous data

            const fetchPromises = [];

            if (selectedTeam1 && selectedYear) {
                fetchPromises.push(
                    fetch(`/api/player-season-averages?team=${selectedTeam1}&year=${selectedYear}`)
                        .then(res => {
                            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
                            return res.json();
                        })
                        .then(data => setPlayerData1(data))
                        .catch(error => {
                            console.error(`Error fetching data for ${selectedTeam1} in ${selectedYear}:`, error);
                            setPlayerData1([]); // Ensure data is cleared on error
                        })
                );
            }

            if (selectedTeam2 && selectedYear) {
                fetchPromises.push(
                    fetch(`/api/player-season-averages?team=${selectedTeam2}&year=${selectedYear}`)
                        .then(res => {
                            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
                            return res.json();
                        })
                        .then(data => setPlayerData2(data))
                        .catch(error => {
                            console.error(`Error fetching data for ${selectedTeam2} in ${selectedYear}:`, error);
                            setPlayerData2([]); // Ensure data is cleared on error
                        })
                );
            }

            // If no teams are selected, or if there are no fetch promises, stop loading immediately
            if (fetchPromises.length === 0) {
                setIsLoading(false);
                return;
            }

            // Wait for all fetches to complete
            await Promise.all(fetchPromises);
            setIsLoading(false); // End loading
        };

        fetchAllPlayerData();
    }, [selectedTeam1, selectedTeam2, selectedYear]); // Dependencies for this combined fetch


    // Basic inline styles for the select elements for immediate functionality
    const selectStyle = {
        width: '180px',
        padding: '8px',
        borderRadius: '4px',
        border: '1px solid #007bff', // A blue border
        backgroundColor: '#444', // Darker background
        color: 'white',
        fontSize: '1em',
        cursor: 'pointer',
        appearance: 'none', // Remove default system styling for dropdowns
        backgroundImage: 'url("data:image/svg+xml;charset=US-ASCII,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22292.4%22%20height%3D%22292.4%22%3E%3Cpath%20fill%3D%22%23ffffff%22%20d%3D%22M287%2C114.7L154.7%2C247c-2.5%2C2.5-5.6%2C3.7-8.7%2C3.7s-6.2-1.2-8.7-3.7L5.4%2C114.7C0.2%2C109.5-1.4%2C102.1%2C1.5%2C95.5c2.9-6.5%2C9.4-10.8%2C16.8-10.8h255.9c7.4%2C0%2C13.9%2C4.3%2C16.8%2C10.8C293.4%2C102.1%2C291.8%2C109.5%2C287%2C114.7z%22%2F%3E%3C%2Fsvg%3E")', // Custom arrow
        backgroundRepeat: 'no-repeat',
        backgroundPosition: 'right 8px center',
        backgroundSize: '12px',
    };

    const labelStyle = {
        marginBottom: '8px',
        fontSize: '1.1em',
        fontWeight: 'bold',
        color: '#ccc',
    };

    const containerStyle = {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '16px',
    };

    const mainSectionStyle = {
        padding: '16px',
        backgroundColor: '#222',
        color: '#eee',
        minHeight: '100vh',
    };

    const titleStyle = {
        fontSize: '2.5em',
        fontWeight: 'bold',
        textAlign: 'center',
        marginBottom: '32px',
        color: '#007bff',
    };

    const flexContainerStyle = {
        display: 'flex',
        flexWrap: 'wrap',
        justifyContent: 'center',
        gap: '16px',
        marginBottom: '32px',
    };

    return (
        <div style={mainSectionStyle}>
            <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
                <h1 style={titleStyle}>Player Performance Scatter Plot</h1>

                <div style={flexContainerStyle}>
                    {/* Year Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="year-select" style={labelStyle}>Select Year:</label>
                        <select
                            id="year-select"
                            value={selectedYear}
                            onChange={(e) => setSelectedYear(e.target.value)}
                            style={selectStyle}
                        >
                            {availableYears.map(year => (
                                <option key={year} value={year}>{year}</option>
                            ))}
                        </select>
                    </div>

                    {/* Team 1 Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="team1-select" style={labelStyle}>Team 1:</label>
                        <select
                            id="team1-select"
                            value={selectedTeam1}
                            onChange={(e) => setSelectedTeam1(e.target.value)}
                            style={selectStyle}
                        >
                            {availableTeams.map(team => (
                                <option key={team} value={team}>{team}</option>
                            ))}
                        </select>
                    </div>

                    {/* Team 2 Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="team2-select" style={labelStyle}>Team 2 (Optional):</label>
                        <select
                            id="team2-select"
                            value={selectedTeam2}
                            onChange={(e) => setSelectedTeam2(e.target.value)}
                            style={selectStyle}
                        >
                            <option value="">-- Select Team --</option> {/* Option to clear selection */}
                            {availableTeams.map(team => (
                                <option key={team} value={team}>{team}</option>
                            ))}
                        </select>
                    </div>

                    {/* X-Axis Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="x-axis-select" style={labelStyle}>X-Axis:</label>
                        <select
                            id="x-axis-select"
                            value={selectedXAxis}
                            onChange={(e) => setSelectedXAxis(e.target.value)}
                            style={selectStyle}
                        >
                            {Object.entries(availableStats).map(([key, value]) => (
                                <option key={key} value={key}>{value.label}</option>
                            ))}
                        </select>
                    </div>

                    {/* Y-Axis Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="y-axis-select" style={labelStyle}>Y-Axis:</label>
                        <select
                            id="y-axis-select"
                            value={selectedYAxis}
                            onChange={(e) => setSelectedYAxis(e.target.value)}
                            style={selectStyle}
                        >
                            {Object.entries(availableStats).map(([key, value]) => (
                                <option key={key} value={key}>{value.label}</option>
                            ))}
                        </select>
                    </div>
                </div>

                {isLoading ? (
                    <p style={{ textAlign: 'center', fontSize: '1.2em', color: '#007bff' }}>Loading player data...</p>
                ) : (
                    (combinedPlayerData.length > 0) ? (
                        <ResponsiveContainer width="100%" height={500}>
                            <ScatterChart
                                margin={{
                                    top: 20, right: 20, bottom: 20, left: 20,
                                }}
                            >
                                <CartesianGrid strokeDasharray="3 3" stroke="#555" />
                                <XAxis
                                    type="number"
                                    dataKey={selectedXAxis}
                                    name={availableStats[selectedXAxis].label}
                                    stroke="#eee"
                                    tickFormatter={(value) => value.toFixed(1)}
                                >
                                    <Label value={availableStats[selectedXAxis].label} offset={0} position="bottom" fill="#eee" />
                                </XAxis>
                                <YAxis
                                    type="number"
                                    dataKey={selectedYAxis}
                                    name={availableStats[selectedYAxis].label}
                                    stroke="#eee"
                                    tickFormatter={(value) => value.toFixed(1)}
                                >
                                    <Label value={availableStats[selectedYAxis].label} angle={-90} position="insideLeft" fill="#eee" />
                                </YAxis>
                                {/* Tooltip component */}
                                <Tooltip
                                    cursor={{ strokeDasharray: '3 3' }}
                                    content={<CustomTooltip availableStats={availableStats} selectedXAxis={selectedXAxis} selectedYAxis={selectedYAxis} selectedTeam1={selectedTeam1} selectedTeam2={selectedTeam2} />}
                                />
                                {/* Custom Legend with dynamic payload */}
                                <Legend
                                    wrapperStyle={{ paddingTop: '20px', color: '#eee' }}
                                    content={<CustomChartLegend customPayload={legendPayload} />}
                                />
                                {/* Single Scatter component with combined data and custom dot */}
                                {combinedPlayerData.length > 0 && (
                                    <Scatter
                                        data={combinedPlayerData}
                                        shape={<CustomDot selectedTeam1={selectedTeam1} selectedTeam2={selectedTeam2} />}
                                    />
                                )}
                            </ScatterChart>
                        </ResponsiveContainer>
                    ) : (
                        <p style={{ textAlign: 'center', fontSize: '1.2em', color: '#aaa' }}>
                            {selectedTeam1 && selectedYear ? `No player data available for ${selectedTeam1} in ${selectedYear}.` : 'Please select a team and year to view the scatter plot.'}
                            {selectedTeam2 && ` or ${selectedTeam2}`}
                        </p>
                    )
                )}
            </div>
        </div>
    );
};

export default PlayerScatterPlot;
