use actix_web::{App, HttpResponse, HttpServer, Responder, get, web::{self}};
use std::{fs};
use mlua::{Lua, prelude::LuaResult};

fn dynamic_routing_lua(route: &str) -> HttpResponse {
    let lua = Lua::new();
    let file_name = if route.is_empty() { String::from("index") } else { String::from(route) };
    if let Ok(script_content) = fs::read_to_string(format!("logic/{}.lua", &file_name)) {
        let lua_output =  match lua.load(script_content).eval::<_>() {
            Ok(content) => content,
            Err(_) => String::from("Error executing Lua, please check server logs."),
        };
        
        return HttpResponse::Ok().body(lua_output);

    } else {
        return HttpResponse::Ok().body("Unexistent route.")
    }

}

#[get("/{route}")]
async fn dynamic_routing(route: web::Path<String>) -> impl Responder {
    return dynamic_routing_lua(&route.into_inner());

}

#[get("/")]
async fn index() -> impl Responder {
    return dynamic_routing_lua("index");

}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

   if let Ok(routes) = fs::read_dir("logic") {
    println!("Available routes:");
    for e in routes.flatten() {
        if let Some(name) = e.file_name().to_str() {
            println!("-> : {}", name);
        }
    }
    println!("Wasted time on this shit and it is just for debugging ffs.\n")
   }
    //println!("Routes detected: {:?}", entries);
    let address = "127.0.0.1";
    let port = 8080; 
    println!("Server starting on: {}:{}", &address, &port);
    HttpServer::new(|| App::new().service(dynamic_routing).service(index))
        .bind((address, port))?
        .run()
        .await
}

