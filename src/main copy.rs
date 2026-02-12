use actix_web::{App, HttpResponse, HttpServer, Responder, get, web::{self}};
use std::{fs};
use mlua::{Lua, prelude::LuaResult};

 
#[get("/{route}")]
async fn dynamic_routing(route: web::Path<String>) -> impl Responder {
    let lua = Lua::new();
    let file_name = route.into_inner();

    let script_content: String = match fs::read_to_string(format!("logic/{}.lua", &file_name)) {
        Ok(content) => return content,
        Err(..) => return String::from("print('error')"),
    };
    
    let lua_output =  match lua.load(script_content).eval() {
        Ok(content) => return content,
        Err(_) => return String::from("Error interpreting Lua, please consult the server logs."),
    };

    let data = match lua_output {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => { 
            println!("Unable to read");
            return HttpResponse::Ok().body("Error reading file.");
        },
    };

    return data
    
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //let mut entries: Vec<_> = vec![];

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
    HttpServer::new(|| App::new().service(dynamic_routing))
        .bind((address, port))?
        .run()
        .await
}

