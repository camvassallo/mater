import React from 'react';
import { Routes, Route, useParams, Navigate } from 'react-router-dom';
import PlayersTable from './PlayersTable';

const App = () => {
    return (
        <main style={{ padding: '0', margin: '0', width: '100vw', height: '100vh' }}>
            <nav className="navbar is-info">
                <div className="navbar-brand">
                    <span className="navbar-item title is-4">🏀 Player Stats Dashboard</span>
                </div>
            </nav>
            <Routes>
                <Route path="/" element={<Navigate to="/team/Duke/year/2025" replace />} />
                <Route path="/team/:team/year/:year" element={<PlayerRoute />} />
                <Route path="*" element={<h2 className="has-text-centered mt-6">404 - Page not found</h2>} />
            </Routes>
        </main>
    );
};

// ✅ Pull team/year from URL and pass to the table
const PlayerRoute = () => {
    const { team, year } = useParams();
    const parsedYear = parseInt(year, 10) || 2025;

    return <PlayersTable team={team} year={parsedYear} />;
};

export default App;
