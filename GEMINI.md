# Meteora DLMM Project

## Project Overview

The **Meteora API Time Series Collector** is a Python-based application designed to monitor and analyze Dynamic Liquidity Market Maker (DLMM) data from the Meteora API. Its primary goal is to build a high-fidelity time series of liquidity and fee data to better identify investment opportunities, moving beyond simple 24-hour metrics.

The system consists of two main components:
1.  **Data Collector:** A background service that periodically polls the Meteora API and stores historical data in a local DuckDB database.
2.  **Analysis UI:** A Streamlit web application that visualizes the collected data, offering tables and charts to explore opportunities based on a custom "Geek 24h Fee / TVL" metric.

### Key Technologies
*   **Python**: Core programming language.
*   **DuckDB**: High-performance in-process SQL OLAP database for storing time-series data.
*   **Streamlit**: Web framework for the dashboard UI.
*   **APScheduler**: For scheduling the periodic data collection jobs.
*   **Tenacity & Ratelimit**: For robust API interaction handling retries and rate limits.
*   **Altair**: For data visualization in the UI.

## Building and Running

### Using Docker (Recommended)
The project includes a `Dockerfile` for easy deployment.

1.  **Build the Image:**
    ```bash
    docker build --no-cache -t dlmm-time-series .
    ```

2.  **Run the Container:**
    ```bash
    docker run -d -v $(pwd)/data:/data -p 8501:8501 --name dlmm-time-series dlmm-time-series
    ```
    *   Access the UI at `http://localhost:8501`.
    *   Data will be persisted in the `./data` directory.

### Running Locally

1.  **Prerequisites:**
    *   Python 3.x
    *   `pip`

2.  **Installation:**
    ```bash
    # Create virtual environment (optional)
    python -m venv venv
    source venv/bin/activate

    # Install dependencies
    pip install -r requirements.txt
    ```

3.  **Configuration:**
    *   Copy `.env.sample` to `.env`.
    *   Adjust settings like `API_BASE_URL`, `DB_FILENAME`, and rate limits as needed.

4.  **Start the Data Collector:**
    ```bash
    python load_database.py
    ```
    *   This script runs indefinitely, polling the API every minute.

5.  **Start the Web UI:**
    ```bash
    streamlit run app.py
    ```
    *   Access the UI at `http://localhost:8501`.

## Development Conventions

### Code Structure
*   **`app.py`**: Entry point for the Streamlit web application. Handles UI layout, data querying, and visualization.
*   **`load_database.py`**: Entry point for the data collector service.
*   **`meteora_project/`**: Core package containing application logic.
    *   `config.py`: Centralized configuration management using environment variables.
    *   `db.py`: Database management functions (setup, inserts, updates).
    *   `main.py`: Scheduler logic for the data collector.
    *   `apis/`: Modules for interacting with external APIs (Meteora, Jupiter).
    *   `db.sql`: Schema definition for the DuckDB database.

### Database Schema
The database schema is defined in `meteora_project/db.sql`. Key tables include:
*   `tokens`: Stores token metadata (mint address, symbol).
*   `pairs`: Stores DLMM pair information (address, bin step, base fee).
*   `pair_history`: The core time-series table recording price, liquidity, and fees at each collection interval.
*   `v_pair_history`: A complex view that pre-calculates many analysis metrics (volatility, "Geek Fee/TVL", etc.) for the UI.

### Pattern & Style
*   **AsyncIO**: The collector uses `asyncio` for concurrent operations.
*   **SQL-First Analysis**: Much of the heavy lifting for data analysis is done directly in SQL (DuckDB) via the `v_pair_history` view and complex queries in `app.py`.
*   **Robustness**: Extensive use of `tenacity` for retrying failed operations and `ratelimit` to respect API constraints.
