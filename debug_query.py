import duckdb

SOL_MINT = "So11111111111111111111111111111111111111112"
conn = duckdb.connect('meteora_dlmm_time_series.duckdb', read_only=True)

print("--- Total Pairs ---")
print(conn.execute("SELECT count(*) FROM pairs").fetchone()[0])

print("\n--- Total History Rows ---")
print(conn.execute("SELECT count(*) FROM pair_history").fetchone()[0])

print("\n--- Check SOL Pairs (Raw) ---")
# Check if we have ANY pairs with SOL mint
count_sol = conn.execute(f"""
    SELECT count(*) 
    FROM pairs p
    JOIN tokens t_x ON p.mint_x_id = t_x.id
    JOIN tokens t_y ON p.mint_y_id = t_y.id
    WHERE t_x.mint = '{SOL_MINT}' OR t_y.mint = '{SOL_MINT}'
""").fetchone()[0]
print(f"Pairs involving SOL: {count_sol}")

print("\n--- Check Liquidity > 20k (Raw) ---")
count_liq = conn.execute("SELECT count(*) FROM pair_history WHERE liquidity > 20000").fetchone()[0]
print(f"Rows with Liquidity > 20k: {count_liq}")

print("\n--- Check View v_pair_history ---")
try:
    view_count = conn.execute("SELECT count(*) FROM v_pair_history").fetchone()[0]
    print(f"Rows in v_pair_history: {view_count}")
except Exception as e:
    print(f"Error querying view: {e}")

print("\n--- Test Full Query ---")
query = f"""
        WITH latest_stats AS (
            SELECT 
                p.name,
                p.pair_address,
                h.liquidity,
                v.pct_geek_fees_liquidity_24h as geek_ratio
            FROM pairs p
            JOIN tokens t_x ON p.mint_x_id = t_x.id
            JOIN tokens t_y ON p.mint_y_id = t_y.id
            JOIN v_pair_history v ON p.pair_address = v.pair_address
            JOIN pair_history h ON p.id = h.pair_id
            WHERE 
                (t_x.mint = '{SOL_MINT}' OR t_y.mint = '{SOL_MINT}')
                AND h.liquidity > 20000
                AND h.created_at = (SELECT MAX(created_at) FROM pair_history)
        )
        SELECT * FROM latest_stats
        ORDER BY geek_ratio DESC
        LIMIT 10
"""
results = conn.execute(query).fetchall()
print(f"Results returned: {len(results)}")
for r in results:
    print(r)
