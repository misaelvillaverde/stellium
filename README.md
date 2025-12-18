# Stellium

An MCP (Model Context Protocol) server for astrological calculations using the Swiss Ephemeris library. Provides natal chart storage, transit calculations, synastry analysis, and more.

## Features

- **Natal Chart Storage** - Store birth charts with full planetary positions and house cusps
- **House System** - Full Placidus house calculations with all 12 cusps
- **North Node** - True Lunar Node included in all calculations
- **Transit Analysis** - Daily transits with aspects to natal planets
- **Retrograde Tracking** - Current and upcoming retrograde periods
- **Lunar Information** - Moon phases and void-of-course periods
- **Synastry** - Compatibility analysis between two charts

## Installation

### Prerequisites

- Rust 1.70+
- Clang (for Swiss Ephemeris compilation)

### Build

```bash
git clone https://github.com/misaelvillaverde/stellium.git
cd stellium
cargo build --release
```

The binary will be at `./target/release/stellium`

## Claude Desktop Configuration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "stellium": {
      "command": "/path/to/stellium"
    }
  }
}
```

## Tools

### Chart Management

#### `store_natal_chart`
Store a natal chart with birth data.

```json
{
  "name": "John Doe",
  "birth_date": "1990-05-15",
  "birth_time": "14:30:00",
  "birth_location": "New York, NY",
  "latitude": 40.7128,
  "longitude": -74.0060,
  "timezone": "America/New_York"
}
```

#### `get_natal_chart`
Retrieve a stored natal chart by name.

#### `list_natal_charts`
List all stored natal charts.

#### `search_natal_charts`
Search charts by name (case-insensitive partial match).

```json
{
  "query": "john"
}
```

#### `delete_natal_chart`
Delete a chart (requires name and birth date for confirmation).

```json
{
  "name": "John Doe",
  "birth_date": "1990-05-15"
}
```

### Astrological Analysis

#### `get_daily_transits`
Get current planetary positions and aspects to your natal chart.

```json
{
  "date": "2024-12-18"
}
```

#### `get_retrograde_status`
Check which planets are retrograde and upcoming retrograde periods.

```json
{
  "date": "2024-12-18",
  "include_upcoming": true,
  "days_ahead": 90
}
```

#### `get_lunar_info`
Get lunar phase, void-of-course status, and cycle dates.

```json
{
  "date": "2024-12-18"
}
```

#### `get_transit_report`
Get major astrological events over a date range.

```json
{
  "start_date": "2024-12-01",
  "end_date": "2024-12-31",
  "include_minor_aspects": false
}
```

#### `get_compatibility`
Analyze synastry between two natal charts.

```json
{
  "person1_name": "John",
  "person2_name": "Jane",
  "include_minor_aspects": false
}
```

Returns:
- All inter-chart aspects with orbs
- House placements for context
- Exact aspects highlighted (< 1° orb)
- Summary of harmonious vs challenging aspects

## Data Storage

Natal charts are stored persistently in:
- **macOS**: `~/Library/Application Support/com.stellium.stellium/`
- **Linux**: `~/.local/share/stellium/`
- **Windows**: `%APPDATA%\stellium\`

Charts are keyed by name + birth date to prevent duplicates.

## Technical Details

- **Ephemeris**: Uses Swiss Ephemeris via [libswisseph-sys](https://crates.io/crates/libswisseph-sys)
- **Precision**: Moshier analytical ephemeris (0.1 arc seconds for planets, 3 arc seconds for Moon)
- **House System**: Placidus (default)
- **Aspects**: Conjunction, Sextile, Square, Trine, Opposition (+ minor aspects optional)

## License

MIT
