"""
All of these calls cross the FFI boundary.
"""

import driftpyrs as d

AUTHORITY = "J1TnP8zvVxbtF5KFp5xRmWuvG9McnhzmBd9XGfCyuxFP"

print("=== Program IDs ===")
print(f"program_id:                 {d.get_program_id()}")
print(f"vault_program_id:           {d.get_vault_program_id()}")
print(f"jit_proxy_id:               {d.get_jit_proxy_id()}")
print(f"token_program_id:           {d.get_token_program_id()}")
print(f"token_2022_program_id:      {d.get_token_2022_program_id()}")
print(f"associated_token_program:   {d.get_associated_token_program_id()}")

print("\n=== State & Market PDAs ===")
print(f"state_account:              {d.get_state_account()}")
print(f"perp_market(0):             {d.derive_perp_market_account(0)}")
print(f"spot_market(0):             {d.derive_spot_market_account(0)}")
print(f"spot_market_vault(0):       {d.derive_spot_market_vault(0)}")
print(f"drift_signer:               {d.derive_drift_signer()}")

print("\n=== User PDAs ===")
print(f"user_account(sub=0):        {d.derive_user_account(AUTHORITY, 0)}")
print(f"user_account(sub=1):        {d.derive_user_account(AUTHORITY, 1)}")
print(f"stats_account:              {d.derive_stats_account(AUTHORITY)}")
print(f"swift_order_account:        {d.derive_swift_order_account(AUTHORITY)}")
print(f"revenue_share:              {d.derive_revenue_share(AUTHORITY)}")
print(f"revenue_share_escrow:       {d.derive_revenue_share_escrow(AUTHORITY)}")

print("\n=== Oracle PDAs ===")
print(f"pyth_lazer_oracle(6):       {d.derive_pyth_lazer_oracle(6)}")

print("\n=== Math Functions ===")
print(
    f"standardize_price(1_000_500, 1000, 'long'):   {d.standardize_price(1_000_500, 1000, 'long')}"
)
print(
    f"standardize_price(1_000_500, 1000, 'short'):  {d.standardize_price(1_000_500, 1000, 'short')}"
)
print(
    f"standardize_price_i64(-500, 100, 'long'):     {d.standardize_price_i64(-500, 100, 'long')}"
)
print(
    f"standardize_price_i64(-500, 100, 'short'):    {d.standardize_price_i64(-500, 100, 'short')}"
)
print(
    f"standardize_base_asset_amount(1_500_000, 1_000_000):       {d.standardize_base_asset_amount(1_500_000, 1_000_000)}"
)
print(
    f"standardize_base_asset_amount_ceil(1_500_000, 1_000_000):  {d.standardize_base_asset_amount_ceil(1_500_000, 1_000_000)}"
)

print("\n=== URL Utils ===")
print(
    f"http_to_ws('https://api.mainnet.solana.com'):  {d.http_to_ws('https://api.mainnet.solana.com')}"
)
print(
    f"get_ws_url('https://api.mainnet.solana.com'):  {d.get_ws_url('https://api.mainnet.solana.com')}"
)
print(
    f"get_http_url('wss://api.mainnet.solana.com'):  {d.get_http_url('wss://api.mainnet.solana.com')}"
)

print("\n=== Pyth Lazer Mappings ===")
print(f"feed_id 6 -> perp market:   {d.pyth_lazer.feed_id_to_perp_market_index(6)}")
print(f"perp market 0 -> feed_id:   {d.pyth_lazer.perp_market_index_to_feed_id(0)}")
print(f"unknown feed_id 99999:      {d.pyth_lazer.feed_id_to_perp_market_index(99999)}")

print("\n✓ All functions called successfully!")
