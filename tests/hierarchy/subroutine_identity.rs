//! §13.3.1, §6.21, §13.4.1, §3.14, §14.3: one subroutine declaration has one
//! static storage, one static initialization, one static result and one frame
//! scope, whichever table entry a call reaches it through. A name that rivals
//! the subroutine's or its static's stays its own object.

use xezim::simulate;

/// Every line the simulation prints, in order.
fn prints_exactly(src: &str, expected: &[&str]) {
    let sim = simulate(src, 1000).expect("simulate");
    let got: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(got, expected);
}

#[test]
fn a_static_local_is_one_cell_for_bare_and_path_calls() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g
    task t(); static int s = 0; s++; $display("s=%0d", s); endtask
    initial begin t(); g.t(); t(); end
  end
  initial #1 $finish;
endmodule
"#,
        &["s=1", "s=2", "s=3"],
    );
}

/// §23.6: `f.n` names the static through the function's own scope.
#[test]
fn a_static_local_is_one_cell_for_bare_and_path_calls_and_path_reads() {
    prints_exactly(
        r#"
module top;
  if (1) begin : a
    function int f(); static int n = 0; n++; return n; endfunction
    initial begin int r; $display("a1 %0d", f()); #2 r = f(); $display("a2 %0d n=%0d", r, f.n); end
  end
  initial begin #1 $display("top %0d", a.f()); #4 $display("top %0d", a.f.n); $finish; end
endmodule
"#,
        &["a1 1", "top 2", "a2 3 n=3", "top 3"],
    );
}

#[test]
fn an_uninitialized_static_local_is_one_cell_for_bare_and_path_calls() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g
    task t(); static int s; s++; $display("s=%0d", s); endtask
    initial begin t(); g.t(); t(); end
  end
  initial #1 $finish;
endmodule
"#,
        &["s=1", "s=2", "s=3"],
    );
}

/// A static function that exits without assigning returns its previous value.
#[test]
fn a_static_functions_result_persists_per_declaration() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g1 function int f(int a); if (a) f = 7; endfunction end
  if (1) begin : g2 function int f(int a); if (a) f = 9; endfunction end
  initial begin
    int a, b, c, d;
    a = g1.f(1); b = g2.f(1); c = g1.f(0); d = g2.f(0);
    $display("%0d %0d %0d %0d", a, b, c, d);
    $finish;
  end
endmodule
"#,
        &["7 9 7 9"],
    );
}

#[test]
fn a_static_initializer_runs_once_across_spellings() {
    prints_exactly(
        r#"
module top;
  function int bump(); static int b = 0; b++; $display("bump %0d", b); return b; endfunction
  if (1) begin : g
    function int f(); static int s = bump(); s++; return s; endfunction
    initial begin #1 $display("f %0d", f()); #1 $display("g %0d", g.f()); end
  end
  initial #3 $finish;
endmodule
"#,
        &["bump 1", "f 2", "g 3"],
    );
}

#[test]
fn a_block_task_called_by_path_keeps_its_modules_timescale() {
    prints_exactly(
        r#"
`timescale 1us/1ns
module top; m u(); initial #1 $finish; endmodule
`timescale 1ns/1ps
module m;
  if (1) begin : g
    task automatic t(); #3; $display("%0d %f", $time, $realtime); endtask
    initial begin t(); g.t(); end
  end
endmodule
"#,
        &["3 3.000000", "6 6.000000"],
    );
}

#[test]
fn a_block_task_called_by_path_finds_the_modules_clocking_block() {
    prints_exactly(
        r#"
module m; logic c=0; always #1 c=~c; int d=5; clocking cb @(posedge c); input d; endclocking
  if (1) begin : g task automatic t(); @(cb); $display("%0d %0d", $time, cb.d); endtask initial begin t(); g.t(); end end
endmodule
module top; m u(); initial #9 $finish; endmodule
"#,
        &["1 5", "3 5"],
    );
}

/// Ten posedges at 5, 15, ..., 95: `w` counts 1 to 10.
#[test]
fn a_static_task_called_by_path_from_a_clocked_process_keeps_its_local() {
    prints_exactly(
        r#"
module top;
  logic clk = 0; always #5 clk = ~clk;
  if (1) begin : g
    int acc = 0;
    task tk(); static int w = 0; w++; acc += w; endtask
    always @(posedge clk) g.tk();
  end
  initial begin #100 $display("%0d", g.acc); $finish; end
endmodule
"#,
        &["55"],
    );
}

