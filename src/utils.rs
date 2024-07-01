use atty::Stream;
use colored::Colorize;
use anyhow::Result;

use crate::haproxy::HaproxyLogEntry;

pub fn is_stdin_redirected() -> Result<bool> {
    if atty::is(Stream::Stdin) {
        return Ok(false);
    }

    Ok(true)
}

pub fn output_table(entry: &HaproxyLogEntry) -> Result<String> {
    let mut result = "".to_string();
    
    result.push_str(&format!("{}: {}\n", "Month".bold(), entry.month.white()));
    result.push_str(&format!("{}: {}\n", "Day".bold(), entry.day.white()));
    result.push_str(&format!("{}: {}\n", "Time".bold(), entry.time.white()));
    result.push_str(&format!("{}: {}\n", "Host".bold(), entry.host.white()));
    result.push_str(&format!("{}: {}\n", "Process ID".bold(), entry.process_id.white()));
    result.push_str(&format!("{}: {}\n", "Source IP Port".bold(), entry.source_ip_port.white()));
    result.push_str(&format!("{}: {}\n", "Time Stamp Accepted".bold(), entry.time_stamp_accepted.white()));
    result.push_str(&format!("{}: {}\n", "Frontend Name".bold(), entry.frontend_name.purple()));
    result.push_str(&format!("{}: {}\n", "Backend Name".bold(), entry.backend_name.yellow()));
    result.push_str(&format!("{}: {}\n", "Server Name".bold(), entry.server_name.blue()));
    result.push_str(&format!("{}: {}\n", "Timers".bold(), entry.timers.to_string().white()));

    result.push_str(&format!("∟ {}: {}\n", "Client Request".bold(), entry.timers.client_request.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Queue Wait".bold(), entry.timers.queue_wait.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Establish".bold(), entry.timers.establish.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Server Response".bold(), entry.timers.server_response.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Total".bold(), entry.timers.total.to_string().white()));

    result.push_str(&format!("{}: {}\n", "Response Code".bold(), match entry.response_code.as_str().parse::<u16>() {
        Ok(code) => {
            if code >= 200 && code < 300 {
                entry.response_code.green()
            } else if code >= 300 && code < 400 {
                entry.response_code.yellow()
            } else if code >= 400 {
                entry.response_code.red()
            } else {
                entry.response_code.white()
            }
        }
        Err(_) => entry.response_code.white()
    }));
    result.push_str(&format!("{}: {}\n", "Bytes Read".bold(), entry.bytes_read.white()));
    result.push_str(&format!("{}: {}\n", "Termination State".bold(), match entry.termination_state.is_error() {
        false => entry.termination_state.to_string().green(),
        true => entry.termination_state.to_string().red()
    }));

    result.push_str(&format!("∟ {}: {}\n", "Termination Reason".bold(), entry.termination_state.termination_reason.description.white()));
    result.push_str(&format!("∟ {}: {}\n", "Session State".bold(), entry.termination_state.session_state.description.white()));
    result.push_str(&format!("∟ {}: {}\n", "Persistence Cookie".bold(), entry.termination_state.persistence_cookie.description.white()));
    result.push_str(&format!("∟ {}: {}\n", "Persistence Operations".bold(), entry.termination_state.persistence_operations.description.white()));

    result.push_str(&format!("{}: {}\n", "Connection Counts".bold(), entry.conn_counts.to_string().white()));

    result.push_str(&format!("∟ {}: {}\n", "Current".bold(), entry.conn_counts.current.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Limit".bold(), entry.conn_counts.limit.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Max".bold(), entry.conn_counts.max.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Total".bold(), entry.conn_counts.total.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Rejected".bold(), entry.conn_counts.rejected.to_string().white()));

    result.push_str(&format!("{}: {}\n", "Queue".bold(), entry.queue.to_string().white()));

    result.push_str(&format!("∟ {}: {}\n", "Server".bold(), entry.queue.server.to_string().white()));
    result.push_str(&format!("∟ {}: {}\n", "Backend".bold(), entry.queue.backend.to_string().white()));

    result.push_str(&format!("{}: {}\n", "Request".bold(), entry.request.white()));

    Ok(result)
}

#[cfg(unix)]
pub fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
pub fn reset_sigpipe() {
    // no-op
}
