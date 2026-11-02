-- Paints a page with the reference Lua generator, writing what the Rust port must match.
-- usage: texlua tools/oracle.lua <lua dir> <spec file> <settings> <out prefix>
local dir, specf, setting, out = arg[1], arg[2], arg[3], arg[4]
table.insert(package.searchers, 2, function(name)
  local chunk = loadfile(dir .. "/" .. name:gsub("%.", "/") .. ".lua")
  return chunk or "\n\tno file"
end)
local f = assert(io.open(specf)); local txt = f:read("a"); f:close()
local n = {}
for w in txt:gmatch("%S+") do n[#n + 1] = tonumber(w) or (0/0) end
local set = require("flora.settings").parse(setting)
local lines = {}
for i = 13, #n, 4 do lines[#lines + 1] = { x0 = n[i], y0 = n[i + 1], x1 = n[i + 2], y1 = n[i + 3] } end
local page = math.floor(n[8])
local base = tonumber(set.seed or 1) or 1
local nan = function(v) return v ~= v end
local spec = {
  width = n[1], height = n[2], text = { x0 = n[3], y0 = n[4], x1 = n[5], y1 = n[6] },
  header_y = not nan(n[7]) and n[7] or nil, lines = lines,
  seed = base * 7919 + page * 104729, page = page, layout = set.layout, mix = set.mix, ornaments = set.ornaments,
  palette = set.palette, spin = set.spin, gilt = set.gilt, gilt_dpi = set.giltdpi, gilt_env = set.giltenv,
  gilt_look = set.giltlook, doc_seed = tostring(base),
  title = not nan(n[9]) and { x0 = n[9], y0 = n[10], x1 = n[11], y1 = n[12] } or nil,
}
local Paint = require("flora.paint")
Paint.reset_resources()
local t = os.clock()
local pt = require("flora.compose").page(spec)
local dt = os.clock() - t
local o = assert(io.open(out .. ".body", "wb")); o:write(pt:body()); o:close()
local res = Paint.resources()
o = assert(io.open(out .. ".res", "wb"))
for _, g in ipairs(res.gs) do o:write("gs ", g[1], " ", g[2], "\n") end
for _, p in ipairs(res.patterns) do o:write("pat ", p[1], " ", p[2], "\n") end
local function sum(s) local v = 0 for i = 1, #s do v = (v * 31 + s:byte(i)) % 1000000007 end return v end
for _, x in ipairs(res.xobjects or {}) do
  o:write(string.format("img %s %d %d %d %d\n", x[1], x.image.w, x.image.h, sum(x.image.rgb), sum(x.image.alpha)))
end
o:close()
print(string.format("lua %.3fs, %d objetos, %d bytes", dt, pt.count, #pt:body()))
