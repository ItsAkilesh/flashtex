Generated from commit `dfd66984201acff4e2b9de2501345a3b91ec18d5` by `cargo test --test recovery generate_recovery_evidence -- --ignored --exact`.

# FlashTeX recovery evidence

Generated from the real compiler by the command above. Status, diagnostics, recovery notes, ranges, and positioned items are observed rather than handwritten.

## unmatched open brace

Input:

```text
Visible {tail.
```

Status: `recovered`

Diagnostics:

- `unmatched '{' — group never closed` — recovery: `treated the rest of the document as part of the group`; byte range: `8..9`

Positioned text items:

- `Visible` — byte range `0..7`
- `tail.` — byte range `9..14`

## stray closing brace

Input:

```text
Visible } tail.
```

Status: `recovered`

Diagnostics:

- `unmatched '}' — no group is open here` — recovery: `ignored the stray brace and continued`; byte range: `8..9`

Positioned text items:

- `Visible` — byte range `0..7`
- `tail.` — byte range `10..15`

## unterminated environment

Input:

```text
\begin{document}Visible
```

Status: `recovered`

Diagnostics:

- `unterminated environment 'document' — no matching \end` — recovery: `closed the environment at end of input`; byte range: `0..16`

Positioned text items:

- `Visible` — byte range `16..23`

## mismatched end

Input:

```text
Visible \begin{quote}body\end{itemize} Tail.
```

Status: `recovered`

Diagnostics:

- `environment 'quote' is not implemented; its body is typeset as plain text` — recovery: `typeset the body without the environment's formatting`; byte range: `8..14`
- `\end{itemize} does not match \begin{quote}` — recovery: `closed the innermost open environment`; byte range: `25..29`

Positioned text items:

- `Visible` — byte range `0..7`
- `body` — byte range `21..25`
- `Tail.` — byte range `39..44`

## stray end

Input:

```text
Visible \end{quote} Tail.
```

Status: `recovered`

Diagnostics:

- `\end{quote} with no matching \begin` — recovery: `ignored the stray \end`; byte range: `8..12`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `20..25`

## unknown command

Input:

```text
Visible \frobnicate{argument} Tail.
```

Status: `recovered`

Diagnostics:

- `\frobnicate is not supported by this compiler version; unrestricted TeX math mode is not implemented` — recovery: `skipped the command; any braced argument was typeset as plain text`; byte range: `8..19`

Positioned text items:

- `Visible` — byte range `0..7`
- `argument` — byte range `20..28`
- `Tail.` — byte range `30..35`

## include of a file the request did not supply

Input:

```text
Visible \input{chapter.tex} Tail.
```

Status: `recovered`

Diagnostics:

- `included file not found: looked for 'chapter.tex' and 'chapter.tex.tex'` — recovery: `skipped the missing include and continued`; byte range: `8..14`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `28..33`

## math command outside math mode

Input:

```text
Visible \frac{a}{b} Tail.
```

Status: `recovered`

Diagnostics:

- `\frac requires math mode` — recovery: `skipped the command and typeset its braced arguments as plain text`; byte range: `8..13`

Positioned text items:

- `Visible` — byte range `0..7`
- `a` — byte range `14..15`
- `b` — byte range `17..18`
- `Tail.` — byte range `20..25`

## unsupported preamble command

Input:

```text
\tikz \begin{document}Visible\end{document}
```

Status: `recovered`

Diagnostics:

- `\tikz is not supported in the document preamble` — recovery: `skipped the command and did not typeset preamble content`; byte range: `0..5`

Positioned text items:

- `Visible` — byte range `22..29`

## unimplemented environment

Input:

```text
Visible \begin{quote}body\end{quote} Tail.
```

Status: `recovered`

Diagnostics:

- `environment 'quote' is not implemented; its body is typeset as plain text` — recovery: `typeset the body without the environment's formatting`; byte range: `8..14`

Positioned text items:

- `Visible` — byte range `0..7`
- `body` — byte range `21..25`
- `Tail.` — byte range `37..42`

## missing required command argument

Input:

