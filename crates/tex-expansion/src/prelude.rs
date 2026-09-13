//! The LaTeX-kernel prelude: macros that real LaTeX (and plain TeX)
//! define *in TeX*, reproduced here verbatim or near-verbatim from
//! `latex.ltx`/`plain.tex` so that their expansion behaviour (including
//! `\meaning`, `\ifx` comparisons and `\protect` interaction) matches the
//! real thing rather than a Rust approximation. Executed once when the
//! first engine is created (see `expand.rs::build_initial_state`); every
//! engine starts from the resulting state.
//!
//! Only expansion-relevant kernel macros are here. Typesetting commands
//! (`\section`, `\textbf`, ...) are deliberately absent: they pass
//! through to the typesetter untouched (see CONTRACT.md).

pub const PRELUDE: &str = r"\catcode`\@=11
\def\makeatletter{\catcode`\@=11 }
\def\makeatother{\catcode`\@=12 }
\def\@empty{}
\def\space{ }
\chardef\@ne=1
\chardef\tw@=2
\chardef\thr@@=3
\chardef\sixt@@n=16
\chardef\@cclv=255
\mathchardef\@cclvi=256
\mathchardef\@m=1000
\mathchardef\@M=10000
\mathchardef\@MM=20000
\newcount\m@ne \m@ne=-1
\newcount\count@
\newdimen\dimen@
\newdimen\z@ \z@=0pt
\newskip\skip@
\long\def\@gobblefour#1#2#3#4{}
\long\def\@car#1#2\@nil{#1}
\long\def\@cdr#1#2\@nil{#2}
\long\def\@onlypreamble#1{}
\long\def\@gobble#1{}
\long\def\@gobbletwo#1#2{}
\long\def\@firstofone#1{#1}
\long\def\@firstoftwo#1#2{#1}
\long\def\@secondoftwo#1#2{#2}
\def\@nnil{\@nil}
\let\@@par\par
\let\endgraf\par
\let\bgroup={
\let\egroup=}
\long\def\@namedef#1{\expandafter\def\csname #1\endcsname}
\def\@nameuse#1{\csname #1\endcsname}
\def\@ifundefined#1{%
  \ifcsname#1\endcsname
    \expandafter\ifx\csname#1\endcsname\relax
      \expandafter\expandafter\expandafter\@firstoftwo
    \else
      \expandafter\expandafter\expandafter\@secondoftwo
    \fi
  \else
    \expandafter\@firstoftwo
  \fi}
\long\def\@ifnextchar#1#2#3{%
  \let\reserved@d=#1%
  \def\reserved@a{#2}%
  \def\reserved@b{#3}%
  \futurelet\@let@token\@ifnch}
\def\@ifnch{%
  \ifx\@let@token\@sptoken
    \let\reserved@c\@xifnch
  \else
    \ifx\@let@token\reserved@d
      \let\reserved@c\reserved@a
    \else
      \let\reserved@c\reserved@b
    \fi
  \fi
  \reserved@c}
\def\:{\let\@sptoken= } \: %
\def\:{\@xifnch} \expandafter\def\: {\futurelet\@let@token\@ifnch}
\let\kernel@ifnextchar\@ifnextchar
\long\def\@testopt#1#2{\kernel@ifnextchar[{#1}{#1[{#2}]}}
\def\@protected@testopt#1{\ifx\protect\@typeset@protect\expandafter\@testopt\else\@x@protect#1\fi}
\long\def\@x@protect#1\fi#2#3{\fi\protect#1}
\def\@ifstar#1{\@ifnextchar *{\@firstoftwo{#1}}}
\long\def\loop#1\repeat{%
  \def\iterate{#1\relax\expandafter\iterate\fi}%
  \iterate
  \let\iterate\relax}
\let\repeat\fi
\toksdef\toks@=0
\long\def\g@addto@macro#1#2{%
  \begingroup
    \toks@\expandafter{#1#2}%
    \xdef#1{\the\toks@}%
  \endgroup}
\def\@begindocumenthook{}
\def\@enddocumenthook{}
\long\def\AtBeginDocument#1{\g@addto@macro\@begindocumenthook{#1}}
\long\def\AtEndDocument#1{\g@addto@macro\@enddocumenthook{#1}}
\let\@typeset@protect\relax
\let\protect\@typeset@protect
\def\@unexpandable@protect{\noexpand\protect\noexpand}
\def\set@display@protect{\let\protect\string}
\def\protected@edef{%
  \let\@@protect\protect
  \let\protect\@unexpandable@protect
  \afterassignment\restore@protect
  \edef}
\def\protected@xdef{%
  \let\@@protect\protect
  \let\protect\@unexpandable@protect
  \afterassignment\restore@protect
  \xdef}
\def\restore@protect{\let\protect\@@protect}
\def\@currentlabel{}
\def\@currenvir{document}
\def\p@{}
\long\def\@for#1:=#2\do#3{%
  \expandafter\def\expandafter\@fortmp\expandafter{#2}%
  \ifx\@fortmp\@empty \else
    \expandafter\@forloop#2,\@nil,\@nil\@@#1{#3}\fi}
\long\def\@forloop#1,#2,#3\@@#4#5{\def#4{#1}\ifx #4\@nnil \else
       #5\def#4{#2}\ifx #4\@nnil \else#5\@iforloop #3\@@#4{#5}\fi\fi}
\long\def\@iforloop#1,#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
       \expandafter\@fornoop \else
      #4\relax\expandafter\@iforloop\fi#2\@@#3{#4}}
\long\def\@fornoop#1\@@#2#3{}
\long\def\@tfor#1:=#2\do#3{%
  \edef\@fortmp{\unexpanded{#2}}%
  \ifx\@fortmp\@empty \else
    \@tforloop#2\@nil\@nil\@@#1{#3}\fi}
\long\def\@tforloop#1#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
       \expandafter\@fornoop \else
      #4\relax\expandafter\@tforloop\fi#2\@@#3{#4}}
\def\setlength#1#2{#1 #2\relax}
\def\addtolength#1#2{\advance#1 #2\relax}
\DeclareRobustCommand{\MakeUppercase}[1]{{\protected@edef\reserved@a{#1}\expandafter\uppercase\expandafter{\reserved@a}}}
\DeclareRobustCommand{\MakeLowercase}[1]{{\protected@edef\reserved@a{#1}\expandafter\lowercase\expandafter{\reserved@a}}}
\let\uppercase@\uppercase
\let\lowercase@\lowercase
\catcode`\@=12
";
