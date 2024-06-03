use rocket::{fairing::{Fairing, Info, Kind}, Build, Config, Data, Orbit, Request, Response, Rocket};

fn url_from_rocket_config(config: &Config) -> String {
    format!(
        "{scheme}://{address}:{port}",
        scheme = if config.tls_enabled() {
            "https"
        } else {
            "http"
        },
        address = &config.address,
        port = &config.port
    )
}

#[derive(Clone)]
pub struct Slogger {}

#[rocket::async_trait]
impl Fairing for Slogger {
    fn info(&self) -> Info {
        Info {
            name: "Slog Fairing",
            kind: Kind::Ignite | Kind::Liftoff | Kind::Request | Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        Ok(rocket.manage(self.clone()))
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let config = rocket.config();

        let url = url_from_rocket_config(config);
        log::info!(
            "Rocket Launched {}",
            serde_json::json!({
                "log_level": config.log_level,
                "ident": config.ident,
                "tls": config.tls_enabled(),
                "limits": config.limits,
                "keep_alive": config.keep_alive,
                "workers": config.workers,
                "port": config.port,
                "host": config.address,
                "profile": config.profile,
            }));

        for route in rocket.routes() {
            log::info!(
                "Route Registered {}",
                serde_json::json!({
                    "rank": route.rank,
                    "route": route.name.as_ref().map(|route| route.to_string()),
                    "content-type": route.format.as_ref().map(|format| format.to_string()),
                //    "path": route.uri,
                    "url": format!("{}{}", url, route.uri),
                    "method": route.method,
                }));
        }

        for catcher in rocket.catchers() {
            log::info!(
                "Catcher Registered {}",
                serde_json::json!({
                    "route": catcher.name.as_ref().map(|catcher| catcher.to_string()),
                    "code": catcher.code,
                    "path": catcher.base,
                    "url":format!("{}{}", url, catcher.base),
                }));
        }

        log::info!(
            "Accepting Connections {}",
            serde_json::json!({
                "port": config.port,
                "host": config.address,
                "url": url,
            }));
    }

    async fn on_request(&self, _request: &mut Request<'_>, _: &mut Data<'_>) {
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let user_agent = request
            .headers()
            .get("user-agent")
            .collect::<Vec<_>>()
            .join("; ");
        let remote_socket_addr = request.remote().unwrap();
        log::info!(
            "Reqest {}",
            serde_json::json!({
                "user_adent": user_agent,
                "source_ip": remote_socket_addr,
                "status": response.status()
           }));
    }
}
