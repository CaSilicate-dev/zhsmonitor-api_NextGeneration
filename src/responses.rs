use serde::Serialize;

#[derive(Serialize)]
pub struct ValuePointFloat {
    pub time: String,
    pub value: f64,
}

#[derive(Serialize)]
pub struct ValuePointInt {
    pub time: String,
    pub value: i64,
}

#[derive(Serialize)]
pub struct ServerResponse {
    pub cpustat: Vec<ValuePointFloat>,
    pub memstat: Vec<ValuePointFloat>,
    pub netsendstat: Vec<ValuePointFloat>,
    pub netrecvstat: Vec<ValuePointFloat>,
}

#[derive(Serialize)]
pub struct MinecraftResponse {
    pub latency: Vec<ValuePointFloat>,
    pub players: Vec<ValuePointInt>,
}

#[derive(Serialize)]
pub struct CurrentStatusResponse {
    pub ecs_cpu_avg: f64,
    pub ecs_cpu_stddev: f64,
    pub ecs_mem_avg: f64,
    pub ecs_mem_stddev: f64,
    pub phy_cpu_avg: f64,
    pub phy_cpu_stddev: f64,
    pub phy_mem_avg: f64,
    pub phy_mem_stddev: f64,
    pub mc_latency_avg: f64,
    pub mc_latency_stddev: f64,
    pub mc_current_players: i32,
    pub sysytemdirect_latency_avg: f64,
    pub sysytemdirect_latency_stddev: f64,
    pub sysytemdirect_current_players: i32,
    pub trc_latency_avg: f64,
    pub trc_latency_stddev: f64,
    pub trc_current_players: i32,
}
