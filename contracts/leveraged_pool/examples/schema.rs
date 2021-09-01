use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use leveraged_pools::pool::{ExecuteMsg,HyperparametersResponse,
    InstantiateMsg, QueryMsg, TSPricePoint, LeveragedPricePoint,
    AssetPriceHistoryResponse};
use leveraged_pool::state::Hyperparameters;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Hyperparameters), &out_dir);
    export_schema(&schema_for!(TSPricePoint), &out_dir);
    export_schema(&schema_for!(LeveragedPricePoint), &out_dir);
    export_schema(&schema_for!(AssetPriceHistoryResponse), &out_dir);
    export_schema(&schema_for!(HyperparametersResponse), &out_dir);
}
