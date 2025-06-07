pub mod config;
pub mod relay;
pub mod router;
pub mod loadbalance;
pub mod app;

// 重新导出主要的启动函数
pub use app::start_server;
