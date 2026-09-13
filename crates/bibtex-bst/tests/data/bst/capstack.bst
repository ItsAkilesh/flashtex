% Exercises the [307] lit_stack/lit_stk_type reallocation: pushes 100
% integers onto the literal stack without popping them all off before the
% entry finishes (initial LIT_STK_SIZE=50, +50).
ENTRY { } { } { }
FUNCTION {article} { skip$ }
FUNCTION {misc} { skip$ }
FUNCTION {p1} { #1 #1 #1 #1 #1 #1 #1 #1 #1 #1 }
FUNCTION {p10} { p1 p1 p1 p1 p1 p1 p1 p1 p1 p1 }
FUNCTION {doit} { p10 }
READ
ITERATE {doit}