#[test]
fn a_static_module_task_keeps_its_local_in_a_clocked_process() {
    prints_exactly(
        r#"
module top;
  logic clk = 0; always #5 clk = ~clk;
  int acc = 0;
  task tk(); static int w = 0; w++; acc += w; endtask
  always @(posedge clk) tk();
  initial begin #100 $display("%0d", acc); $finish; end
endmodule
"#,
        &["55"],
    );
}

/// §21.2.1.5.
#[test]
fn percent_m_in_a_block_subroutine_names_the_block_for_path_calls() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g_a task automatic look(); $display("m=%m"); endtask end
  if (1) begin : g_b task automatic look(); $display("m=%m"); endtask end
  initial begin g_a.look(); g_b.look(); $finish; end
endmodule
"#,
        &["m=top.g_a.look", "m=top.g_b.look"],
    );
}

#[test]
fn a_block_function_called_by_path_assigns_its_own_result() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g function automatic int s(int n); s = 0; repeat (n) s += 2; endfunction end
  initial begin $display("%0d %0d", g.s(3), g.s(5)); $finish; end
endmodule
"#,
        &["6 10"],
    );
}

#[test]
fn a_block_function_called_by_path_assigns_its_own_result_in_an_instance() {
    prints_exactly(
        r#"
module m;
  if (1) begin : g
    function automatic int s(int n); s = 0; repeat (n) s += 2; endfunction
    initial $display("%0d", g.s(3));
  end
endmodule
module top; m u(); endmodule
"#,
        &["6"],
    );
}

#[test]
fn same_named_tasks_with_static_locals_in_two_packages_run_their_own_bodies() {
    prints_exactly(
        r#"
package p; task t(); static int s=0; $display("p"); endtask endpackage
package q; task t(); static int s=0; $display("q"); endtask endpackage
module top; initial begin p::t(); q::t(); end endmodule
"#,
        &["p", "q"],
    );
}

#[test]
fn a_module_task_with_a_static_local_is_not_replaced_by_a_package_task() {
    prints_exactly(
        r#"
package p; task t(); static int s=0; $display("p"); endtask endpackage
module top; task t(); static int s=0; $display("m"); endtask initial begin p::t(); t(); end endmodule
"#,
        &["p", "m"],
    );
}

/// §13.4, §23.9: inside its own body a function's name in call position is
/// the function, not the result variable, and an instance's call stays there.
#[test]
fn a_recursive_call_in_an_instance_runs_that_instances_function() {
    prints_exactly(
        r#"
module m;
  function automatic int f(int n); if (n <= 0) f = 0; else f = 1 + f(n - 1); endfunction
  initial $display("%0d", f(3));
endmodule
module top;
  function automatic int f(int n); f = 100; endfunction
  m u();
endmodule
"#,
        &["3"],
    );
}

/// A recursive call under a block's key stays in that block.
#[test]
fn a_recursive_block_function_called_by_path_recurses_into_its_own_block() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g1
    function automatic int f(int n); if (n <= 0) f = 1; else f = 10 + f(n - 1); endfunction
  end
  if (1) begin : g2
    function automatic int f(int n); if (n <= 0) f = 2; else f = 20 + f(n - 1); endfunction
  end
  initial begin $display("%0d %0d", g1.f(2), g2.f(2)); $finish; end
endmodule
"#,
        &["21 42"],
    );
}

/// §23.6: `f.n` written in `f` is its static, in an instance and under a path.
#[test]
fn a_function_reads_its_own_static_through_its_name_in_an_instance() {
    prints_exactly(
        r#"
module m;
  function int f(); static int n = 0; n++; return f.n + 10; endfunction
  if (1) begin : g
    function int h(); static int k = 0; k++; return h.k + 20; endfunction
    initial begin int a, b; #1 a = h(); b = g.h(); $display("%0d %0d", a, b); end
  end
  initial begin int r; $display("%0d", f()); r = f(); $display("%0d %0d", r, f.n); end
endmodule
module top; m u(); endmodule
"#,
        &["11", "12 2", "21 22"],
    );
}

/// §13.4.1: member and string writes reach the result under every spelling.
#[test]
fn struct_and_string_results_work_in_an_instance_and_under_a_path() {
    prints_exactly(
        r#"
typedef struct { int a; int b; } s_t;
module m;
  function automatic s_t f(int x); f.a = x; f.b = x + 1; endfunction
  function automatic string nm(int x); nm = "v"; if (x > 0) nm = {nm, "1"}; endfunction
  if (1) begin : g
    function automatic s_t f2(int x); f2.a = x; f2.b = x + 2; endfunction
    function automatic string nm2(int x); nm2 = "w"; endfunction
  end
  initial begin
    s_t r;
    r = f(1); $display("%0d %0d", r.a, r.b);
    r = g.f2(2); $display("%0d %0d", r.a, r.b);
    $display("%s %s", nm(1), g.nm2(0));
  end
endmodule
module top; m u(); endmodule
"#,
        &["1 2", "2 4", "v1 w"],
    );
}

