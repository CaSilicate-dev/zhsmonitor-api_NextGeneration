pub fn generate_server_stat(table: &str, time_interval: &str, bucket_size: &str) -> String {
    format!(
        r#"
SELECT
    parted_time.bucket_time as create_time,
    ROUND(AVG(parted_time.cpu_usage), 1) AS cpu_usage,
    ROUND(AVG(parted_time.memory_usage), 1) AS memory_usage,
    ROUND(AVG(parted_time.net_send_rate), 1) AS net_send_rate,
    ROUND(AVG(parted_time.net_recv_rate), 1) AS net_recv_rate
FROM (
    SELECT
        DATE_FORMAT(
            FROM_UNIXTIME(FLOOR(UNIX_TIMESTAMP(create_time) / {cs}) * {cs}),
            '%Y-%m-%d %H:%i:%s'
        ) AS bucket_time,
        cpu_usage,
        memory_usage,
        net_send_rate,
        net_recv_rate
    FROM {table}
    WHERE create_time > DATE_SUB(NOW(), INTERVAL {maxirv} SECOND) AND deleted = 0
) AS parted_time
GROUP BY parted_time.bucket_time
ORDER BY parted_time.bucket_time;
    "#,
        cs = bucket_size,
        table = table,
        maxirv = time_interval
    )
}

pub fn generate_mc_stat(table: &str, time_interval: &str, bucket_size: &str) -> String {
    format!(
        r#"
SELECT
    parted_time.bucket_time as create_time,
    ROUND(AVG(parted_time.latency), 1) AS latency,
    MAX(parted_time.players) AS players
FROM (
    SELECT
        DATE_FORMAT(
            FROM_UNIXTIME(FLOOR(UNIX_TIMESTAMP(create_time) / {cs}) * {cs}),
            '%Y-%m-%d %H:%i:%s'
        ) AS bucket_time,
        latency,
        players
    FROM {table}
    WHERE create_time > DATE_SUB(NOW(), INTERVAL {maxirv} SECOND) AND deleted = 0
) AS parted_time
GROUP BY parted_time.bucket_time
ORDER BY parted_time.bucket_time;
    "#,
        cs = bucket_size,
        table = table,
        maxirv = time_interval
    )
}

pub enum CompressionType {
    Average,
    StandardDeviation,
}

pub enum DataType {
    CPU,
    Memory,
    Latency,
}
pub fn generate_current_general(
    compression_type: CompressionType,
    data_type: DataType,
    table: &str,
    interval: i32,
) -> String {
    let algorithm;
    let field;
    let mut extra = "".to_string();
    match compression_type {
        CompressionType::Average => algorithm = "AVG",
        CompressionType::StandardDeviation => algorithm = "STDDEV",
    }
    match data_type {
        DataType::CPU => field = "cpu_usage",
        DataType::Memory => field = "memory_usage",
        DataType::Latency => {
            extra = "AND latency != 5000 ".to_string();
            field = "latency";
        }
    }

    format!("SELECT {algo}({type}) FROM {table} WHERE create_time > DATE_SUB(NOW(), INTERVAL {interval} SECOND) {extra}AND deleted = 0;",
            algo=algorithm, type=field, table=table, interval=interval)
}
pub fn generate_current_players(table: &str) -> String {
    format!(
        "SELECT players FROM {table} WHERE deleted = 0 ORDER BY create_time DESC LIMIT 1;",
        table = table
    )
}
