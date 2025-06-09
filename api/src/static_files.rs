use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};
use mime_guess::from_path;

// 在编译时嵌入整个 public 目录
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../public");

/// 处理静态文件请求
pub async fn serve_static_file(Path(path): Path<String>) -> impl IntoResponse {
    serve_file(&path).await
}

/// 处理根路径的静态文件请求（默认返回 index.html）
pub async fn serve_index() -> impl IntoResponse {
    serve_file("index.html").await
}

/// 内部函数：根据路径提供文件
async fn serve_file(path: &str) -> Response {
    // 清理路径，移除开头的斜杠
    let clean_path = path.trim_start_matches('/');
    
    // 如果路径为空，默认返回 index.html
    let file_path = if clean_path.is_empty() {
        "index.html"
    } else {
        clean_path
    };

    // 尝试从嵌入的目录中获取文件
    match STATIC_DIR.get_file(file_path) {
        Some(file) => {
            // 获取文件内容
            let contents = file.contents();
            
            // 根据文件扩展名猜测 MIME 类型
            let mime_type = from_path(file_path)
                .first_or_octet_stream()
                .to_string();

            // 创建响应
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::CACHE_CONTROL, "public, max-age=3600") // 缓存1小时
                .body(contents.into())
                .unwrap()
        }
        None => {
            // 文件不存在，尝试返回 404.html
            match STATIC_DIR.get_file("404.html") {
                Some(not_found_file) => {
                    let contents = not_found_file.contents();
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(contents.into())
                        .unwrap()
                }
                None => {
                    // 如果连 404.html 都没有，返回简单的文本响应
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                        .body("404 - File Not Found".into())
                        .unwrap()
                }
            }
        }
    }
}

/// 列出所有嵌入的静态文件（用于调试）
pub fn list_embedded_files() -> Vec<String> {
    let mut files = Vec::new();
    collect_files(&STATIC_DIR, "", &mut files);
    files
}

/// 递归收集目录中的所有文件
fn collect_files(dir: &Dir, prefix: &str, files: &mut Vec<String>) {
    for file in dir.files() {
        let path = if prefix.is_empty() {
            file.path().to_string_lossy().to_string()
        } else {
            format!("{}/{}", prefix, file.path().to_string_lossy())
        };
        files.push(path);
    }
    
    for subdir in dir.dirs() {
        let new_prefix = if prefix.is_empty() {
            subdir.path().to_string_lossy().to_string()
        } else {
            format!("{}/{}", prefix, subdir.path().to_string_lossy())
        };
        collect_files(subdir, &new_prefix, files);
    }
}

/// 获取嵌入文件的信息（用于状态页面）
pub fn get_static_files_info() -> serde_json::Value {
    let files = list_embedded_files();
    serde_json::json!({
        "embedded_files_count": files.len(),
        "files": files
    })
}
