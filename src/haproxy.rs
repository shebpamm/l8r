use colored::Colorize;
use serde::Serialize;
use crate::RE;

#[derive(Debug, Serialize)]
pub struct HaproxyTimers {
    pub raw: String,
    pub client_request: u64,
    pub queue_wait: u64,
    pub establish: u64,
    pub server_response: u64,
    pub total: u64,
}

impl HaproxyTimers {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 5 {
            return Err("Failed to parse timers".into());
        }

        Ok(HaproxyTimers {
            raw: s.to_string(),
            client_request: parts[0].parse()?,
            queue_wait: parts[1].parse()?,
            establish: parts[2].parse()?,
            server_response: parts[3].parse()?,
            total: parts[4].parse()?,
        })
    }
}

impl std::fmt::Display for HaproxyTimers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}/{}/{}", self.client_request, self.queue_wait, self.establish, self.server_response, self.total)
    }
}



#[derive(Debug, Serialize, PartialEq)]
pub struct HaproxyTerminationStateEntry {
    pub shorthand: char,
    pub description: String,
}

impl HaproxyTerminationStateEntry {
    pub fn reason(shorthand: char) -> HaproxyTerminationStateEntry {
        let description = match shorthand {
            'C' => "the TCP session was unexpectedly aborted by the client.",
            'S' => "the TCP session was unexpectedly aborted by the server, or the \
                   server explicitly refused it.",
            'P' => "the session was prematurely aborted by the proxy, because of a \
                   connection limit enforcement, because a DENY filter was matched, \
                   because of a security check which detected and blocked a dangerous \
                   error in server response which might have caused information leak \
                   (e.g. cacheable cookie).",
            'L' => "the session was locally processed by HAProxy and was not passed to \
                   a server. This is what happens for stats and redirects.",
            'R' => "a resource on the proxy has been exhausted (memory, sockets, source \
                   ports, ...). Usually, this appears during the connection phase, and \
                   system logs should contain a copy of the precise error. If this \
                   happens, it must be considered as a very serious anomaly which \
                   should be fixed as soon as possible by any means.",
            'I' => "an internal error was identified by the proxy during a self-check. \
                   This should NEVER happen, and you are encouraged to report any log \
                   containing this, because this would almost certainly be a bug. It \
                   would be wise to preventively restart the process after such an \
                   event too, in case it would be caused by memory corruption.",
            'D' => "the session was killed by HAProxy because the server was detected as down and was configured to kill all connections when going down.",
            'U' => "the session was killed by HAProxy on this backup server because an \
                   active server was detected as up and was configured to kill all \
                   backup connections when going up.",
            'K' => "the session was actively killed by an admin operating on HAProxy.",
            'c' => "the client-side timeout expired while waiting for the client to send or receive data.",
            's' => "the server-side timeout expired while waiting for the server to send or receive data.",
            '-' => "normal session completion, both the client and the server closed with nothing left in the buffers.",
            _ => "Unknown termination state"
        };
        let description = description.to_string();
        HaproxyTerminationStateEntry {
            shorthand,
            description
        }
    }

    pub fn state(shorthand: char) -> HaproxyTerminationStateEntry {
        let description = match shorthand {
            'R' => "the proxy was waiting for a complete, valid REQUEST from the client (HTTP mode only). Nothing was sent to any server.",
            'Q' => "the proxy was waiting in the QUEUE for a connection slot. This can only happen when servers have a 'maxconn' parameter set. It can also happen in the global queue after a redispatch consecutive to a failed attempt to connect to a dying server. If no redispatch is reported, then no connection attempt was made to any server.",
            'C' => "the proxy was waiting for the CONNECTION to establish on the server. The server might at most have noticed a connection attempt.",
            'H' => "the proxy was waiting for complete, valid response HEADERS from the server (HTTP only).",
            'D' => "the session was in the DATA phase.",
            'L' => "the proxy was still transmitting LAST data to the client while the server had already finished. This one is very rare as it can only happen when the client dies while receiving the last packets.",
            'T' => "the request was tarpitted. It has been held open with the client during the whole 'timeout tarpit' duration or until the client closed, both of which will be reported in the 'Tw' timer.",
            '-' => "normal session completion after end of data transfer.",
            _ => "Unknown session state"
        }; 
        let description = description.to_string();
        HaproxyTerminationStateEntry {
            shorthand,
            description
        }
    }

