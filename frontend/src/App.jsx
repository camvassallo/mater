import { useEffect, useState } from 'react';

function App() {
    const [message, setMessage] = useState("Loading...");

    useEffect(() => {
        fetch("/api/players?team=Duke")
            .then(res => res.text())
            .then(setMessage)
            .catch(() => setMessage("Failed to connect to Rust API"));
    }, []);

    return (
        <main style={{ padding: "2rem", fontFamily: "sans-serif" }}>
            <h1>{message}</h1>
        </main>
    );
}

export default App;