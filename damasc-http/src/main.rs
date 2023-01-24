use std::env;
use std::sync::Arc;
use std::{collections::BTreeSet, sync::Mutex};

use actix_files::Files;
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use damasc_lang::identifier::Identifier;
use damasc_repl::parser;
use damasc_repl::state::State;

use serde::Deserialize;

#[derive(Deserialize)]
struct ReplInput {
    statement: String,
}

#[derive(Template)]
#[template(path = "404.html.j2")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "result.html.j2")]
struct ResultTemplate<'x> {
    repl: &'x ReplInput,
    error: Option<String>,
    output: Option<String>,
    vars: BTreeSet<&'x Identifier<'x>>,
    examples: &'static [&'static str],
}

#[derive(Template)]
#[template(path = "index.html.j2")]
struct HomeTemplate<'x> {
    repl: &'x ReplInput,
    examples: &'static [&'static str],
}

const EXAMPLES: [&str; 10] = [
    r#"true"#,
    r#"true && false"#,
    r#"5*5"#,
    r#"[1,2,3]"#,
    r#"{x:32,y:42}"#,
    r#""hello""#,
    r#"[_,m,_]=[1,2,3]"#,
    r#"t=type(x);{x:[_ is Integer, x]}={x:[23,true]}"#,
    r#"let [x,y] = [23,42]"#,
    r#"{ 3;4;4;2 } |> map x;y where x!=y into [x,y, x*y]"#,
];

#[post("/")]
async fn eval(
    repl: web::Form<ReplInput>,
    env_mutex: Data<Arc<Mutex<State<'_, '_, '_>>>>,
) -> impl Responder {
    let Ok(mut repl_state) = env_mutex.lock() else {
        return HttpResponse::Ok().content_type("text/html").body("Locked");
    };

    let vars = repl_state.vars();

    if repl.statement.len() > 500 {
        return HttpResponse::Ok().content_type("text/html").body(
            ResultTemplate {
                error: Some("Input length is limited to 500 characters".to_string()),
                repl: &repl,
                output: None,
                vars,
                examples: &EXAMPLES,
            }
            .render()
            .unwrap(),
        );
    }

    match parser::command_all_consuming(&repl.statement) {
        Ok(stmt) => {
            let (output, error) = match repl_state.eval(stmt) {
                Ok(r) => (Some(format!("{r}")), None),
                Err(e) => (None, Some(format!("{e:?}"))),
            };

            let vars = repl_state.vars();

            ResultTemplate {
                error,
                repl: &repl,
                output,
                vars,
                examples: &EXAMPLES,
            }
        }

        Err(e) => ResultTemplate {
            error: Some(e),
            repl: &repl,
            output: None,
            vars,
            examples: &EXAMPLES,
        },
    }
    .render()
    .map(|s| HttpResponse::Ok().content_type("text/html").body(s))
    .unwrap_or_else(template_error)
}

fn template_error(_: askama::Error) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("text/html")
        .body("Template Error")
}

#[get("/")]
async fn home() -> impl Responder {
    HomeTemplate {
        repl: &ReplInput {
            statement: "".to_owned(),
        },
        examples: &EXAMPLES,
    }
    .render()
    .map(|s| HttpResponse::Ok().content_type("text/html").body(s))
    .unwrap_or_else(template_error)
}

async fn not_found() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body(NotFoundTemplate {}.render().unwrap())
}

#[derive(Deserialize, Debug)]
struct Configuration {
    ip: String,
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let repl = State::new();
    let repl_mutex = Arc::new(Mutex::new(repl));
    let repl_mutex_data = Data::new(repl_mutex.clone());

    let conf = Configuration {
        ip: env::var("DAMASC_HOST").unwrap_or("127.0.0.1".into()),
        port: env::var("DAMASC_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8080),
    };

    let server = HttpServer::new(move || {
        App::new()
            .app_data(repl_mutex_data.clone())
            .service(home)
            .service(eval)
            .service(Files::new("/", "./public/"))
            .default_service(web::route().to(not_found))
    })
    .bind((conf.ip, conf.port))?;

    println!("Server started");
    for (adr, scheme) in server.addrs_with_scheme() {
        println!("Listening on {scheme}://{adr}");
    }

    server.run().await
}