    pub fn cookie(shorthand: char) -> HaproxyTerminationStateEntry {
        let description = match shorthand {
            'N' => "the client provided NO cookie. This is usually the case for new visitors, so counting the number of occurrences of this flag in the logs generally indicate a valid trend for the site frequentation.",
            'I' => "the client provided an INVALID cookie matching no known server. This might be caused by a recent configuration change, mixed cookies between HTTP/HTTPS sites, persistence conditionally ignored, or an attack.",
            'D' => "the client provided a cookie designating a server which was DOWN, so either 'option persist' was used and the client was sent to this server, or it was not set and the client was redispatched to another server.",
            'V' => "the client provided a VALID cookie, and was sent to the associated server.",
            'E' => "the client provided a valid cookie, but with a last date which was older than what is allowed by the 'maxidle' cookie parameter, so the cookie is consider EXPIRED and is ignored. The request will be redispatched just as if there was no cookie.",
            'O' => "the client provided a valid cookie, but with a first date which was older than what is allowed by the 'maxlife' cookie parameter, so the cookie is consider too OLD and is ignored. The request will be redispatched just as if there was no cookie.",
            'U' => "a cookie was present but was not used to select the server because some other server selection mechanism was used instead (typically a 'use-server' rule).",
            '-' => "does not apply (no cookie set in configuration).",
            _ => "Unknown cookie operation"
        };
        let description = description.to_string();
        HaproxyTerminationStateEntry {
            shorthand,
            description
        }
    }

    pub fn operations(shorthand: char) -> HaproxyTerminationStateEntry {
        let description = match shorthand {
            'N' => "NO cookie was provided by the server, and none was inserted either.",
            'I' => "no cookie was provided by the server, and the proxy INSERTED one. Note that in 'cookie insert' mode, if the server provides a cookie, it will still be overwritten and reported as 'I' here.",
            'U' => "the proxy UPDATED the last date in the cookie that was presented by the client. This can only happen in insert mode with 'maxidle'. It happens every time there is activity at a different date than the date indicated in the cookie. If any other change happens, such as a redispatch, then the cookie will be marked as inserted instead.",
            'P' => "a cookie was PROVIDED by the server and transmitted as-is.",
            'R' => "the cookie provided by the server was REWRITTEN by the proxy, which happens in 'cookie rewrite' or 'cookie prefix' modes.",
            'D' => "the cookie provided by the server was DELETED by the proxy.",
            '-' => "does not apply (no cookie set in configuration).",
            _ => "Unknown cookie operation"
        };

        let description = description.to_string();
        HaproxyTerminationStateEntry {
            shorthand,
            description
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct HaproxyTerminationState {
    pub raw: String,
    pub termination_reason: HaproxyTerminationStateEntry,
    pub session_state: HaproxyTerminationStateEntry,
    pub persistence_cookie: HaproxyTerminationStateEntry,
    pub persistence_operations: HaproxyTerminationStateEntry,
}

impl HaproxyTerminationState {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let termination_reason = HaproxyTerminationStateEntry::reason(s.chars().nth(0).ok_or("")?);
        let session_state = HaproxyTerminationStateEntry::state(s.chars().nth(1).ok_or("")?);
        let persistence_cookie = HaproxyTerminationStateEntry::cookie(s.chars().nth(2).ok_or("")?);
        let persistence_operations = HaproxyTerminationStateEntry::operations(s.chars().nth(3).ok_or("")?);
        let raw = s.to_string();


        Ok(HaproxyTerminationState {
            raw,
            termination_reason,
            session_state,
            persistence_cookie,
            persistence_operations
        })
    }

    pub fn is_error(&self) -> bool {
        !(self.termination_reason.shorthand == '-' && self.session_state.shorthand == '-' && self.persistence_cookie.shorthand == '-' && self.persistence_operations.shorthand == '-')
    }
}

impl std::fmt::Display for HaproxyTerminationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}", self.termination_reason.shorthand, self.session_state.shorthand, self.persistence_cookie.shorthand, self.persistence_operations.shorthand)
    }
}

#[derive(Debug, Serialize)]
pub struct HaproxyConnectionCounts {
    pub raw: String,
    pub current: u64,
    pub limit: u64,
    pub max: u64,
    pub total: u64,
    pub rejected: u64,
}

impl HaproxyConnectionCounts {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 5 {
            return Err("Failed to parse connection counts".into());
        }

        Ok(HaproxyConnectionCounts {
            raw: s.to_string(),
            current: parts[0].parse()?,
            limit: parts[1].parse()?,
            max: parts[2].parse()?,
            total: parts[3].parse()?,
            rejected: parts[4].parse()?,
        })
    }
}

impl std::fmt::Display for HaproxyConnectionCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}/{}/{}", self.current, self.limit, self.max, self.total, self.rejected)
    }
}

#[derive(Debug, Serialize)]
pub struct HaproxyQueueStats {
    pub server: u64,
    pub backend: u64,
}

impl HaproxyQueueStats {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err("Failed to parse queue stats".into());
        }

        Ok(HaproxyQueueStats {
            server: parts[0].parse()?,
            backend: parts[1].parse()?,
        })
    }
}

