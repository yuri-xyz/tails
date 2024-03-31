; ModuleID = 'tests.literals'
source_filename = "tests.literals"

@string_literal = private unnamed_addr constant [5 x i8] c"test\00", align 1

define private void @tests_literals.tests() {
fn.entry:
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 1)
  call void (i32, ...) @emit(i32 -1)
  call void (i32, ...) @emit(i32 -2147483648)
  call void (i32, ...) @emit(i32 -2147483647)
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 0)
  call void (i32, ...) @emit(i32 1)
  call void (i8, ...) @emit(i8 1)
  call void (i16, ...) @emit(i16 1)
  call void (i32, ...) @emit(i32 1)
  call void (i64, ...) @emit(i64 1)
  call void (ptr, ...) @emit(ptr @string_literal)
  call void (i8, ...) @emit(i8 97)
  ret void
}

declare void @emit(...)