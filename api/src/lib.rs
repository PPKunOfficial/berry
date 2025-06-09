pub mod config;
pub mod relay;
pub mod loadbalance;
pub mod auth;
pub mod app;
pub mod router;
pub mod static_files;

// 重新导出主要的启动函数
pub use app::start_server;
