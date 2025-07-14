import React from 'react';
import { Routes, Route, useParams, Navigate } from 'react-router-dom';
import PlayersTable from './PlayersTable';
import TeamStatsTable from './TeamStatsTable';
import PlayerGameStatsTable from './PlayerGameStatsTable';
import PlayerScatterPlot from './PlayerScatterPlot'; // NEW: Import the new component
import { Link } from 'react-router-dom';

const App = () => {
    return (
        <main style={{padding: '0', margin: '0', width: '100vw', height: '100vh'}} className="bg-gray-900 text-gray-100">
            <nav className="navbar is-info p-4 bg-blue-700 text-white shadow-lg">
                <div className="navbar-brand flex items-center justify-between w-full">
                    <Link to="/teams" className="navbar-item title is-4 text-white hover:text-blue-200 transition-colors duration-200">
                        🏀 CamPom Web
                    </Link>
                    <div className="navbar-menu">
                        <div className="navbar-end flex space-x-4">
                            <Link to="/teams" className="navbar-item text-white hover:text-blue-200 transition-colors duration-200 text-lg">
                                Teams
                            </Link>
                            <Link to="/player-chart/Duke/2025" className="navbar-item text-white hover:text-blue-200 transition-colors duration-200 text-lg">
                                Player Chart
                            </Link>
                        </div>
                    </div>
                </div>
            </nav>
            <Routes>
                <Route path="/" element={<Navigate to="/teams" replace/>}/>
                <Route path="/teams" element={<TeamStatsTable/>}/>
                <Route path="/team/:team/year/:year" element={<PlayerRoute/>}/>
                <Route path="/player/:team/:year/:pid" element={<PlayerGameStatsRoute/>}/>
                {/* NEW ROUTE: For the player scatter plot */}
                <Route path="/player-chart/:team/:year" element={<PlayerScatterPlot/>}/>
                <Route path="*" element={<h2 className="has-text-centered mt-6 text-red-500">404 - Page not found</h2>}/>
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
