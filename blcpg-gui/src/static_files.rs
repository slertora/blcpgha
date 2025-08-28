// Copyright (c) 2024 Santiago Lertora <santiagolertora@gmail.com>
// Licensed under the MIT License

use tower_http::services::ServeDir;

pub fn static_handler() -> ServeDir {
    ServeDir::new("static")
}
