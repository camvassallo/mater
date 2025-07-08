import React from 'react';
import { Routes, Route, useParams, Navigate } from 'react-router-dom';
import PlayersTable from './PlayersTable';
import TeamStatsTable from './TeamStatsTable';
import PlayerGameStatsTable from './PlayerGameStatsTable'; // NEW: Import the new component
import { Link } from 'react-router-dom';

const App = () => {
    return (
        <main style={{padding: '0', margin: '0', width: '100vw', height: '100vh'}}>
            <nav className="navbar is-info">
                <div className="navbar-brand">
                    <Link to="/teams" className="navbar-item title is-4">
                        🏀 CamPom Web
                    </Link>
                </div>
            </nav>
            <Routes>
                <Route path="/" element={<Navigate to="/teams" replace/>}/>
                <Route path="/teams" element={<TeamStatsTable/>}/>
                <Route path="/team/:team/year/:year" element={<PlayerRoute/>}/>
                {/* NEW ROUTE: For individual player game stats */}
                <Route path="/player/:team/:year/:pid" element={<PlayerGameStatsRoute/>}/>
                <Route path="*" element={<h2 className="has-text-centered mt-6">404 - Page not found</h2>}/>
            </Routes>
        </main>
    );
};

// Pull team/year from URL and pass to the table
const PlayerRoute = () => {
    const {team, year} = useParams();
    const parsedYear = parseInt(year, 10) || 2025; // Default to 2025 if year is invalid

    return <PlayersTable team={team} year={parsedYear}/>;
};

// NEW: Pull team, year, and pid from URL and pass to the game stats table
const PlayerGameStatsRoute = () => {
    const {team, year, pid} = useParams();
    const parsedYear = parseInt(year, 10) || 2025; // Default to 2025 if year is invalid
    const parsedPid = parseInt(pid, 10) || 0; // Default to 0 if pid is invalid

    return <PlayerGameStatsTable team={team} year={parsedYear} pid={parsedPid}/>;
};


export default App;