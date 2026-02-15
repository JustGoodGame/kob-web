hello = "Welcome to "
heythere = "KobWeb Lua!"

local cjson = require("cjson")
local yaya = require("yaya")
local json = cjson.encode({
    foo = "bar",
    some_object = {},
    some_array = cjson.empty_array
})

--print(type(params[1]), type(params[2]))
--[[print(json)
print(request.method)]]
--return hello .. heythere .. "All of this is being ran on Lua using mlua as we speak kekww.\n" .. "Today it is \n" .. os.date() .. "\n" .. here you have 
yo = function ()
    local parts = {}
    for k, v in pairs(query_params) do
        table.insert(parts, k .. "=" .. v)
    end
    return table.concat(parts, ", ")
end
print(string.format("path_params: [%s]\nquery_params: [%s]", table.concat(path_params, ", "), yo()))
return string.format(
[[
%s %s, All of this being ran on Lua using mlua as we speak kekww 
Today it is: %s
And here you have the output of the rock CJSON being used on Kob-Web!:
%s
path_params: %s
query_params: %s
Request-Type: %s
]], hello,  heythere, os.date(), json, table.concat(path_params, ", "), yo(), request.method)
