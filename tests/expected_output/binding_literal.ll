; ModuleID = 'tests.binding_literal'
source_filename = "tests.binding_literal"

define private void @tests_binding_literal.tests() {
fn.entry:
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 1)
  call void (i32, ...) @emit(i32 -2147483648)
  call void (i32, ...) @emit(i32 -2147483647)
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 1)
  call void (i8, ...) @emit(i8 1)
  call void (i16, ...) @emit(i16 1)
  call void (i32, ...) @emit(i32 1)
  call void (i64, ...) @emit(i64 1)
  call void (float, ...) @emit(float 3.140000e+02)
  call void (ptr, ...) @emit(ptr null)
  call void (i8, ...) @emit(i8 97)
  ret void
}

declare void @emit(...)