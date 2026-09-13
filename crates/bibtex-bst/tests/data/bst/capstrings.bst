% Exercises the [216] glb_str_ptr/global_strs/glb_str_end reallocation:
% declares 15 global string variables (initial MAX_GLOB_STRS=10, +10).
STRINGS { gv0 gv1 gv2 gv3 gv4 gv5 gv6 gv7 gv8 gv9 gv10 gv11 gv12 gv13 gv14 }
ENTRY { } { } { }
FUNCTION {article} { skip$ }
FUNCTION {misc} { skip$ }
FUNCTION {doit} { "x" 'gv0 := }
READ
ITERATE {doit}
