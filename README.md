# Mater - College Basketball Analytics Platform

A full-stack web application for analyzing college basketball statistics with advanced analytics, percentile rankings, and interactive visualizations. Built with Rust and React.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Running Locally](#running-locally)
- [Building for Production](#building-for-production)
- [Deployment](#deployment)
- [API Documentation](#api-documentation)
- [Project Structure](#project-structure)
- [Database Schema](#database-schema)
- [Contributing](#contributing)

## Overview

Mater is a basketball analytics platform that ingests data from Barttorvik.com and provides advanced statistical analysis for college basketball teams and players. The platform features:

- Real-time player and team statistics
- Rolling averages (last 30 days)
- Percentile rankings across all statistical categories
- Interactive scatter plots for multi-team player comparison
- Mobile-responsive data tables with category-based navigation
- Game-by-game performance tracking

## Architecture

### High-Level Architecture

```
┌─────────────────┐
│   React SPA     │  ← Frontend (Vite + React)
│   (Port 5173)   │
└────────┬────────┘
         │ HTTP/JSON
         ↓
┌─────────────────┐
│  Actix-Web API  │  ← Backend (Rust)
│   (Port 8080)   │
└────────┬────────┘
         │ CQL
         ↓
┌─────────────────┐
│  ScyllaDB       │  ← Database (Cassandra-compatible)
│   (Port 9042)   │
└─────────────────┘
```

### Data Flow

1. **Data Ingestion**: Backend fetches data from Barttorvik.com endpoints (JSON/CSV/gzipped)
2. **Processing**: Rust backend processes, transforms, and calculates percentiles
3. **Storage**: Data stored in ScyllaDB with optimized schema for time-series queries
4. **API**: RESTful endpoints serve aggregated statistics
5. **Frontend**: React app consumes API and renders interactive visualizations

### Backend Architecture (Rust)

The backend is modular and organized by function:

- **`main.rs`**: API server setup, endpoint routing, percentile calculations
- **`analytics_calculator.rs`**: Advanced metrics (BPM, ORtg, percentiles)
- **`analytics_types.rs`**: Data structures and DTOs
- **`db_utils.rs`**: Database connection and utilities
- **`init_db.rs`**: Schema creation and data seeding
- **`get_player_stats.rs`**: Player statistics queries
- **`get_team_stats.rs`**: Team statistics queries
- **`get_game_stats.rs`**: Game-level statistics queries

### Frontend Architecture (React)

Component-based architecture with client-side routing:

- **`App.jsx`**: Main application component with routing
- **`PlayersTable.jsx`**: Mobile-responsive player stats table with tabs
- **`PlayerScatterPlot.jsx`**: Interactive scatter plot for player comparison
- **`PlayerGameStatsTable.jsx`**: Game-by-game player performance
- **`TeamStatsTable.jsx`**: Team aggregate statistics

## Features

### Player Analytics
- **Comprehensive Statistics**: 62 statistical categories per player
- **Rolling Averages**: Last 30 days performance tracking
- **Percentile Rankings**: Color-coded percentile gradients (0-100) for all stats
- **Game Logs**: Complete game-by-game performance history
- **Player Profiles**: Individual player pages with detailed analytics

### Team Analytics
- **Team Rosters**: Complete roster with player roles and demographics
- **Advanced Metrics**: Offensive/Defensive ratings, tempo, efficiency
- **Game Results**: Win/loss records with contextual statistics

### Visualizations
- **Interactive Scatter Plots**:
  - Compare players across multiple teams
  - Customizable X/Y axes (any statistical category)
  - Color-coded by team
  - Hover tooltips with player details
- **Percentile Heatmaps**: Gradient color coding (red → yellow → green)
- **Mobile-Optimized Tables**: Swipeable category tabs for mobile devices

### Mobile Experience
- **Responsive Design**: Optimized for mobile, tablet, and desktop
- **Tab-Based Navigation**: 8 statistical categories (Info, Scoring, Shooting, Rebounding, Playmaking, Defense, Advanced, All)
- **Pinned Columns**: Player name always visible while scrolling
- **Touch-Friendly**: Large touch targets, horizontal scrolling
- **Ultra-Compact Layout**: 7.5px fonts, 22px rows for maximum data density

## Tech Stack

### Backend
- **Runtime**: Rust 2024 Edition
- **Web Framework**: Actix-Web 4.x
- **Database**: ScyllaDB (Cassandra-compatible)
- **Database Driver**: scylla 0.13
- **Async Runtime**: Tokio 1.x
- **Data Processing**:
  - serde (JSON serialization)
  - csv (CSV parsing)
  - chrono (date/time handling)
  - flate2 (gzip decompression)

### Frontend
- **Framework**: React 19.1
- **Build Tool**: Vite 6.x
- **Routing**: React Router v7
- **Data Grid**: AG Grid Community 33.2
- **Charts**: Recharts 3.1
- **CSS Framework**: Bulma 1.0
- **Linting**: ESLint 9.x

### Infrastructure
- **Reverse Proxy**: Nginx
- **Deployment**: systemd services
- **Development**: Hot Module Replacement (HMR) via Vite

## Prerequisites

- **Rust**: 1.70+ ([Install Rust](https://rustup.rs/))
- **Node.js**: 18+ ([Install Node](https://nodejs.org/))
- **ScyllaDB**: 5.0+ ([Install ScyllaDB](https://www.scylladb.com/download/))
- **Git**: For cloning the repository

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/camvassallo/mater.git
cd mater
```

### 2. Setup Database

Start ScyllaDB (if not already running):

```bash
# macOS (via Docker)
docker run --name scylla -p 9042:9042 -d scylladb/scylla

# Linux (via systemd)
sudo systemctl start scylla-server
```

Initialize the database schema:

```bash
cd backend
cargo run --bin init_db
```

This creates the `basketball` keyspace and required tables.

### 3. Backend Setup

Install dependencies and build:

```bash
cd backend
cargo build --release
```

### 4. Frontend Setup

Install Node dependencies:

```bash
cd frontend
npm install
```

## Running Locally

### Development Mode

**Terminal 1 - Backend**:
```bash
cd backend
cargo run
# Server starts on http://localhost:8080
```

**Terminal 2 - Frontend**:
```bash
cd frontend
npm run dev
# Dev server starts on http://localhost:5173
```

**Terminal 3 - Database** (if using Docker):
```bash
docker start scylla
```

### Access the Application

Navigate to `http://localhost:5173` in your browser.

The frontend development server proxies API requests to `http://localhost:8080`.

### Development Workflow

1. Backend changes: Rust auto-recompiles on save (with `cargo watch`)
2. Frontend changes: Vite provides instant HMR
3. Database changes: Re-run `init_db` to reset schema

## Building for Production

### Backend Build

```bash
cd backend
cargo build --release
# Binary output: target/release/mater
```

### Frontend Build

```bash
cd frontend
npm run build
# Static files output: dist/
```

### Build Both

```bash
# From project root
cd backend && cargo build --release && cd ../frontend && npm run build
```

## Deployment

### Deployment Script

The project includes automated deployment scripts:

```bash
cd deploy
./package_and_deploy.sh
```

This script:
1. Builds the Rust backend binary
2. Builds the React frontend static assets
3. Copies files to the server
4. Restarts systemd services

### Manual Deployment

#### 1. Deploy Backend

```bash
# Copy backend binary to server
scp backend/target/release/mater user@server:/opt/mater/

# Setup systemd service
sudo cp backend/backend.service /etc/systemd/system/
sudo systemctl enable backend
sudo systemctl start backend
```

#### 2. Deploy Frontend

```bash
# Copy frontend build to nginx root
scp -r frontend/dist/* user@server:/var/www/mater/

# Configure nginx
sudo cp frontend/nginx.conf /etc/nginx/sites-available/mater
sudo ln -s /etc/nginx/sites-available/mater /etc/nginx/sites-enabled/
sudo systemctl reload nginx
```

#### 3. Configure Nginx Reverse Proxy

```nginx
server {
    listen 80;
    server_name your-domain.com;

    root /var/www/mater;
    index index.html;

    # Frontend SPA
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Backend API proxy
    location /api/ {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## API Documentation

Base URL: `http://localhost:8080`

### Endpoints

#### 1. Get All Players by Team/Year
```http
GET /api/players?team={team}&year={year}
```

**Query Parameters**:
- `team` (required): Team name (e.g., "Duke", "North Carolina")
- `year` (required): Season year (e.g., 2025, 2026)

**Response**: Array of player objects with demographics and season stats

---

#### 2. Get Player Season Averages
```http
GET /api/player-season-averages?team={team}&year={year}
```

**Query Parameters**:
- `team` (required): Team name
- `year` (required): Season year

**Response**: Array of player season averages

---

#### 3. Get Player Rolling Averages
```http
GET /api/player-rolling-averages?team={team}&year={year}&last_n_days={days}
```

**Query Parameters**:
- `team` (required): Team name
- `year` (required): Season year
- `last_n_days` (optional, default: 30): Number of recent days to average

**Response**: Array of player objects with rolling averages and percentiles

**Example Response**:
```json
[
  {
    "player_name": "Kyle Filipowski",
    "team": "Duke",
    "avg_pts": 16.8,
    "pct_pts": 87.5,
    "avg_o_rtg": 118.2,
    "pct_o_rtg": 92.3,
    ...
  }
]
```

---

#### 4. Get Player Stats with Percentiles
```http
GET /api/player-stats-with-percentiles?team={team}&year={year}
```

**Query Parameters**:
- `team` (required): Team name
- `year` (required): Season year

**Response**: Array of player season stats with percentile rankings

---

#### 5. Get Team Statistics
```http
GET /api/team-stats?team={team}&year={year}
```

**Query Parameters**:
- `team` (required): Team name
- `year` (required): Season year

**Response**: Team aggregate statistics

---

#### 6. Get Game Statistics
```http
GET /api/game-stats?team={team}&year={year}&pid={player_id}
```

**Query Parameters**:
- `team` (required): Team name
- `year` (required): Season year
- `pid` (required): Player ID

**Response**: Array of game-by-game statistics for a player

---

### Percentile Calculations

All percentile endpoints calculate rankings across the entire dataset:
- **0-20th percentile**: Below average (red)
- **20-40th percentile**: Below average (orange)
- **40-60th percentile**: Average (yellow)
- **60-80th percentile**: Above average (light green)
- **80-100th percentile**: Elite (dark green)

**Inverted Stats** (lower is better):
- Turnovers (TO%)
- Fouls (FC/40)
- Defensive Rating (D-Rtg)

## Project Structure

```
mater/
├── backend/
│   ├── src/
│   │   ├── main.rs                    # API server & endpoints
│   │   ├── analytics_calculator.rs    # Percentile calculations
│   │   ├── analytics_types.rs         # Data structures
│   │   ├── db_utils.rs                # Database utilities
│   │   ├── init_db.rs                 # Schema initialization
│   │   ├── get_player_stats.rs        # Player queries
│   │   ├── get_team_stats.rs          # Team queries
│   │   └── get_game_stats.rs          # Game queries
│   ├── Cargo.toml                     # Rust dependencies
│   ├── backend.service                # systemd service file
│   └── api_endpoints.md               # API documentation
├── frontend/
│   ├── src/
│   │   ├── App.jsx                    # Main app component
│   │   ├── PlayersTable.jsx           # Player stats table
│   │   ├── PlayerScatterPlot.jsx      # Interactive scatter plot
│   │   ├── PlayerGameStatsTable.jsx   # Game logs
│   │   ├── TeamStatsTable.jsx         # Team stats
│   │   ├── index.css                  # Global styles
│   │   └── main.jsx                   # React entry point
│   ├── public/                        # Static assets
│   ├── package.json                   # Node dependencies
│   ├── vite.config.js                 # Vite configuration
│   ├── nginx.conf                     # Nginx config
│   └── index.html                     # HTML template
├── deploy/
│   ├── deploy.sh                      # Deployment script
│   ├── package_and_deploy.sh          # Build & deploy
│   └── setup_scylla.sh                # Database setup
├── .github/
│   └── workflows/                     # CI/CD workflows
└── README.md                          # This file
```

## Database Schema

### Keyspace
```cql
CREATE KEYSPACE basketball
WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
```

### Tables

#### player_game_stats
Stores individual game statistics for each player.

**Primary Key**: `(team, year, pid, game_date)`
- Partition key: `team, year`
- Clustering keys: `pid, game_date`

**Key Columns**:
- `player_name TEXT`
- `team TEXT`
- `year INT`
- `pid INT` (player ID)
- `game_date TEXT`
- Statistics: `pts, ast, reb, stl, blk, min_per, o_rtg, usg, e_fg, ts_per, etc.`

#### player_season_stats
Season-long aggregated statistics per player.

**Primary Key**: `(team, year, pid)`

**Key Columns**:
- Player demographics: `yr, ht, num, player_type`
- Aggregate stats: Same as game stats, but season totals/averages

#### team_results
Team-level game results and statistics.

**Primary Key**: `(team, year, game_date)`

---

### Indexing Strategy

- **Time-series optimization**: Clustering by `game_date` for efficient range queries
- **Team partitioning**: Data partitioned by team for scalability
- **Player lookups**: Secondary index on `pid` for player-specific queries

## Contributing

### Development Guidelines

1. **Code Style**:
   - Rust: Follow `rustfmt` conventions
   - JavaScript: ESLint configuration enforced
   - Use descriptive variable names

2. **Commit Messages**:
   - Use conventional commits format
   - Include issue references where applicable

3. **Testing**:
   - Add tests for new backend functions
   - Verify mobile responsiveness for frontend changes

4. **Pull Requests**:
   - Include description of changes
   - Reference related issues
   - Ensure CI checks pass

### Roadmap

- [ ] Player comparison tool (head-to-head)
- [ ] Historical data (multi-season trends)
- [ ] Team schedule and predictions
- [ ] User accounts and saved dashboards
- [ ] Export data to CSV/Excel
- [ ] Advanced filtering and search
- [ ] Real-time score updates
- [ ] Player shot charts
- [ ] Lineup analysis

## License

This project is private and proprietary.

## Contact

For questions or issues, please open a GitHub issue or contact the maintainer.

---

**Built with ❤️ for college basketball analytics**
