use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use chrono::Local;

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

/// 로그 파일을 초기화한다.
/// 파일 경로: /var/tmp/bear-YYYYMMDDHHMMSS.log (append-only)
pub fn init() {
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let path = format!("/var/tmp/bear-{}.log", timestamp);

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => {
            let _ = LOG_FILE.set(Mutex::new(file));
        }
        Err(err) => eprintln!("로그 파일 생성 실패 ({}): {}", path, err),
    }
}

/// 로그 파일에 한 줄을 기록한다.
/// 형식: "로컬_타임스탬프: 코드_위치: 로그_메시지"
pub fn write_log(location: &str, message: &str) {
    let Some(mutex) = LOG_FILE.get() else { return };
    let Ok(mut file) = mutex.lock() else { return };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let _ = writeln!(file, "{}: {}: {}", timestamp, location, message);
    let _ = file.flush();
}

/// CLI 실행 로그를 기록하는 매크로.
/// 호출 지점의 파일 경로와 라인 번호를 자동으로 코드 위치에 포함한다.
#[macro_export]
macro_rules! cli_log {
    ($($arg:tt)*) => {{
        let __cli_log_location = format!("{}:{}", file!(), line!());
        $crate::claude_code_client::logger::write_log(
            &__cli_log_location,
            &format!($($arg)*),
        );
    }};
}
