% badstyle.bst -- original FlashTeX test style full of deliberate syntax
% errors, to compare BibTeX's error messages and recovery. Not for use.

ENTRY { title author } {} {}

FUNCTION {show} { write$ newline$ }

FUNCTION {a} { "x" show undefinedfn show #12x show }

FUNCTION {b} { #- show "ok" show }

FUNCTION {c} { "unterminated show }

FUNCTION {rec} { 'rec rec { rec } show }

FUNCTION {quoted} { 'nosuch 'show }

BOGUS {x}

ENTRY { another } {} {}

execute {a}

READ

EXECUTE {nosuchfn}

EXECUTE {title}

MACRO {m} {"late"}

FUNCTION {show} { pop$ }

INTEGERS { i i }

STRINGS { str1 str2 }

FUNCTION {emit} { "emitted" show title show }

ITERATE {emit}
ITERATE {rec}
EXECUTE {b}
EXECUTE {quoted}
EXECUTE {a}
REVERSE {
  emit
}
SORT
READ
iterate emit
FUNCTION {unfinished} { "never closed"
