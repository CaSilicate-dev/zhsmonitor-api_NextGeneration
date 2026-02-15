mod generate_sql;
mod responses;
mod generate_leveled_sql;

use sqlx::mysql::MySqlPoolOptions;
use axum::{routing::get, Router, response::{Json, IntoResponse}, http::StatusCode, extract::{State, Query, }};
use serde::Deserialize;
use std::fs;
use sqlx::{MySqlPool, Row};
use generate_sql::{CompressionType, DataType};
use crate::responses::{ValuePointFloat, ValuePointInt};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use sqlx::types::BigDecimal;
use crate::generate_leveled_sql::gl_current_general;
use tower_http::cors::{CorsLayer, Any};

#[derive(Clone, Deserialize)]
struct AppConfig {
    bing_addr: String,
    db_url: String,
}

#[derive(Clone)]
struct AppState {
    pool: MySqlPool,
    //config: AppConfig,
}

#[derive(Deserialize)]
struct QueryParams {
    level: i32,
}

fn advanced_round(x: f64, decimal: i32) -> f64 {
    (x * (10f64.powi(decimal))).round() / (10f64.powi(decimal))
}
fn auto_round(x: f64) -> f64 {
    let mut value = x;
    if value != -1.0 {
        value /= 1048576.0;
        if value < 0.1 {
            advanced_round(value, 4)
        } else if value < 1.0 {
            advanced_round(value, 3)
        } else if value < 10.0 {
            advanced_round(value, 2)
        } else if value < 100.0 {
            advanced_round(value, 1)
        } else {
            value.round()
        }
    } else {
        -1.0
    }
}
async fn root_stat_ecs(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql = match generate_leveled_sql::gl_server_stat("ecs_stat", level) {
        Ok(a) => a,
        Err(_e) => {
            return (StatusCode::BAD_GATEWAY, "Invalid level").into_response();
        }
    };

    let rows = match sqlx::query(sql.as_str()).fetch_all(pool).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let mut resp: responses::ServerResponse = responses::ServerResponse {
        cpustat: Vec::new(),
        memstat: Vec::new(),
        netsendstat: Vec::new(),
        netrecvstat: Vec::new(),
    };

    for row in rows {
        let time: String = row.try_get("create_time").unwrap_or("").to_string();
        let cpu = row.try_get("cpu_usage").unwrap_or(-1.0);
        let mem = row.try_get("memory_usage").unwrap_or(-1.0);
        let mut net_send_rate = row.try_get("net_send_rate").unwrap_or(BigDecimal::from_f64(-1.0).unwrap()).to_f64().unwrap();
        let mut net_recv_rate = row.try_get("net_recv_rate").unwrap_or(BigDecimal::from_f64(-1.0).unwrap()).to_f64().unwrap();

        net_send_rate = auto_round(net_send_rate);
        net_recv_rate = auto_round(net_recv_rate);
        
        resp.cpustat.push(ValuePointFloat {time: time.clone(), value: cpu});
        resp.memstat.push(ValuePointFloat {time: time.clone(), value: mem});
        resp.netsendstat.push(ValuePointFloat {time: time.clone(), value: net_send_rate});
        resp.netrecvstat.push(ValuePointFloat {time, value: net_recv_rate});
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn root_stat_physical(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql = match generate_leveled_sql::gl_server_stat("physical_stat", level) {
        Ok(a) => a,
        Err(_e) => {
            return (StatusCode::BAD_GATEWAY, "Invalid level").into_response();
        }
    };

    let rows = match sqlx::query(sql.as_str()).fetch_all(pool).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let mut resp: responses::ServerResponse = responses::ServerResponse {
        cpustat: Vec::new(),
        memstat: Vec::new(),
        netsendstat: Vec::new(),
        netrecvstat: Vec::new(),
    };

    for row in rows {
        let time = row.try_get("create_time").unwrap_or("").to_string();
        let cpu = row.try_get("cpu_usage").unwrap_or(-1.0);
        let mem = row.try_get("memory_usage").unwrap_or(-1.0);
        let mut net_send_rate = row.try_get("net_send_rate").unwrap_or(BigDecimal::from_f64(-1.0).unwrap()).to_f64().unwrap();
        let mut net_recv_rate = row.try_get("net_recv_rate").unwrap_or(BigDecimal::from_f64(-1.0).unwrap()).to_f64().unwrap();
        if net_send_rate != -1.0 {
            net_send_rate /= 1048576.0;
        }
        if net_recv_rate != -1.0 {
            net_recv_rate /= 1048576.0;
        }
        resp.cpustat.push(ValuePointFloat {time: time.clone(), value: cpu});
        resp.memstat.push(ValuePointFloat {time: time.clone(), value: mem});
        resp.netsendstat.push(ValuePointFloat {time: time.clone(), value: net_send_rate});
        resp.netrecvstat.push(ValuePointFloat {time, value: net_recv_rate});
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn root_stat_mc(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql = match generate_leveled_sql::gl_mc_stat("mcserver_stat", level) {
        Ok(a) => a,
        Err(_e) => {
            return (StatusCode::BAD_GATEWAY, "Invalid level").into_response();
        }
    };

    let rows = match sqlx::query(sql.as_str()).fetch_all(pool).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let mut resp = responses::MinecraftResponse {
        latency: Vec::new(),
        players: Vec::new(),
    };

    for row in rows {
        let time = row.try_get("create_time").unwrap_or("").to_string();
        let latency = row.try_get("latency").unwrap_or(-1.0);
        let players = row.try_get("players").unwrap_or(-1);
        resp.latency.push(ValuePointFloat {time: time.clone(), value: latency});
        resp.players.push(ValuePointInt {time, value: players});
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn root_stat_sysytemdirect(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql = match generate_leveled_sql::gl_mc_stat("sysytemdirect_stat", level) {
        Ok(a) => a,
        Err(_e) => {
            return (StatusCode::BAD_GATEWAY, "Invalid level").into_response();
        }
    };

    let rows = match sqlx::query(sql.as_str()).fetch_all(pool).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let mut resp = responses::MinecraftResponse {
        latency: Vec::new(),
        players: Vec::new(),
    };

    for row in rows {
        let time = row.try_get("create_time").unwrap_or("").to_string();
        let latency = row.try_get("latency").unwrap_or(-1.0);
        let players = row.try_get("players").unwrap_or(-1);
        resp.latency.push(ValuePointFloat {time: time.clone(), value: latency});
        resp.players.push(ValuePointInt {time, value: players});
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn root_stat_trc(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql = match generate_leveled_sql::gl_mc_stat("trc_stat", level) {
        Ok(a) => a,
        Err(_e) => {
            return (StatusCode::BAD_GATEWAY, "Invalid level").into_response();
        }
    };

    let rows = match sqlx::query(sql.as_str()).fetch_all(pool).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let mut resp = responses::MinecraftResponse {
        latency: Vec::new(),
        players: Vec::new(),
    };

    for row in rows {
        let time = row.try_get("create_time").unwrap_or("").to_string();
        let latency = row.try_get("latency").unwrap_or(-1.0);
        let players = row.try_get("players").unwrap_or(-1);
        resp.latency.push(ValuePointFloat {time: time.clone(), value: latency});
        resp.players.push(ValuePointInt {time, value: players});
    }

    (StatusCode::OK, Json(resp)).into_response()
}

async fn get_value(pool: &MySqlPool, sql: &str) -> Result<f64, sqlx::Error> {
    let value = sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await;
    match value {
        Ok(v) => Ok(v as f64),
        Err(_) => {
            let v = sqlx::query_scalar::<_, f64>(sql)
                .fetch_one(pool)
                .await?;
            Ok(v)
        }
    }
}

async fn root_stat_current(State(state): State<AppState>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let level = params.level;
    let pool = &state.pool;

    let sql_ecs_cpu_avg = match gl_current_general(CompressionType::Average, DataType::CPU, "ecs_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_ecs_cpu_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::CPU, "ecs_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_ecs_mem_avg = match gl_current_general(CompressionType::Average, DataType::Memory, "ecs_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_ecs_mem_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::Memory, "ecs_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_phy_cpu_avg = match gl_current_general(CompressionType::Average, DataType::CPU, "physical_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_phy_cpu_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::CPU, "physical_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_phy_mem_avg = match gl_current_general(CompressionType::Average, DataType::Memory, "physical_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_phy_mem_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::Memory, "physical_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_mc_lac_avg = match gl_current_general(CompressionType::Average, DataType::Latency, "mcserver_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_mc_lac_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::Latency, "mcserver_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_mc_cur_ply = generate_sql::generate_current_players("mcserver_stat");
    let sql_sds_lac_avg = match gl_current_general(CompressionType::Average, DataType::Latency, "sysytemdirect_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_sds_lac_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::Latency, "sysytemdirect_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_sds_cur_ply = generate_sql::generate_current_players("sysytemdirect_stat");
    let sql_trc_lac_avg = match gl_current_general(CompressionType::Average, DataType::Latency, "trc_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_trc_lac_stv = match gl_current_general(CompressionType::StandardDeviation, DataType::Latency, "trc_stat", level) {Ok(a) => a, Err(e) => {eprintln!("{}", e); return (StatusCode::BAD_REQUEST, "Invalid level").into_response()}};
    let sql_trc_cur_ply = generate_sql::generate_current_players("trc_stat");

    let sqls = vec![
        sql_ecs_cpu_avg,
        sql_ecs_cpu_stv,
        sql_ecs_mem_avg,
        sql_ecs_mem_stv,
        sql_phy_cpu_avg,
        sql_phy_cpu_stv,
        sql_phy_mem_avg,
        sql_phy_mem_stv,
        sql_mc_lac_avg,
        sql_mc_lac_stv,
        sql_mc_cur_ply,
        sql_sds_lac_avg,
        sql_sds_lac_stv,
        sql_sds_cur_ply,
        sql_trc_lac_avg,
        sql_trc_lac_stv,
        sql_trc_cur_ply,
    ];



    let mut results = Vec::with_capacity(sqls.len());

    for sql in sqls {
        let value: f64 = match get_value(&pool, sql.as_str()).await {
            Ok(a) => {
                advanced_round(a, 3)
            },
            Err(e) => {
                eprintln!("{}", e);
                -1.0

            }
        };
        results.push(value);
    }

    let resp = responses::CurrentStatusResponse {
        ecs_cpu_avg: *results.get(0).unwrap(),
        ecs_cpu_stddev: *results.get(1).unwrap(),
        ecs_mem_avg: *results.get(2).unwrap(),
        ecs_mem_stddev: *results.get(3).unwrap(),
        phy_cpu_avg: *results.get(4).unwrap(),
        phy_cpu_stddev: *results.get(5).unwrap(),
        phy_mem_avg: *results.get(6).unwrap(),
        phy_mem_stddev: *results.get(7).unwrap(),
        mc_latency_avg: *results.get(8).unwrap(),
        mc_latency_stddev: *results.get(9).unwrap(),
        mc_current_players: *results.get(10).unwrap() as i32,
        sysytemdirect_latency_avg: *results.get(11).unwrap(),
        sysytemdirect_latency_stddev: *results.get(12).unwrap(),
        sysytemdirect_current_players: *results.get(13).unwrap() as i32,
        trc_latency_avg: *results.get(14).unwrap(),
        trc_latency_stddev: *results.get(15).unwrap(),
        trc_current_players: *results.get(16).unwrap() as i32,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

#[tokio::main]
async fn main() {

    let content = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: AppConfig = serde_json::from_str(content.as_str()).unwrap();

    let pool = MySqlPoolOptions::new().max_connections(10).min_connections(5).connect(config.db_url.as_str()).await.expect("Failed to connect to database");

    let app_state = AppState {
        pool,
        //config: config.clone(),
    };

    let app = Router::new()
        .route("/stat/ecs", get(root_stat_ecs))
        .route("/stat/physical", get(root_stat_physical))
        .route("/stat/mc", get(root_stat_mc))
        .route("/stat/sysytemdirect", get(root_stat_sysytemdirect))
        .route("/stat/trc", get(root_stat_trc))
        .route("/stat/current", get(root_stat_current))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));
    let listener = tokio::net::TcpListener::bind(config.bing_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
