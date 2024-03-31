; ModuleID = 'tests.as_int_to_int'
source_filename = "tests.as_int_to_int"

define private void @tests_as_int_to_int.tests() {
fn.entry:
  call void (i32, ...) @emit(i32 123)
  call void (i32, ...) @emit(i32 123)
  call void (i32, ...) @emit(i32 123)
  call void (i32, ...) @emit(i32 123)
  call void (i16, ...) @emit(i16 123)
  call void (i32, ...) @emit(i32 123)
  call void (i64, ...) @emit(i64 123)
  call void (i32, ...) @emit(i32 123)
  call void (i16, ...) @emit(i16 123)
  call void (i8, ...) @emit(i8 123)
  ret void
}

declare void @emit(...)