/// §13.4.1: a collection result fills in place in an instance and under a path.
#[test]
fn a_collection_result_is_filled_in_an_instance_and_under_a_path() {
    prints_exactly(
        r#"
typedef int aa_t[string];
typedef int q_t[$];
module m;
  function automatic aa_t mk(); mk["a"] = 1; mk["b"] = 2; $display("in %0d", mk.num()); endfunction
  function automatic q_t mq(); mq.push_back(1); mq.push_back(2); endfunction
  if (1) begin : g function automatic aa_t mk3(); mk3["a"] = 1; mk3["b"] = 2; mk3["c"] = 3; endfunction end
  initial begin
    aa_t v;
    q_t q;
    v = mk(); $display("%0d", v.num());
    q = mq(); $display("%0d", q.size());
    v = g.mk3(); $display("%0d %0d", v.num(), v["c"]);
  end
endmodule
module top; m u(); endmodule
"#,
        &["in 2", "2", "2", "3 3"],
    );
}

/// §13.3.1, §26.3: a package subroutine has one static across bare and
/// qualified calls, and two packages do not share one.
#[test]
fn a_package_subroutines_static_is_one_cell_per_declaration() {
    prints_exactly(
        r#"
package p;
  function int cnt(); static int n = 0; n++; return n; endfunction
  task tk(); static int c = 0; c++; $display("tk %0d", c); endtask
endpackage
package q;
  function int cnt(); static int n = 100; n++; return n; endfunction
endpackage
module top;
  import p::*;
  initial begin
    $display("%0d", cnt());
    $display("%0d", p::cnt());
    $display("%0d", q::cnt());
    tk();
    p::tk();
    $finish;
  end
endmodule
"#,
        &["1", "2", "101", "tk 1", "tk 2"],
    );
}

/// §13.4.1: the result variable is internal to the function.
#[test]
fn a_collection_result_leaves_a_same_named_top_queue_alone() {
    prints_exactly(
        r#"
typedef int q_t[$];
module top; int f[$] = '{1,2,3}; m u(); initial #1 $display("%0d", f.size()); endmodule
module m;
  function automatic q_t f(); q_t x; x.push_back(1); return x; endfunction
  initial begin q_t r; r = f(); $display("%0d", r.size()); end
endmodule
"#,
        &["1", "3"],
    );
}

/// §13.4.1.
#[test]
fn a_fixed_array_result_leaves_a_same_named_top_array_alone() {
    prints_exactly(
        r#"
typedef logic [7:0] ua_t[2];
module top;
  logic [7:0] f[2] = '{8'h11, 8'h22};
  if (1) begin : g function automatic ua_t f(); f[0] = 8'haa; f[1] = 8'hbb; endfunction end
  initial begin ua_t r; r = g.f(); $display("%h %h", r[0], r[1]); $display("%h %h", f[0], f[1]); end
endmodule
"#,
        &["aa bb", "11 22"],
    );
}

/// §23.7(a), §13.4.1: `f.a` selects a member of the result, not the local `a`.
#[test]
fn a_struct_results_member_write_survives_a_local_of_that_name_in_an_instance() {
    prints_exactly(
        r#"
typedef struct { int a; int b; } s_t;
module m;
  function automatic s_t f(int x); int a; a = x; f.a = a + 1; f.b = 3; endfunction
  initial begin s_t r; r = f(5); $display("%0d %0d", r.a, r.b); end
endmodule
module top; m u(); endmodule
"#,
        &["6 3"],
    );
}

/// §23.7(a), §13.4.1.
#[test]
fn a_class_results_member_write_survives_a_local_of_that_name_in_an_instance() {
    prints_exactly(
        r#"
class C; int v; function new(int x); v = x; endfunction endclass
module m;
  function automatic C f(int a); int v; v = 1; f = new(0); f.v = a; endfunction
  initial begin C r; r = f(5); $display("%0d", r.v); end
endmodule
module top; m u(); endmodule
"#,
        &["5"],
    );
}

