use mlua::{Lua};
use std::{collections::HashMap};
use std::fs::{self, read_to_string};
use actix_web::{HttpRequest, HttpResponse, web};

use crate::utils::tomlparser::server;

fn fetch_script(route: &str) -> (Option<String>, Vec<String>, String) {
    let parts: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
    
    // Always start by checking if index.lua exists (handles both "/" and "/something")
    let index_paths = vec![
        "logic/index.lua".to_string(),
        "logic/index/index.lua".to_string(),
    ];
    
    let mut deepest_content: Option<String> = index_paths.iter()
        .find_map(|path| fs::read_to_string(path).ok());
    let mut deepest_index = -1i32;  // -1 means we're at index level
    
    // If route is empty, we're done
    if parts.is_empty() {
        return (deepest_content, vec![], String::from("/"));
    }
    
    // Try to drill deeper from index
    let mut current_path = String::new();
    let mut deepest_route = String::from("/");  // Track the matched route

    for (i, part) in parts.iter().enumerate() {
        // Build path progressively: "users" -> "users/test" -> "users/test/45"
        if current_path.is_empty() {
            current_path = part.to_string();
        } else {
            current_path = format!("{}/{}", current_path, part);
        }
        
        // Try both file patterns at this level
        let possible_paths = vec![
            format!("logic/{}.lua", current_path),
            format!("logic/{}/index.lua", current_path),
        ];
        
        // Check if file exists at this level
        let found = possible_paths.iter()
            .find_map(|path| fs::read_to_string(path).ok());
        
        if let Some(content) = found {
            deepest_content = Some(content);
            deepest_index = i as i32;
            deepest_route = format!("/{}", current_path.clone());

        }
    }
    
    // Calculate remaining params (everything after the deepest file found)
    let remaining_params: Vec<String> = if deepest_index == -1 {
        // We only found index.lua, so all parts are params
        parts.iter().map(|s| s.to_string()).collect()
    } else {
        // Return everything after the deepest file
        parts[deepest_index as usize + 1..].iter().map(|s| s.to_string()).collect()
    };

    (deepest_content, remaining_params, deepest_route)

}

fn matches_pattern(route: &str, pattern: &str) -> bool {
    if pattern.ends_with("/*") {
        let prefix = pattern.trim_end_matches("/*");
        route.starts_with(prefix) 
    } else {
        route == pattern
    }
}

pub fn dynamic_routing_lua(req: HttpRequest, route: &str) -> HttpResponse {    
    // Get system's lua paths to import already setup lua libraries
    let lua_path = std::env::var("LUA_PATH").unwrap_or_default();
    let lua_cpath = std::env::var("LUA_CPATH").unwrap_or_default();
    
    let lua = unsafe { Lua::unsafe_new() };
    lua.load(&format!(
        r#"
        package.path = package.path .. ";{}" .. "./logic/?.lua;"
        package.cpath = package.cpath .. ";{}"
        "#,
        lua_path, lua_cpath
    )).exec();
    
    // Create and Pass request information onto the LuaVM
    let requestinfo
     = HashMap::from([
      ("method", req.method().to_string()),
      ("route", req.uri().to_string()),
      ("clientip", req.peer_addr().expect("No ip").to_string()),
      //("httpversion", req.version().to_string()),
    ]);

    let mut luareqinfo = lua.create_table_from(requestinfo).expect("Couldn't fetch request info");
    lua.globals().set("request", luareqinfo);
    
    // Fetch server's current logic
    let script_content = fetch_script(route);
    //println!("route:'{}' - scr_conten2:'{}'", route, &script_content.2);

    // Fetch and pass query parameters to the Lua VM under the variable "query_params"
    let mut qphashmap: HashMap<_, _>  = HashMap::new();
    let query_params = String::from(req.query_string());
    for pair in query_params.split("&").collect::<Vec<_>>() {
    
        let a = pair.split_once("=");
        if let Some((k, v)) = a {
            qphashmap.insert(k, v);
        }
    }
    let qpluatable = lua.create_table_from(qphashmap).unwrap();
    lua.globals().set("query_params", qpluatable);

    // Fetch and pass Path Parameters to the Lua VM under the variable "path_params"
    if server().routing.allow_path_params.iter().any(|x| matches_pattern(&script_content.2, x)) {
        //println!("ALLOW PARAMS FOR: '{}'", script_content.2);
        let path_params = lua.create_sequence_from(script_content.1.iter().map(|x| x.as_str())).unwrap();
        lua.globals().set("path_params", path_params);

    } else if !script_content.1.is_empty() {
        return HttpResponse::Ok().body("Unexistent route.") 
    }
    
    if let Some(content) = script_content.0 {
        let lua_output =  match lua.load(content).eval::<Option<String>>() {
            Ok(Some(content)) => content,
            Ok(None) => String::from("Unexistent route."),
            Err(e) =>  {
                 println!("Lua error: {}", e);
                 String::from("Error executing Lua, please check server logs.")
            },
        };
        
    return HttpResponse::Ok().body(lua_output);

    } else {
        return HttpResponse::Ok().body("Unexistent route.")
    }
    

}