```text
Visible \textbf Tail.
```

Status: `recovered`

Diagnostics:

- `\textbf requires a braced argument` — recovery: `used an empty argument and continued`; byte range: `8..15`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `16..21`

## missing required heading argument

Input:

```text
Visible.

\section
```

Status: `recovered`

Diagnostics:

- `\section requires a braced argument` — recovery: `used an empty argument and continued`; byte range: `10..18`

Positioned text items:

- `Visible.` — byte range `0..8`

## empty required macro argument

Input:

```text
\newcommand{\echo}[1]{#1} Visible \echo{} Tail.
```

Status: `recovered`

Diagnostics:

- `macro \echo received an empty required argument` — recovery: `substituted an empty argument and continued`; byte range: `39..41`

Positioned text items:

- `Visible` — byte range `26..33`
- `Tail.` — byte range `42..47`

## required argument missing closing brace

Input:

```text
Visible \textbf{Tail
```

Status: `recovered`

Diagnostics:

- `argument to \textbf is missing its closing brace` — recovery: `closed the argument at end of input`; byte range: `15..16`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail` — byte range `16..20`

## optional argument missing closing bracket

Input:

```text
Visible \usepackage[broken
```

Status: `recovered`

Diagnostics:

- `optional argument is missing its closing ']'` — recovery: `used the text through end of input as the option`; byte range: `19..26`
- `\usepackage requires a braced argument` — recovery: `used an empty argument and continued`; byte range: `8..19`
- `\usepackage was given an empty package list` — recovery: `no packages were loaded`; byte range: `8..19`

Positioned text items:

- `Visible` — byte range `0..7`

## empty document class

Input:

```text
\documentclass{} Visible.
```

Status: `recovered`

Diagnostics:

- `\documentclass was given an empty argument` — recovery: `no document class was recorded`; byte range: `0..14`

Positioned text items:

- `Visible.` — byte range `17..25`

## empty package list

Input:

```text
\usepackage{} Visible.
```

Status: `recovered`

Diagnostics:

- `\usepackage was given an empty package list` — recovery: `no packages were loaded`; byte range: `0..13`

Positioned text items:

- `Visible.` — byte range `14..22`

## unsupported package

Input:

```text
\usepackage{tikz} Visible.
```

Status: `recovered`

Diagnostics:

- `packages tikz are recognised but not implemented` — recovery: `continued without package-specific commands or formatting`; byte range: `0..17`

Positioned text items:

- `Visible.` — byte range `18..26`

## invalid macro name

Input:

```text
Visible \newcommand{oops}{body} Tail.
```

Status: `recovered`

Diagnostics:

- `\newcommand requires a single command name as its first argument` — recovery: `ignored the invalid macro definition`; byte range: `19..25`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `32..37`

## invalid macro argument count

Input:

```text
\newcommand{\x}[10]{x} Visible.
```

Status: `recovered`

Diagnostics:

- `\newcommand argument count must be an integer from 0 to 9` — recovery: `ignored the invalid macro definition`; byte range: `15..19`

Positioned text items:

- `Visible.` — byte range `23..31`

## newcommand redefines existing command

Input:

```text
\newcommand{\section}{x} Visible.
```

Status: `recovered`

Diagnostics:

- `\newcommand cannot redefine existing command \section` — recovery: `kept the existing command definition`; byte range: `0..21`

Positioned text items:

- `Visible.` — byte range `25..33`

## renewcommand targets undefined command

Input:

```text
\renewcommand{\missing}{x} Visible.
```

Status: `recovered`

Diagnostics:

- `\renewcommand cannot redefine undefined command \missing` — recovery: `ignored the invalid redefinition`; byte range: `0..23`

Positioned text items:

- `Visible.` — byte range `27..35`

## macro recursion limit

Input:

```text
\newcommand{\loop}{\loop} Visible \loop Tail.
```

Status: `recovered`

Diagnostics:

- `macro \loop exceeded the expansion recursion limit of 64` — recovery: `stopped expanding this macro invocation`; byte range: `34..39`

Positioned text items:

- `Visible` — byte range `26..33`
- `Tail.` — byte range `40..45`

## undeclared macro replacement parameter

Input:

```text
\newcommand{\oops}{#1} Visible \oops Tail.
```

Status: `recovered`

Diagnostics:

- `macro replacement references #1 but that argument is not declared` — recovery: `omitted the unavailable argument`; byte range: `31..36`

Positioned text items:

- `Visible` — byte range `23..30`
- `Tail.` — byte range `37..42`

## stray display math close

Input:

```text
Visible \] Tail.
```

Status: `recovered`

Diagnostics:

- `stray \] has no matching \[` — recovery: `ignored the stray display-math delimiter`; byte range: `8..10`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `11..16`

## math script outside math mode

Input:

```text
Visible ^ Tail.
```

Status: `recovered`

Diagnostics:

- `math script marker used outside math mode` — recovery: `ignored the script marker and continued`; byte range: `8..9`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `10..15`

## unclosed inline math

Input:

```text
Visible $x+1
```

Status: `recovered`

Diagnostics:

- `inline math is missing its closing '$'` — recovery: `closed math mode at end of input and typeset its contents`; byte range: `8..9`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `+` — byte range `10..11`
- `1` — byte range `11..12`

## unclosed dollar display math

Input:

```text
Visible $$x+1
```

Status: `recovered`

Diagnostics:

- `display math is missing its closing delimiter` — recovery: `closed math mode at end of input and typeset its contents`; byte range: `8..9`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `10..11`
- `+` — byte range `11..12`
- `1` — byte range `12..13`

## unclosed bracket display math

Input:

```text
Visible \[x+1
```

Status: `recovered`

Diagnostics:

- `display math is missing its closing delimiter` — recovery: `closed math mode at end of input and typeset its contents`; byte range: `8..10`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `10..11`
- `+` — byte range `11..12`
- `1` — byte range `12..13`

## unmatched math closing brace

Input:

```text
Visible $a}b$ Tail.
```

Status: `recovered`

Diagnostics:

- `unmatched '}' in math mode` — recovery: `ignored the stray brace and continued`; byte range: `10..11`

Positioned text items:

- `Visible` — byte range `0..7`
- `a` — byte range `9..10`
- `b` — byte range `11..12`
- `Tail.` — byte range `14..19`

## duplicate math script

Input:

```text
Visible $x^a^b$ Tail.
```

Status: `recovered`

Diagnostics:

- `duplicate script on a math atom` — recovery: `used the last script and continued`; byte range: `12..13`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `b` — byte range `13..14`
- `Tail.` — byte range `16..21`

## unattached math script

Input:

```text
Visible $^a$ Tail.
```

Status: `recovered`

Diagnostics:

- `script marker has no preceding math atom` — recovery: `ignored the unattached script`; byte range: `9..10`

Positioned text items:

- `Visible` — byte range `0..7`
- `Tail.` — byte range `13..18`

## math group missing closing brace

Input:

```text
Visible $x^{a$ Tail.
```

Status: `recovered`

Diagnostics:

- `math group is missing its closing brace` — recovery: `closed the group at the math delimiter`; byte range: `12..13`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `a` — byte range `12..13`
- `Tail.` — byte range `15..20`

## math script missing argument

Input:

```text
Visible $x^$ Tail.
```

Status: `recovered`

Diagnostics:

- `math script is missing its argument` — recovery: `used an empty script and continued`; byte range: `10..11`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `Tail.` — byte range `13..18`

## unexpected nested math delimiter

Input:

```text
Visible $a\[b$ Tail.
```

Status: `recovered`

Diagnostics:

- `unexpected math delimiter inside math mode` — recovery: `typeset the delimiter literally and continued`; byte range: `10..12`

Positioned text items:

- `Visible` — byte range `0..7`
- `a` — byte range `9..10`
- `$` — byte range `10..12`
- `b` — byte range `12..13`
- `Tail.` — byte range `15..20`

## unknown math command

Input:

```text
Visible $x+\bogus$ Tail.
```

Status: `recovered`

Diagnostics:

- `\bogus is not supported in math mode` — recovery: `typeset the command literally and continued`; byte range: `11..17`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `+` — byte range `10..11`
- `\bogus` — byte range `11..17`
- `Tail.` — byte range `19..24`

## missing braced math argument

Input:

```text
Visible $x+\frac a{b}$ Tail.
```

Status: `recovered`

Diagnostics:

- `\frac requires a braced math argument` — recovery: `used an empty argument and continued`; byte range: `11..16`
- `\frac requires a braced math argument` — recovery: `used an empty argument and continued`; byte range: `11..16`

Positioned text items:

- `Visible` — byte range `0..7`
- `x` — byte range `9..10`
- `+` — byte range `10..11`
- `─` — byte range `11..16`
- `a` — byte range `17..18`
- `b` — byte range `19..20`
- `Tail.` — byte range `23..28`

## include naming a document the request did not supply

Input:

```text
{"id":"recovery","payload":{"documents":[{"path":"main.tex","text":"Visible main document. \\input{absent}"},{"path":"chapter.tex","text":"Other."}],"entry_path":"main.tex","project_id":"recovery-evidence","revision":1},"protocol_version":1,"type":"compile"}
```

Status: `recovered`

Diagnostics:

- `included file not found: looked for 'absent' and 'absent.tex'` — recovery: `skipped the missing include and continued`; byte range: `23..29`

Positioned text items:

- `Visible` — byte range `0..7`
- `main` — byte range `8..12`
- `document.` — byte range `13..22`

## absolute document path

Input:

```text
{"id":"recovery","payload":{"documents":[{"path":"/absolute.tex","text":"Visible."}],"entry_path":"/absolute.tex","project_id":"recovery-evidence","revision":1},"protocol_version":1,"type":"compile"}
```

Status: `failed`

Diagnostics:

- `rejected document path '/absolute.tex': paths must be project-relative with no parent traversal` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## parent traversal entry path

Input:

```text
{"id":"recovery","payload":{"documents":[{"path":"main.tex","text":"Visible."}],"entry_path":"../outside.tex","project_id":"recovery-evidence","revision":1},"protocol_version":1,"type":"compile"}
```

Status: `failed`

Diagnostics:

- `rejected entry_path '../outside.tex': paths must be project-relative with no parent traversal` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## no documents

Input:

```text
{"id":"recovery","payload":{"documents":[],"entry_path":"main.tex","project_id":"recovery-evidence","revision":1},"protocol_version":1,"type":"compile"}
```

Status: `failed`

Diagnostics:

- `no documents supplied to compile` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## malformed JSON request

Input:

```text
{not json
```

Status: `error`

Diagnostics:

- `expected '"' at position 1` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## unknown protocol version

Input:

```text
{"protocol_version":99,"id":"bad-version","type":"compile","payload":{}}
```

Status: `error`

Diagnostics:

- `protocol version 99 is not supported; this build speaks version 1` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## missing protocol version

Input:

```text
{"id":"missing-version","type":"compile","payload":{}}
```

Status: `error`

Diagnostics:

- `protocol_version is required` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## unknown message type

Input:

```text
{"protocol_version":1,"id":"bad-type","type":"mystery"}
```

Status: `error`

Diagnostics:

- `message type 'mystery' is not supported` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## missing message type

Input:

```text
{"protocol_version":1,"id":"missing-type"}
```

Status: `error`

Diagnostics:

- `type is required` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## missing compile payload

Input:

```text
{"protocol_version":1,"id":"missing-payload","type":"compile"}
```

Status: `error`

Diagnostics:

- `compile requires a payload` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## oversized request line

Input:

```text
<exactly 8388609 ASCII 'x' bytes>
```

Status: `error`

Diagnostics:

- `line exceeds the 8388608-byte limit` — recovery: `null`; byte range: `null`

Positioned text items:

- none

## invalid UTF-8 request line

Input:

```text
<hex ff>
```

Status: `error`

Diagnostics:

- `request line is not valid UTF-8` — recovery: `null`; byte range: `null`

Positioned text items:

- none

