% builtins.bst -- original FlashTeX test style exercising every BibTeX
% built-in function, including their error paths. Not for real documents.

ENTRY
  { author title editor year key crossref note journal booktitle }
  { num }
  { label extra }

INTEGERS { i j k }
STRINGS { gs ht }

MACRO {jan} {"January"}

FUNCTION {not} { { #0 } { #1 } if$ }

FUNCTION {show} { write$ newline$ }

FUNCTION {showint} { int.to.str$ show }

FUNCTION {names}
{ duplicate$ empty$
    { pop$ "(no names)" show }
    { 'ht :=
      ht num.names$ 'j :=
      "count " j int.to.str$ * show
      #1 'i :=
        { i j > not }
        { ht i "{ff~}{vv~}{ll}{, jj}" format.name$ show
          ht i "{vvv }{ll}{, f.}{, jj}" format.name$ show
          ht i "{f.~}{v.~}{l}" format.name$ show
          ht i "{L}{{/}}{F{-}}{J{~}}" format.name$ show
          ht i "{ll{ }}{ ff{ }}" format.name$ show
          i #1 + 'i :=
        }
      while$
    }
  if$
}

FUNCTION {cases}
{ duplicate$ "t" change.case$ show
  duplicate$ "l" change.case$ show
  duplicate$ "u" change.case$ show
  duplicate$ purify$ show
  duplicate$ width$ showint
  duplicate$ text.length$ showint
  duplicate$ #3 text.prefix$ show
  duplicate$ #2 #5 substring$ show
  duplicate$ #-1 #4 substring$ show
  duplicate$ add.period$ show
  pop$
}

FUNCTION {entry.body}
{ newline$
  "cite " cite$ * " type " * type$ * show
  author names
  editor names
  title empty$ 'skip$ { title cases } if$
  journal missing$ { "journal missing" show } { journal show } if$
  crossref empty$ 'skip$ { "crossref=" crossref * show } if$
  cite$ purify$ "l" change.case$ 'label :=
  label show
  num #1 + 'num :=
  num showint
}

FUNCTION {article} { entry.body }
FUNCTION {book} { entry.body }
FUNCTION {inproceedings} { entry.body }
FUNCTION {default.type} { "default: " type$ * show entry.body }

READ

FUNCTION {sortify}
{ title empty$
    { cite$ }
    { title purify$ "l" change.case$ }
  if$
  'sort.key$ :=
}

ITERATE {sortify}
SORT

FUNCTION {begin}
{ preamble$ show
  "entry.max$=" entry.max$ int.to.str$ * show
  "global.max$=" global.max$ int.to.str$ * show
}

EXECUTE {begin}
ITERATE {call.type$}
REVERSE {sortify}

FUNCTION {edge.periods}
{ "A}" add.period$ show
  "}}" add.period$ show
  "Hi!}" add.period$ show
  "" add.period$ show
  "Why?" add.period$ show
}

FUNCTION {edge.chars}
{ #65 int.to.chr$ show
  #200 int.to.chr$ show
  "AB" chr.to.int$ showint
  "A" chr.to.int$ showint
  #-42 int.to.str$ show
  #0 int.to.str$ show
  quote$ show
  "x" "y" swap$ * show
}

FUNCTION {edge.substrings}
{ "abc" #5 #1 substring$ show
  "abc" #0 #1 substring$ show
  "abcdef" #-2 #3 substring$ show
  "abcdef" #2 #-1 substring$ show
  "abcdef" #9 #-7 substring$ show
  "abcdef" #3 #4 substring$ show
  "{\ss}tra{\ss}e {\"o}l" duplicate$ width$ showint
  duplicate$ text.length$ showint
  duplicate$ #3 text.prefix$ show
  duplicate$ #0 text.prefix$ show
  purify$ show
  "{{abc}} {\relax x}" duplicate$ #2 text.prefix$ show
  purify$ show
}

FUNCTION {edge.case}
{ "{\AE}sop {\oe}uvre {\i}ll {\ss} {\l}ukasz {\o}" duplicate$ "u" change.case$ show
  duplicate$ "l" change.case$ show
  "t" change.case$ show
  "Title: {A} Sub {\relax x}: after:   Spaces {\em Emph} WORD" duplicate$ "t" change.case$ show
  "T" change.case$ show
  "x" "tt" change.case$ show
  "{unbalanced" "u" change.case$ show
  "unbal}" purify$ show
  "{\AA}ngstr{\"o}m {\j}" duplicate$ width$ showint
  purify$ show
  "wide {\ae} {\OE} {\AE} {\oe} {\ss} {\em x} {\'e} \relax" width$ showint
}

FUNCTION {edge.errors}
{ #1 "a" + showint
  "a" #1 - showint
  pop$
  "s" 'gs := gs show
  "a" "b" * 'gs := gs show
  #5 'i := i showint
  "q" 'title :=
  "z" 'label :=
  title
  "a" #1 =
  'skip$ 'skip$ =
  #1 #1 = showint
  "same" "same" = showint
  #1 'skip$ 'skip$ if$
  "a" 'skip$ 'skip$ if$
  #3 #2 > showint
  #3 #2 < showint
  "a" #2 #3 while$
  "w" warning$
  #7 warning$
  cite$
  type$
  missing$
  #1 empty$ showint
  "   " empty$ showint
  #1 #2 stack$
  "top" top$
  'skip$ top$
  #3 top$
  top$
  "leftover"
  #9
}

FUNCTION {edge.names}
{ "Jean-Paul {\'E}mile de La Fontaine and others" #2 "{ll}" format.name$ show
  "Jean-Paul {\'E}mile de La Fontaine and others" #3 "{ll}" format.name$ show
  "Jean-Paul {\'E}mile de La Fontaine and others" #1 "{ff}{x}" format.name$ show
  "Solo" #2 "{ll}" format.name$ show
  "Trailing, Comma," #1 "{ll}/{ff}" format.name$ show
  "A, B, C, D" #1 "{ll}/{jj}/{ff}" format.name$ show
  "Bad } Brace" #1 "{ll}" format.name$ show
  "de la {\'E}toile, Jr., Jean {\relax Ch}arles" #1 "{vv~}{ll}{, jj}{, ff}" format.name$ show
  "de la {\'E}toile, Jr., Jean {\relax Ch}arles" #1 "{f.}{ v.}{ l.}{ j.}" format.name$ show
  "{\v{S}}imon {\"{U}}ber-{M}ann van~der~Waals" #1 "{f.~}{vv~}{ll}" format.name$ show
  "Ab Cd Ef Gh" #1 "{ff }{ll}" format.name$ show
  "Aa Bbbbbb Cc" #1 "{ff~}{ll}" format.name$ show
  "Aa B. Cc" #1 "{f~}{ll}" format.name$ show
  "x" #1 "{ll" format.name$ show
  "x" #1 "}{ll}" format.name$ show
  "Name" #1 #5 format.name$ show
  "Name" "a" "b" format.name$ show
  "and and and" num.names$ showint
  "A and B AND C and{D} and {E and F}" num.names$ showint
  "" num.names$ showint
}

FUNCTION {edge.output}
{ "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" show
  "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cc   dd" show
  "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen" show
  "ab cdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijkl mnop" show
  "   " show
  newline$
  "no newline at the end of a write" write$
}

EXECUTE {edge.periods}
EXECUTE {edge.chars}
EXECUTE {edge.substrings}
EXECUTE {edge.case}
EXECUTE {edge.errors}
EXECUTE {edge.names}
EXECUTE {edge.output}
