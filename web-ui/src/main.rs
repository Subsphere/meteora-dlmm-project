use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use duckdb::{Connection, Config, AccessMode};
use serde::Serialize;
use askama::Template;

// Define the SOL mint address to filter by
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Serialize)]
struct Opportunity {
    name: String,
    pair_address: String,
    bin_step: i32,
    base_fee_percentage: f64,
    liquidity: f64,
    fees_24h: f64,
    geek_ratio: f64,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

struct AppState {
    db_path: String,
}

#[get("/")]
async fn index() -> impl Responder {
    let html = IndexTemplate.render().unwrap_or_else(|_| "Template Error".to_string());
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[get("/api/opportunities")]
async fn get_opportunities(data: web::Data<AppState>) -> impl Responder {
    // Open DB in read-only mode for each request (DuckDB handles concurrency well)
    // We open it every time to ensure we see the latest data written by the Python script
    let config = Config::default().access_mode(AccessMode::ReadOnly).unwrap();
    let conn = match Connection::open_with_flags(&data.db_path, config) {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("DB Connection Error: {}", e)),
    };

    let query = format!(r#"
        WITH latest_stats AS (
            SELECT 
                p.name,
                p.pair_address,
                p.bin_step,
                p.base_fee_percentage,
                h.liquidity,
                -- Estimate 24h fees based on 'geek fee' metric logic from Python app
                -- (pct_geek_fees_liquidity_24h is roughly (24h_fees / liquidity) * 100)
                -- We select the latest row from v_pair_history
                v.pct_geek_fees_liquidity_24h as geek_ratio,
                (v.pct_geek_fees_liquidity_24h / 100.0) * h.liquidity as fees_24h
            FROM pairs p
            JOIN tokens t_x ON p.mint_x_id = t_x.id
            JOIN tokens t_y ON p.mint_y_id = t_y.id
            -- Join with view to get the pre-calculated metrics
            JOIN v_pair_history v ON p.pair_address = v.pair_address
            JOIN pair_history h ON p.id = h.pair_id
            WHERE 
                -- Filter 1: SOL Pairs Only (one side must be SOL)
                (t_x.mint = '{sol_mint}' OR t_y.mint = '{sol_mint}')
                
                -- Filter 2: Liquidity > $20k
                AND h.liquidity > 20000
                
                -- Get only the latest data point
                AND h.created_at = (SELECT MAX(created_at) FROM pair_history)
                -- Link view timestamp to history timestamp to ensure we get the matching stats
                AND v.dttm = h.created_at
        )
        SELECT * FROM latest_stats
        ORDER BY geek_ratio DESC
        LIMIT 100
    "#, sol_mint = SOL_MINT);

    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Query Prep Error: {}", e)),
    };

    let opportunities_iter = match stmt.query_map([], |row| {
        Ok(Opportunity {
            name: row.get(0)?,
            pair_address: row.get(1)?,
            bin_step: row.get(2)?,
            base_fee_percentage: row.get(3)?,
            liquidity: row.get(4)?,
            geek_ratio: row.get(5)?,
            fees_24h: row.get(6)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Query Exec Error: {}", e)),
    };

    let mut opportunities = Vec::new();
    for opp in opportunities_iter {
        if let Ok(o) = opp {
            opportunities.push(o);
        }
    }

    HttpResponse::Ok().json(opportunities)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    // Default to the file path used by the Python script
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "./meteora_dlmm_time_series.duckdb".to_string());

    println!("Starting server at http://0.0.0.0:8080");
    println!("Reading DB from: {}", db_path);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_path: db_path.clone(),
            }))
            .service(index)
            .service(get_opportunities)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}