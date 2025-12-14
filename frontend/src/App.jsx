import React from 'react';
import { Routes, Route, useParams, Navigate } from 'react-router-dom';
import PlayersTable from './PlayersTable';
import TeamStatsTable from './TeamStatsTable';
import PlayerGameStatsTable from './PlayerGameStatsTable';
import PlayerScatterPlot from './PlayerScatterPlot';
import { Link } from 'react-router-dom';

const App = () => {
    const navBarStyle = {
        padding: '16px',
        backgroundColor: '#0056b3', // Darker blue
        color: 'white',
        boxShadow: '0 2px 4px rgba(0,0,0,0.2)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
    };

    const navBrandStyle = {
        fontSize: '1.5em',
        fontWeight: 'bold',
        color: 'white',
        textDecoration: 'none',
        marginRight: '20px',
    };

    const navLinkStyle = {
        color: 'white',
        textDecoration: 'none',
        fontSize: '1.1em',
        padding: '8px 12px',
        borderRadius: '4px',
        transition: 'background-color 0.2s ease-in-out',
    };

    const navLinkHoverStyle = {
        backgroundColor: '#004085', // Even darker blue on hover
    };

    return (
        <main style={{padding: '0', margin: '0', width: '100vw', minHeight: '100vh', backgroundColor: '#222'}}>
            <nav style={navBarStyle}>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                    <Link to="/teams" style={navBrandStyle}>
                        🏀 CamPom Web
                    </Link>
                    <div style={{ display: 'flex', gap: '15px' }}>
                        <Link
                            to="/teams"
                            style={navLinkStyle}
                            onMouseEnter={e => e.currentTarget.style.backgroundColor = navLinkHoverStyle.backgroundColor}
                            onMouseLeave={e => e.currentTarget.style.backgroundColor = navBarStyle.backgroundColor}
                        >
                            Teams
                        </Link>
                        {/* Updated link to go to a default chart view, allowing PlayerScatterPlot to handle initial URL params */}
                        <Link
                            to="/player-chart/2026/Duke" // Default to year and one team
                            style={navLinkStyle}
                            onMouseEnter={e => e.currentTarget.style.backgroundColor = navLinkHoverStyle.backgroundColor}
                            onMouseLeave={e => e.currentTarget.style.backgroundColor = navBarStyle.backgroundColor}
                        >
                            Player Chart
                        </Link>
                    </div>
                </div>
            </nav>
            <Routes>
                <Route path="/" element={<Navigate to="/teams" replace/>}/>
                <Route path="/teams" element={<TeamStatsTable/>}/>
                <Route path="/team/:team/year/:year" element={<PlayerRoute/>}/>
                <Route path="/player/:team/:year/:pid" element={<PlayerGameStatsTable/>}/>
                {/* UPDATED ROUTE: For the player scatter plot with optional second team */}
                <Route path="/player-chart/:year/:team1/:team2?" element={<PlayerScatterPlot/>}/>
                <Route path="*" element={<h2 style={{ textAlign: 'center', marginTop: '40px', color: '#dc3545' }}>404 - Page not found</h2>}/>
            </Routes>
        </main>
    );
};

const PlayerRoute = () => {
    const {team, year} = useParams();
    const parsedYear = parseInt(year, 10) || 2026;

    return <PlayersTable team={team} year={parsedYear}/>;
};

const PlayerGameStatsRoute = () => {
    const {team, year, pid} = useParams();
    const parsedYear = parseInt(year, 10) || 2026;
    const parsedPid = parseInt(pid, 10) || 0;

    return <PlayerGameStatsTable team={team} year={parsedYear} pid={parsedPid}/>;
};

export default App;
