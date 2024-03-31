; ModuleID = 'tests.constant'
source_filename = "tests.constant"

@tests_constant.A = addrspace(4) global i32 123

define private void @tests_constant.tests() {
fn.entry:
  %access.constant = load i32, ptr addrspace(4) @tests_constant.A, align 4
  %int.add_op = add i32 %access.constant, 3
  %int.add_op1 = add i32 6, %int.add_op
  %access.constant.cached = load i32, ptr addrspace(4) @tests_constant.A, align 4
  call void (i32, ...) @emit(i32 %access.constant.cached)
  call void (i1, ...) @emit(i1 true)
  call void (i1, ...) @emit(i1 false)
  call void (i32, ...) @emit(i32 3)
  call void (i32, ...) @emit(i32 3)
  call void (i32, ...) @emit(i32 6)
  %access.constant.cached2 = load i32, ptr addrspace(4) @tests_constant.A, align 4
  %int.add_op3 = add i32 %access.constant.cached2, 3
  %int.add_op4 = add i32 6, %int.add_op3
  call void (i32, ...) @emit(i32 %int.add_op4)
  ret void
}

declare void @emit(...)