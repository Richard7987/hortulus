#!/bin/sh
# usage: tools/compare.sh <case name> <spec> <settings>
name=$1; spec=$2; set=$3
L=reference/lua
texlua tools/oracle.lua $L "$spec" "$set" build/$name.lua >build/$name.lua.log 2>&1 || { echo "LUA FALLO: $(tail -3 build/$name.lua.log)"; exit 1; }
./plugin/target/release/native "$spec" "$set" build/$name.rs >build/$name.rs.log 2>&1 || { echo "RUST FALLO: $(tail -3 build/$name.rs.log)"; exit 1; }
b=DIFF; r=DIFF
cmp -s build/$name.lua.body build/$name.rs.body && b=igual
cmp -s build/$name.lua.res build/$name.rs.res && r=igual
printf '%-28s cuerpo=%s recursos=%s | %s | %s\n' "$name" $b $r "$(cat build/$name.lua.log | tail -1)" "$(cat build/$name.rs.log | tail -1)"