impl std::fmt::Display for HaproxyQueueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.server, self.backend)
    }
}

// May  8 00:08:30 applb05 haproxy[3091252]: 127.0.0.1:6102 [08/May/2024:00:08:30.660] mclbfe silo-mclb-silo-backend/kube-prod2-node16 0/0/9/17/26 200 1005 - - ---- 823/541/29/2/0 0/0 "GET /silo/collections/1b629de5_1aaf_47d7_8b6d_5cfdcc8337e3 HTTP/1.1"
#[derive(Debug, Serialize)]
pub struct HaproxyLogEntry {
    pub month: String,
    pub day: String,
    pub time: String,
    pub host: String,
    pub process_id: String,
    pub source_ip_port: String, 
    pub time_stamp_accepted: String,
    pub frontend_name: String, 
    pub backend_name: String, 
    pub server_name: String,
    pub timers: HaproxyTimers,
    pub response_code: String,
    pub bytes_read: String,
    pub termination_state: HaproxyTerminationState,
    pub conn_counts: HaproxyConnectionCounts,
    pub queue: HaproxyQueueStats,
    pub request: String, 
}

impl HaproxyLogEntry {
    pub fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let captures = RE.captures(s).ok_or("Failed to parse line")?;
        let data = HaproxyLogEntry {
            month: captures.name("month").ok_or("")?.as_str().to_string(),
            day: captures.name("day").ok_or("")?.as_str().to_string(),
            time: captures.name("time").ok_or("")?.as_str().to_string(),
            host: captures.name("host").ok_or("")?.as_str().to_string(),
            process_id: captures.name("process_id").ok_or("")?.as_str().to_string(),
            source_ip_port: captures.name("source_ip_port").ok_or("")?.as_str().to_string(),
            time_stamp_accepted: captures.name("time_stamp_accepted").ok_or("")?.as_str().to_string(),
            frontend_name: captures.name("frontend_name").ok_or("")?.as_str().to_string(),
            backend_name: captures.name("backend_name").ok_or("")?.as_str().to_string(),
            server_name: captures.name("server_name").ok_or("")?.as_str().to_string(),
            timers: HaproxyTimers::parse(captures.name("queues_stats").ok_or("")?.as_str())?,
            response_code: captures.name("response_code").ok_or("")?.as_str().to_string(),
            bytes_read: captures.name("bytes_read").ok_or("")?.as_str().to_string(),
            termination_state: HaproxyTerminationState::parse(captures.name("termination_state").ok_or("")?.as_str())?,
            conn_counts: HaproxyConnectionCounts::parse(captures.name("conn_counts").ok_or("")?.as_str())?,
            queue: HaproxyQueueStats::parse(captures.name("queue").ok_or("")?.as_str())?,
            request: captures.name("request").ok_or("")?.as_str().to_string(),
        };

        Ok(data)
    }

    pub fn colorless(&self) -> String {
        format!("{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.month,
            self.day,
            self.time,
            self.host,
            self.process_id,
            self.source_ip_port,
            self.time_stamp_accepted,
            self.frontend_name,
            self.backend_name,
            self.server_name,
            self.timers,
            self.response_code,
            self.bytes_read,
            self.termination_state,
            self.conn_counts,
            self.queue,
            self.request
        )
    }
    pub fn colorize(&self) -> String {
        format!("{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.month.white(),
            self.day.white(),
            self.time.white(),
            self.host.white(),
            self.process_id.white(),
            self.source_ip_port.white(),
            self.time_stamp_accepted.white(),
            self.frontend_name.purple(),
            self.backend_name.yellow(),
            self.server_name.blue(),
            self.timers.to_string().white(),
            match self.response_code.as_str().parse::<u16>() {
                Ok(code) => {
                    if code >= 200 && code < 300 {
                        self.response_code.green()
                    } else if code >= 300 && code < 400 {
                        self.response_code.yellow()
                    } else if code >= 400 {
                        self.response_code.red()
                    } else {
                        self.response_code.white()
                    }
                }
                Err(_) => self.response_code.white()
            },
            self.bytes_read.white(),
            match self.termination_state.is_error() {
                false => self.termination_state.to_string().green(),
                true => self.termination_state.to_string().red()
            },
            self.conn_counts.to_string().white(),
            self.queue.to_string().white(),
            self.request.white()
        )

    }

    // Check if error code is 400 or higher, or if no ---- termination_state
    pub fn is_error(&self) -> bool {
        match self.response_code.as_str().parse::<u16>() {
            Ok(code) => code >= 400 || self.termination_state.is_error(),
            Err(_) => true
        }
    
    }


}
