% changes/checkpoint.ch -- the points where a resident engine may take a
% checkpoint (lane XETEX-INCR; DESIGN.md sections 5.1 and 5.2, and the
% pdfTeX engine's crates/flashtex-engine/changes/checkpoint.ch, whose
% mechanism this is, against xetex.web's text).
%
% A checkpoint captures the whole engine state, so it can only be taken where
% the Pascal program itself holds nothing on the (Rust) call stack that the
% state does not describe. That point is |big_switch| in |main_control|:
% between two commands, with every local of |main_control| dead. There the
% engine calls the hand-written |flashtex_checkpoint_hook| (src/ckpt.rs)
% whenever |ckpt_request| is nonzero, and a restored run re-enters
% |main_control| at |big_switch| (|ckpt_resuming| skips \.{\\everyjob},
% which the uninterrupted run inserted long before): |resume_main_control|.
%
% The begin-document snapshot $S_0$ (section 5.1) is armed without looking at
% any token: the host names a control sequence (\.{\\document}, whose
% expansion LaTeX's \.{\\begin\{document\}} ends in); when |macro_call| pushes
% its body, the input level is remembered, and when |pop_input| goes below
% that level the body -- with everything it expanded to -- has been
% consumed, and the next |big_switch| takes $S_0$. After each shipout, when
% the host asks (|ckpt_on_shipout|), the next |big_switch| takes a page
% checkpoint (section 5.2), from which a later run restarts near its edit.
% None of this changes what
% the program computes: the new variables are read only by these tests and by
% the hook, and the hook itself only reads the state. (XeTeX's |null| is not
% zero, so ``no control sequence armed'' is 0, which no control sequence is.)
%
% GPL-2.0-or-later, like the rest of crates/flashtex-xetex.

@x l.8240 of the merged WEB - leaving the armed input level requests the checkpoint
@d pop_input==@t@> {leave an input level, re-enter the old}
  begin decr(input_ptr); cur_input:=input_stack[input_ptr];
  end
@y
@d pop_input==@t@> {leave an input level, re-enter the old}
  begin decr(input_ptr); cur_input:=input_stack[input_ptr];
  if input_ptr<ckpt_arm_level then
    begin ckpt_arm_level:=0; ckpt_request:=2;
    end;
  end
@z

@x l.9475 of the merged WEB - expanding the armed control sequence arms the checkpoint
begin_token_list(ref_count,macro); name:=warning_index; loc:=link(r);
@y
begin_token_list(ref_count,macro); name:=warning_index; loc:=link(r);
if ckpt_arm_cs<>0 then if warning_index=ckpt_arm_cs then
  begin ckpt_arm_level:=input_ptr; ckpt_arm_cs:=0;
  end;
@z

@x l.15632 of the merged WEB - a checkpoint after every shipout, when asked (section 5.2)
update_terminal; {progress report}
@<Flush the box from memory, showing statistics if requested@>;
end;
@y
update_terminal; {progress report}
@<Flush the box from memory, showing statistics if requested@>;
if ckpt_on_shipout<>0 then ckpt_request:=ckpt_on_shipout;
end;
@z

@x l.24680 of the merged WEB - checkpoints are taken, and resumed, at |big_switch|
begin if every_job<>null then begin_token_list(every_job,every_job_text);
big_switch: flashtex_poll; get_x_token;@/
@y
begin if ckpt_resuming then ckpt_resuming:=false
else if every_job<>null then begin_token_list(every_job,every_job_text);
big_switch: flashtex_poll; if ckpt_request<>0 then flashtex_checkpoint_hook;
get_x_token;@/
@z

@x the new sections go at the end of part 54
@* \[55] Index.
@y
@ The checkpoint layer's variables. They are zero when the engine is made,
and only the host (through |flashtex_checkpoint_hook|) sets them otherwise,
so an engine that is not asked for checkpoints never calls the hook.

@<Glob...@>=
@!ckpt_request:integer; {nonzero: call |flashtex_checkpoint_hook| at |big_switch|}
@!ckpt_arm_cs:pointer; {expanding this control sequence arms the checkpoint (0: none)}
@!ckpt_arm_level:integer; {|input_ptr| with the armed body on top, or 0}
@!ckpt_resuming:boolean; {enter |main_control| at |big_switch|}
@!ckpt_on_shipout:integer; {nonzero: request a checkpoint after each shipout}

@ A run restored from a checkpoint taken at |big_switch| goes on from there
to the end, as the main program does after |main_control|.

@<Last-minute...@>=
procedure resume_main_control;
begin ckpt_resuming:=true;
main_control; {come to life}
final_cleanup; {prepare for death}
close_files_and_terminate;
ready_already:=0;
end;

@ @<Declare the routines of \XeTeX's C parts@>=
procedure flashtex_checkpoint_hook; external;
  {at |big_switch| when |ckpt_request| is nonzero (src/ckpt.rs)}

@* \[55] Index.
@z