/// §23.7(a): `f.n` selects a member of the data object `f`.
#[test]
fn a_struct_member_is_not_the_static_of_a_same_named_block_function() {
    prints_exactly(
        r#"
typedef struct { int n; } s_t;
module top;
  s_t f;
  if (1) begin : g function int f(); static int n = 9; n++; return n; endfunction end
  initial begin f.n = 3; $display("%0d %0d", f.n, g.f()); $finish; end
endmodule
"#,
        &["3 10"],
    );
}

/// §23.6: a task defines a hierarchical level.
#[test]
fn a_block_tasks_static_struct_member_reads_through_the_tasks_name() {
    prints_exactly(
        r#"
typedef struct {int a;} st;
module top;
  if (1) begin : g task t(); static st s = '{a:5}; $display("%0d", t.s.a); endtask initial t(); end
endmodule
"#,
        &["5"],
    );
}

/// §23.6: so does a named block in it.
#[test]
fn a_block_tasks_named_block_static_reads_through_the_tasks_name_in_an_instance() {
    prints_exactly(
        r#"
module m;
  if (1) begin : g task t(); begin : b static int v = 5; end $display("%0d", t.b.v); endtask initial t(); end
endmodule
module top; m u(); endmodule
"#,
        &["5"],
    );
}

/// §23.6: read from a sibling process in the block.
#[test]
fn a_block_functions_statics_read_through_its_name_from_a_sibling_process() {
    prints_exactly(
        r#"
typedef struct {int a;} st;
module top;
  if (1) begin : g
    function int f(); static st s = '{a:5}; begin : b static int v = 6; end return 0; endfunction
    initial $display("%0d %0d", f.s.a, f.b.v);
  end
endmodule
"#,
        &["5 6"],
    );
}

/// §13.4.1: in a hierarchical name the function's name also indicates its scope.
#[test]
fn a_block_function_reads_its_static_struct_member_through_its_own_name() {
    prints_exactly(
        r#"
typedef struct {int a;} st;
module top;
  if (1) begin : g
    function int f(); static st s = '{a:5}; return f.s.a; endfunction
    initial $display("%0d", f());
  end
endmodule
"#,
        &["5"],
    );
}

/// §13.4.1, §23.6.
#[test]
fn a_block_function_reads_its_named_block_static_through_its_own_name() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g
    function int f(); begin : b static int v = 5; end return f.b.v; endfunction
    initial $display("%0d", f());
  end
endmodule
"#,
        &["5"],
    );
}

/// §13.4.1, §7.4: `w[1]` selects the top's packed element during a call to a
/// function that is also named `w`.
#[test]
fn a_function_named_like_a_packed_top_signal_leaves_its_selects_alone() {
    prints_exactly(
        r#"
module top;
  logic [3:0][7:0] w = 32'h44332211;
  function automatic logic [7:0] rd(); return w[1]; endfunction
  m u();
endmodule
module m;
  function automatic logic [31:0] w(); w = top.rd(); endfunction
  initial $display("%h", w());
endmodule
"#,
        &["00000022"],
    );
}

/// §13.4.1, §23.9: the result variable hides the interface port in the block.
#[test]
fn a_block_functions_result_is_not_a_same_named_interface_port() {
    prints_exactly(
        r#"
interface ifc; int n = 3; endinterface
module c(ifc f);
  if (1) begin : g function automatic int f(); f = 1; endfunction end
  initial #1 begin $display("%0d", g.f()); $display("%0d", f.n); end
endmodule
module top; ifc i(); c u(i); endmodule
"#,
        &["1", "3"],
    );
}

/// §23.9: a recursive call in a `with` clause is the innermost `f`.
#[test]
fn a_recursive_call_in_a_with_clause_runs_that_instances_function() {
    prints_exactly(
        r#"
module m;
  function automatic int f(int n);
    int r;
    if (n <= 0) return 1;
    begin int q[$] = '{1,2}; int s[$]; s = q.find(x) with (x == f(n - 1)); r = s.size(); end
    return r;
  endfunction
  initial $display("%0d", f(1));
endmodule
module top; function automatic int f(int n); return 100; endfunction m u(); endmodule
"#,
        &["1"],
    );
}

/// §13.4.1. Pins base behaviour: Verilator rejects `f.n` on an int result.
#[test]
fn a_block_function_called_by_path_reads_its_own_static_through_its_name() {
    prints_exactly(
        r#"
module top;
  if (1) begin : g1 function int f(); static int n = 10; n++; return f.n; endfunction end
  if (1) begin : g2 function int f(); static int n = 20; n++; return f.n; endfunction end
  initial begin $display("%0d %0d", g1.f(), g2.f()); $finish; end
endmodule
"#,
        &["11 21"],
    );
}
