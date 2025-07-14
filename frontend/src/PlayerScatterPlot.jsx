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

// Custom Tooltip component to display player name and stats on hover
const CustomTooltip = ({ active, payload, availableStats, selectedXAxis, selectedYAxis }) => {
    if (active && payload && payload.length) {
        // Collect unique player data objects from the payload.
        // The 'payload' array from Recharts Tooltip can contain multiple entries
        // for a single hovered point if multiple Scatter series overlap,
        // or if both X and Y axis dataKeys are present for a single series.
        // We want to display information for each *unique* player.
        const uniquePlayers = [];
        const seenPlayerIdentifiers = new Set(); // Use a set to track unique players by a combined identifier

        payload.forEach(entry => {
            const playerData = entry.payload; // This contains the original player data object
            // Create a unique identifier for the player (e.g., name + team) to handle duplicates
            const playerIdentifier = `${playerData.player_name}-${playerData.team}`; // Use player_name as per API

            if (playerData && !seenPlayerIdentifiers.has(playerIdentifier)) {
                uniquePlayers.push(playerData);
                seenPlayerIdentifiers.add(playerIdentifier);
            }
        });

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
                {uniquePlayers.map((player, index) => (
                    <div key={`${player.player_name}-${player.team || 'no-team'}-${index}`} style={{ marginBottom: index < uniquePlayers.length - 1 ? '10px' : '0' }}>
                        <p style={{ fontWeight: 'bold', marginBottom: '5px', color: player.team === 'Duke' ? '#007bff' : '#ffc107' }}>
                            {player.player_name || 'Unknown Player'} ({player.team || 'N/A'})
                        </p>
                        {/* Display X-axis stat using the selectedXAxis prop */}
                        <p>{`${availableStats[selectedXAxis].label}: ${player[selectedXAxis]?.toFixed(2) || 'N/A'}`}</p>
                        {/* Display Y-axis stat using the selectedYAxis prop */}
                        <p>{`${availableStats[selectedYAxis].label}: ${player[selectedYAxis]?.toFixed(2) || 'N/A'}`}</p>
                            </div>
                            ))}
                    </div>
                );
                }
                return null;
                };


                const PlayerScatterPlot = () => {
                // Get year, team1, and optional team2 from URL parameters
                const { year: urlYear, team1: urlTeam1, team2: urlUrlTeam2 } = useParams(); // Renamed urlTeam2 to avoid conflict
                const navigate = useNavigate();

                const [playerData1, setPlayerData1] = useState([]);
                const [playerData2, setPlayerData2] = useState([]); // State for the second team's data
                const [selectedXAxis, setSelectedXAxis] = useState('avg_pts');
                const [selectedYAxis, setSelectedYAxis] = useState('avg_ast');
                // Initialize state from URL params, with fallbacks
                const [selectedYear, setSelectedYear] = useState(urlYear || '2025');
                const [selectedTeam1, setSelectedTeam1] = useState(urlTeam1 || 'Duke');
                const [selectedTeam2, setSelectedTeam2] = useState(urlUrlTeam2 || ''); // Default to empty string

                const [availableTeams, setAvailableTeams] = useState([]);
                const [availableYears, setAvailableYears] = useState([]);

                // Mock data for teams and years (replace with API calls if available)
                useEffect(() => {
                // In a real application, you might fetch these from an API as well.
                // For now, we'll keep them static.
                setAvailableTeams(['Duke', 'Kentucky', 'Kansas', 'UCLA', 'Gonzaga', 'North Carolina', 'Villanova', 'Michigan', 'Purdue']);
                setAvailableYears(['2023', '2024', '2025']);

                // Navigate to a default URL if initial URL params are missing
                // This ensures a consistent starting point for the chart
                if (!urlYear || !urlTeam1) {
                navigate(`/player-chart/${selectedYear}/${selectedTeam1}`, { replace: true });
            }
            }, [urlYear, urlTeam1, selectedYear, selectedTeam1, navigate]);

                // Effect to update URL when selections change
                useEffect(() => {
                // Construct the new path based on current component state
                let pathSegments = [selectedYear, selectedTeam1];
                if (selectedTeam2) {
                pathSegments.push(selectedTeam2);
            }
                const newPath = `/player-chart/${pathSegments.join('/')}`;

                // Construct the current URL path from useParams for comparison
                let currentPathSegments = [urlYear, urlUrlTeam2]; // Use urlUrlTeam2 for comparison
                if (urlUrlTeam2) {
                currentPathSegments.push(urlUrlTeam2);
            }
                const currentUrlPath = `/player-chart/${currentPathSegments.filter(Boolean).join('/')}`; // Filter out undefined/null

                // Only navigate if the desired path is different from the current URL path
                if (newPath !== currentUrlPath) {
                navigate(newPath, { replace: true });
            }
            }, [selectedYear, selectedTeam1, selectedTeam2, urlYear, urlTeam1, urlUrlTeam2, navigate]);


                // Fetch data for the first team
                useEffect(() => {
                if (!selectedTeam1 || !selectedYear) {
                setPlayerData1([]);
                return;
            }

                const fetchTeam1Data = async () => {
                try {
                const response = await fetch(`/api/player-season-averages?team=${selectedTeam1}&year=${selectedYear}`);
                if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
                const data = await response.json();
                setPlayerData1(data);
            } catch (error) {
                console.error(`Error fetching data for ${selectedTeam1} in ${selectedYear}:`, error);
                setPlayerData1([]);
            }
            };

                fetchTeam1Data();
            }, [selectedTeam1, selectedYear]);

                // Fetch data for the second team (if selected)
                useEffect(() => {
                if (!selectedTeam2 || !selectedYear) {
                setPlayerData2([]);
                return;
            }

                const fetchTeam2Data = async () => {
                try {
                const response = await fetch(`/api/player-season-averages?team=${selectedTeam2}&year=${selectedYear}`);
                if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
                const data = await response.json();
                setPlayerData2(data);
            } catch (error) {
                console.error(`Error fetching data for ${selectedTeam2} in ${selectedYear}:`, error);
                setPlayerData2([]);
            }
            };

                fetchTeam2Data();
            }, [selectedTeam2, selectedYear]);


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

                    {(playerData1.length > 0 || playerData2.length > 0) ? (
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
                                {/* Tooltip component now passes selectedXAxis and selectedYAxis */}
                                <Tooltip
                                    cursor={{ strokeDasharray: '3 3' }}
                                    content={<CustomTooltip availableStats={availableStats} selectedXAxis={selectedXAxis} selectedYAxis={selectedYAxis} />}
                                />
                                <Legend wrapperStyle={{ paddingTop: '20px', color: '#eee' }} />
                                {/* Scatter for Team 1 (Blue) */}
                                {playerData1.length > 0 && (
                                    <Scatter name={`${selectedTeam1} Players`} data={playerData1} fill="#007bff" />
                                )}
                                {/* Scatter for Team 2 (Yellow/Amber) */}
                                {playerData2.length > 0 && (
                                    <Scatter name={`${selectedTeam2} Players`} data={playerData2} fill="#ffc107" />
                                )}
                            </ScatterChart>
                        </ResponsiveContainer>
                    ) : (
                        <p style={{ textAlign: 'center', fontSize: '1.2em', color: '#aaa' }}>
                            {selectedTeam1 && selectedYear ? `No player data available for ${selectedTeam1} in ${selectedYear}.` : 'Please select a team and year to view the scatter plot.'}
                            {selectedTeam2 && ` or ${selectedTeam2}`}
                        </p>
                    )}
                </div>
            </div>
        );
    };

    export default PlayerScatterPlot;
