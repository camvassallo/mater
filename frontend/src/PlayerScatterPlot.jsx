import React, { useEffect, useState, useMemo } from 'react';
import {
    ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, Label
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
    'avg_rim_att': { field: 'avg_rim_att', label: 'Rim Shots Attempted' },
    'avg_mid_made': { field: 'avg_mid_made', label: 'Mid-Range Made' },
    'avg_mid_att': { field: 'avg_mid_att', label: 'Mid-Range Attempted' },
    'avg_two_pm': { field: 'avg_two_pm', label: '2-Pointers Made' },
    'avg_two_pa': { field: 'avg_two_pa', label: '2-Pointers Attempted' },
    'avg_tpm': { field: 'avg_tpm', label: '3-Pointers Made' },
    'avg_tpa': { field: 'avg_tpa', label: '3-Pointers Attempted' },
    'avg_ftm': { field: 'avg_ftm', label: 'Free Throws Made' },
    'avg_fta': { field: 'avg_fta', label: 'Free Throws Attempted' },
};

const PlayerScatterPlot = () => {
    const { team: urlTeam, year: urlYear } = useParams();
    const navigate = useNavigate();

    const [playerData, setPlayerData] = useState([]);
    const [selectedXAxis, setSelectedXAxis] = useState('avg_pts');
    const [selectedYAxis, setSelectedYAxis] = useState('avg_ast');
    const [selectedTeam, setSelectedTeam] = useState(urlTeam || 'Duke');
    const [selectedYear, setSelectedYear] = useState(urlYear || '2025');
    const [availableTeams, setAvailableTeams] = useState([]);
    const [availableYears, setAvailableYears] = useState([]);

    useEffect(() => {
        // Mock data for teams and years for demonstration.
        // In a real app, you'd fetch these from your backend.
        setAvailableTeams(['Duke', 'Kentucky', 'Kansas', 'UCLA', 'Gonzaga']);
        setAvailableYears(['2023', '2024', '2025']);

        if (!urlTeam || !urlYear) {
            navigate(`/player-chart/${selectedTeam}/${selectedYear}`, { replace: true });
        }
    }, [urlTeam, urlYear, selectedTeam, selectedYear, navigate]);


    useEffect(() => {
        if (!selectedTeam || !selectedYear) return;

        const fetchPlayerData = async () => {
            try {
                const response = await fetch(`/api/player-season-averages?team=${encodeURIComponent(selectedTeam)}&year=${encodeURIComponent(selectedYear)}`);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                const data = await response.json();
                setPlayerData(data);
            } catch (error) {
                console.error("Error fetching player season averages:", error);
                setPlayerData([]);
            }
        };

        fetchPlayerData();
    }, [selectedTeam, selectedYear]);

    useEffect(() => {
        if (selectedTeam && selectedYear && (selectedTeam !== urlTeam || selectedYear !== urlYear)) {
            navigate(`/player-chart/${selectedTeam}/${selectedYear}`, { replace: true });
        }
    }, [selectedTeam, selectedYear, urlTeam, urlYear, navigate]);


    // Custom tooltip content
    const CustomTooltip = ({ active, payload, label }) => {
        if (active && payload && payload.length) {
            const player = payload[0].payload;
            const xAxisLabel = availableStats[selectedXAxis].label;
            const yAxisLabel = availableStats[selectedYAxis].label;

            return (
                <div style={{ padding: '8px', background: '#333', color: 'white', borderRadius: '4px', border: '1px solid #555' }}>
                    <p style={{ fontWeight: 'bold', fontSize: '1.1em' }}>{player.player_name}</p>
                    <p>{`${xAxisLabel}: ${player[selectedXAxis]?.toFixed(2) || 'N/A'}`}</p>
                    <p>{`${yAxisLabel}: ${player[selectedYAxis]?.toFixed(2) || 'N/A'}`}</p>
                    <p>{`Team: ${player.team}`}</p>
                    <p>{`Year: ${player.year}`}</p>
                </div>
            );
        }
        return null;
    };

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
        color: '#ccc', // Lighter gray for labels
    };

    const containerStyle = {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '16px', // Space between selectors
    };

    const mainSectionStyle = {
        padding: '16px',
        backgroundColor: '#222', // Dark background
        color: '#eee', // Light text
        minHeight: '100vh',
    };

    const titleStyle = {
        fontSize: '2.5em',
        fontWeight: 'bold',
        textAlign: 'center',
        marginBottom: '32px',
        color: '#007bff', // Blue for title
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
                    {/* Team Selector */}
                    <div style={containerStyle}>
                        <label htmlFor="team-select" style={labelStyle}>Select Team:</label>
                        <select
                            id="team-select"
                            value={selectedTeam}
                            onChange={(e) => setSelectedTeam(e.target.value)}
                            style={selectStyle}
                        >
                            {availableTeams.map(team => (
                                <option key={team} value={team}>{team}</option>
                            ))}
                        </select>
                    </div>

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

                {playerData.length > 0 ? (
                    <ResponsiveContainer width="100%" height={500}>
                        <ScatterChart
                            margin={{
                                top: 20, right: 20, bottom: 20, left: 20,
                            }}
                        >
                            <CartesianGrid strokeDasharray="3 3" stroke="#555" /> {/* Darker grid lines */}
                            <XAxis
                                type="number"
                                dataKey={selectedXAxis}
                                name={availableStats[selectedXAxis].label}
                                stroke="#eee" // Lighter axis labels
                                tickFormatter={(value) => value.toFixed(1)}
                            >
                                <Label value={availableStats[selectedXAxis].label} offset={0} position="bottom" fill="#eee" />
                            </XAxis>
                            <YAxis
                                type="number"
                                dataKey={selectedYAxis}
                                name={availableStats[selectedYAxis].label}
                                stroke="#eee" // Lighter axis labels
                                tickFormatter={(value) => value.toFixed(1)}
                            >
                                <Label value={availableStats[selectedYAxis].label} angle={-90} position="insideLeft" fill="#eee" />
                            </YAxis>
                            <Tooltip cursor={{ strokeDasharray: '3 3' }} content={<CustomTooltip />} />
                            <Legend wrapperStyle={{ paddingTop: '20px', color: '#eee' }} /> {/* Lighter legend text */}
                            <Scatter name={`${selectedTeam} Players`} data={playerData} fill="#007bff" /> {/* Blue scatter points */}
                        </ScatterChart>
                    </ResponsiveContainer>
                ) : (
                    <p style={{ textAlign: 'center', fontSize: '1.2em', color: '#aaa' }}>
                        {selectedTeam && selectedYear ? `No player data available for ${selectedTeam} in ${selectedYear}.` : 'Please select a team and year to view the scatter plot.'}
                    </p>
                )}
            </div>
        </div>
    );
};

export default PlayerScatterPlot;
