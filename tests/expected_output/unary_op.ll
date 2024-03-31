; ModuleID = 'tests.unary_op'
source_filename = "tests.unary_op"

define private void @tests_unary_op.tests() {
fn.entry:
  call void (i1, ...) @emit(i1 false)
  call void (i1, ...) @emit(i1 true)
  call void (i1, ...) @emit(i1 true)
  call void (i32, ...) @emit(i32 -1)
  call void (i32, ...) @emit(i32 -123)
  call void (i32, ...) @emit(i32 123)
  ret void
}

declare void @emit(...)